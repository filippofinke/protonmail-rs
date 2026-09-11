//! Human-verification (CAPTCHA) resolver for the CLI.
//!
//! Proton's captcha widget can only be *iframed* by `https://mail.proton.me`
//! (its `frame-ancestors` CSP), so we can't embed it. Instead we open it as a
//! TOP-LEVEL page (CSP `frame-ancestors` doesn't apply to top-level nav) and
//! capture the token with a one-line console snippet the user pastes: when the
//! widget solves, it `postMessage`s `{type:'pm_captcha', token}` to
//! `window.parent` which — for a top-level page — is the page itself, so a
//! `message` listener on `window` receives it and `fetch()`es it back to our
//! local listener. No copy-paste, no extra deps.
//!
//! Resolution order:
//!   1. pre-supplied `--captcha-token` (sent verbatim),
//!   2. open captcha top-level + console snippet → auto-return to local server,
//!   3. user can always copy the token and re-run with `--captcha-token`.
//!
//! The captcha HV token is sent RAW (no `<challenge>:` prefix — that form is
//! only for email/sms codes).
//!
//! `--captcha-chrome` instead follows Proton's external-browser flow (the one
//! Proton Bridge's CLI uses): open `verify.proton.me` for the challenge in an
//! isolated Chrome window; that page registers the solved captcha against the
//! challenge itself, and once the user presses ENTER the request is retried
//! with the original challenge token. Nothing is captured from the page.

use proton_core::{Error, HvChallenge, HvResolver, Result};
use std::future::Future;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::Arc;
use std::time::{Duration, Instant};

const HV_TIMEOUT: Duration = Duration::from_secs(300);
/// Preferred local port (keeps a saved bookmarklet reusable across runs).
const PREFERRED_PORT: u16 = 47821;
/// Verification page used when the challenge carries no usable `WebUrl`.
const DEFAULT_VERIFY_ORIGIN: &str = "https://verify.proton.me";

/// Build the HV resolver.
/// - `api_base`: API root (used to derive the captcha widget URL).
/// - `captcha_token`: short-circuits the interactive flow when present.
/// - `use_chrome`: open the captcha in an isolated Chrome window instead of the default browser.
pub fn resolver(captcha_token: Option<String>, api_base: String, use_chrome: bool) -> HvResolver {
    Arc::new(move |chal: HvChallenge| {
        let pre = captcha_token.clone();
        let api = api_base.clone();
        Box::pin(solve(chal, pre, api, use_chrome))
            as Pin<Box<dyn Future<Output = Result<(String, String)>> + Send>>
    })
}

async fn solve(
    chal: HvChallenge,
    pre: Option<String>,
    api_base: String,
    use_chrome: bool,
) -> Result<(String, String)> {
    if !chal.methods.is_empty() && !chal.methods.iter().any(|m| m == "captcha") {
        return Err(Error::Other(format!(
            "human verification requires an unsupported method: {:?} (only 'captcha' is handled)",
            chal.methods
        )));
    }
    if let Some(t) = pre {
        return Ok((t, "captcha".to_string()));
    }

    if use_chrome {
        let url = verify_url(&chal);
        tokio::task::spawn_blocking(move || run_external_flow(&url))
            .await
            .map_err(|e| Error::Other(format!("hv task: {e}")))??;
        // The verify page attached the solved captcha to the challenge, so the
        // retry echoes the challenge token itself (as Proton Bridge does).
        return Ok((chal.token, "captcha".to_string()));
    }

    let (endpoint, _origin) = captcha_endpoint(&api_base);
    let captcha_url = format!(
        "{endpoint}?Token={}&ForceWebMessaging=1",
        urlencode(&chal.token)
    );

    tokio::task::spawn_blocking(move || run_manual_flow(&captcha_url))
        .await
        .map_err(|e| Error::Other(format!("hv task: {e}")))?
}

/// Proton's standalone verification page for `chal`, restricted to the captcha
/// method (the only one whose external-browser result the retry can redeem).
/// Keeps the host of the server-provided `WebUrl` when it is https.
fn verify_url(chal: &HvChallenge) -> String {
    let origin = chal
        .web_url
        .strip_prefix("https://")
        .and_then(|rest| rest.split(['/', '?', '#']).next())
        .filter(|host| !host.is_empty())
        .map_or_else(
            || DEFAULT_VERIFY_ORIGIN.to_string(),
            |host| format!("https://{host}"),
        );
    format!("{origin}/?methods=captcha&token={}", urlencode(&chal.token))
}

/// `--captcha-chrome`: open the verification page in an isolated Chrome window
/// and wait (bounded) for the user to confirm completion with ENTER.
fn run_external_flow(url: &str) -> Result<()> {
    eprintln!("\n── Human verification (CAPTCHA) ──");
    let mut chrome = match launch_chrome(url, &std::process::id().to_string()) {
        Ok(c) => {
            eprintln!("Opened the verification page in an isolated Chrome window.");
            Some(c)
        }
        Err(msg) => {
            eprintln!("Chrome launch failed ({msg}); opening your default browser instead.");
            if open_browser(url).is_err() {
                eprintln!("Could not open a browser. Open this URL manually:\n{url}");
            }
            None
        }
    };
    eprintln!(
        "Solve the CAPTCHA there. When the page says verification is complete, \
         press ENTER here (waiting up to {} minutes).",
        HV_TIMEOUT.as_secs() / 60
    );

    let outcome = wait_for_confirmation(&spawn_enter_reader(), HV_TIMEOUT, || {
        chrome
            .as_mut()
            .is_some_and(|(c, _)| matches!(c.try_wait(), Ok(Some(_))))
    });

    // Close the isolated Chrome window we opened (if any), then remove its
    // throwaway profile. Best effort: a leftover only wastes temp space.
    if let Some((mut c, profile)) = chrome {
        let _ = c.kill();
        let _ = c.wait();
        let _ = std::fs::remove_dir_all(&profile);
    }
    if outcome.is_ok() {
        eprintln!("Verification confirmed — retrying.");
    }
    outcome
}

/// What the stdin reader observed.
enum Confirm {
    Enter,
    Closed,
}

/// Read one line from stdin on a helper thread. If we time out first, the
/// thread stays parked on stdin, which is harmless: the login then fails.
fn spawn_enter_reader() -> mpsc::Receiver<Confirm> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let msg = match std::io::stdin().read_line(&mut line) {
            Ok(n) if n > 0 => Confirm::Enter,
            _ => Confirm::Closed,
        };
        let _ = tx.send(msg);
    });
    rx
}

/// Wait until the user confirms, stdin closes, or `timeout` elapses. A closed
/// Chrome window only warns: when another Chrome owns the profile the process
/// we spawned exits immediately while its window stays open.
fn wait_for_confirmation(
    rx: &mpsc::Receiver<Confirm>,
    timeout: Duration,
    mut chrome_exited: impl FnMut() -> bool,
) -> Result<()> {
    let deadline = Instant::now() + timeout;
    let mut warned = false;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(Error::Other(
                "human verification timed out waiting for ENTER".into(),
            ));
        }
        match rx.recv_timeout(left.min(Duration::from_millis(500))) {
            Ok(Confirm::Enter) => return Ok(()),
            Ok(Confirm::Closed) | Err(RecvTimeoutError::Disconnected) => {
                return Err(Error::Other(
                    "human verification aborted: stdin closed before ENTER was pressed".into(),
                ))
            }
            Err(RecvTimeoutError::Timeout) => {
                if !warned && chrome_exited() {
                    warned = true;
                    eprintln!(
                        "The Chrome window was closed. Press ENTER if verification \
                         completed, or Ctrl+C to abort."
                    );
                }
            }
        }
    }
}

/// Open the captcha top-level, print the capture snippet, wait for the token.
fn run_manual_flow(captcha_url: &str) -> Result<(String, String)> {
    // Prefer a fixed port so a saved bookmarklet keeps working across runs.
    let listener = TcpListener::bind(("127.0.0.1", PREFERRED_PORT))
        .or_else(|_| TcpListener::bind("127.0.0.1:0"))
        .map_err(|e| Error::Other(format!("cannot start HV helper server: {e}")))?;
    let port = listener
        .local_addr()
        .map_err(|e| Error::Other(format!("hv addr: {e}")))?
        .port();
    listener
        .set_nonblocking(true)
        .map_err(|e| Error::Other(format!("hv nonblocking: {e}")))?;

    let snippet = capture_snippet(port);
    let err = std::io::stderr();
    let mut e = err.lock();
    let _ = writeln!(e, "\n── Human verification (CAPTCHA) ──");
    let _ = writeln!(
        e,
        "A browser tab is opening at the captcha page. Then EITHER:"
    );
    let _ = writeln!(
        e,
        "\n  [A] Console: open DevTools → Console (F12) and paste once:"
    );
    let _ = writeln!(e, "      {snippet}");
    let _ = writeln!(
        e,
        "\n  [B] Bookmarklet (one-time): save this as a bookmark, then click it"
    );
    let _ = writeln!(
        e,
        "      on the captcha page (no console, no \"allow pasting\"):"
    );
    let _ = writeln!(e, "      javascript:{snippet}");
    let _ = writeln!(
        e,
        "\nSolve the CAPTCHA — the token returns here automatically."
    );
    let _ = writeln!(
        e,
        "(Fallback: copy the token, re-run login with --captcha-token <token>.)"
    );
    let _ = writeln!(e, "\nCaptcha URL: {captcha_url}");
    let _ = e.flush();

    let _ = open_browser(captcha_url);

    let deadline = Instant::now() + HV_TIMEOUT;
    loop {
        if Instant::now() >= deadline {
            break Err(Error::Other("human verification timed out".into()));
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                if let Some(token) = handle_conn(&mut stream) {
                    if !token.is_empty() {
                        let _ = writeln!(e, "Token received — continuing login.");
                        break Ok((token, "captcha".to_string()));
                    }
                }
            }
            Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(150));
            }
            Err(err) => break Err(Error::Other(format!("hv accept: {err}"))),
        }
    }
}

/// The console one-liner the user pastes. On the `pm_captcha` message it
/// NAVIGATES the tab to our local listener (the page's CSP `connect-src https:
/// wss:` blocks `fetch` to http://localhost, but top-level navigation is not
/// governed by connect-src). Shows a small status banner until then.
fn capture_snippet(port: u16) -> String {
    format!(
        "(function(){{var b=document.createElement('div');b.style='position:fixed;left:0;right:0;bottom:0;background:#111;color:#0f0;font:13px monospace;z-index:2147483647;padding:8px';b.textContent='protonmail-cli: solve the captcha — token returns automatically';document.body.appendChild(b);addEventListener('message',function(e){{var d=e.data;if(typeof d==='string'){{try{{d=JSON.parse(d)}}catch(x){{return}}}}if(d&&d.type==='pm_captcha'&&d.token){{b.style.background='#030';b.textContent='protonmail-cli: token captured — returning…';location.href='http://127.0.0.1:{port}/token?value='+encodeURIComponent(d.token)}}}});console.log('protonmail-cli: ready (pm_captcha listener installed)')}})()"
    )
}

/// Capture the token from `GET /token?value=...`; CORS hides our response from
/// the page, but the request still reaches us.
fn handle_conn(stream: &mut std::net::TcpStream) -> Option<String> {
    // On macOS/BSD the accepted socket inherits the listener's non-blocking
    // mode; without this a request that arrives after `accept` is dropped.
    stream.set_nonblocking(false).ok();
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf).ok()?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = req.lines().next()?.split_whitespace().nth(1)?;

    if let Some(q) = path.strip_prefix("/token") {
        let token = q
            .split_once("value=")
            .map(|(_, v)| urldecode(v.split('&').next().unwrap_or("")))
            .unwrap_or_default();
        let body =
            "<!doctype html><meta charset=utf-8><body style=\"font:16px sans-serif;padding:40px\">\
                    <h2>✓ Verified</h2><p>You can close this tab and return to your terminal.</p>";
        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        return Some(token);
    }
    let _ = write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok"
    );
    None
}

/// Derive the captcha widget endpoint + origin from the API base.
/// `https://mail.proton.me/api` -> (`https://mail-api.proton.me/core/v4/captcha`, origin).
fn captcha_endpoint(api_base: &str) -> (String, String) {
    let (scheme, host) = split_scheme_host(api_base);
    let is_ip_or_local = host.starts_with("127.")
        || host.starts_with("localhost")
        || host
            .split(':')
            .next()
            .unwrap_or("")
            .parse::<std::net::IpAddr>()
            .is_ok();
    if is_ip_or_local || !host.contains('.') {
        let origin = format!("{scheme}://{host}");
        (format!("{origin}/api/core/v4/captcha"), origin)
    } else {
        let (first, rest) = host.split_once('.').unwrap();
        let api_host = format!("{first}-api.{rest}");
        let origin = format!("{scheme}://{api_host}");
        (format!("{origin}/core/v4/captcha"), origin)
    }
}

fn split_scheme_host(url: &str) -> (String, String) {
    if let Some(i) = url.find("://") {
        let scheme = url[..i].to_string();
        let rest = &url[i + 3..];
        let host = rest.split('/').next().unwrap_or(rest).to_string();
        (scheme, host)
    } else {
        (
            "https".to_string(),
            url.split('/').next().unwrap_or(url).to_string(),
        )
    }
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn urldecode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(v);
                    i += 3;
                    continue;
                }
                out.push(b[i]);
                i += 1;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn open_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    let mut cmd = std::process::Command::new("open");
    #[cfg(target_os = "linux")]
    let mut cmd = std::process::Command::new("xdg-open");
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", ""]);
        c
    };
    cmd.arg(url);
    cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    cmd.spawn().map(|_| ())
}

/// Throwaway Chrome profile used only by protonmail-cli (never a user profile).
fn chrome_profile_dir(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("protonmail-cli-hv-{tag}"))
}

/// Open `url` in an isolated Chrome window (throwaway profile named by `tag`).
/// Returns the child process and its profile dir so both can be cleaned up
/// once verification completes.
fn launch_chrome(
    url: &str,
    tag: &str,
) -> std::result::Result<(std::process::Child, PathBuf), String> {
    let chrome = find_chrome().ok_or_else(|| "no Chrome/Chromium binary found".to_string())?;
    let profile = chrome_profile_dir(tag);
    std::process::Command::new(&chrome)
        .arg(format!("--user-data-dir={}", profile.display()))
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg(format!("--app={url}"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|child| (child, profile))
        .map_err(|e| format!("spawn {chrome}: {e}"))
}

fn find_chrome() -> Option<String> {
    #[cfg(target_os = "macos")]
    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
        "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    ];
    #[cfg(target_os = "linux")]
    let candidates = [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "brave-browser",
    ];
    #[cfg(target_os = "windows")]
    let candidates = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    ];
    for c in candidates {
        #[cfg(unix)]
        {
            if c.contains('/') {
                if std::path::Path::new(c).exists() {
                    return Some(c.to_string());
                }
            } else if let Some(p) = which(c) {
                return Some(p);
            }
        }
        #[cfg(not(unix))]
        {
            if std::path::Path::new(c).exists() {
                return Some(c.to_string());
            }
        }
    }
    None
}

#[cfg(unix)]
fn which(bin: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let p = dir.join(bin);
        if p.exists() {
            return Some(p.to_string_lossy().into_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captcha_endpoint_uses_api_subdomain() {
        let (url, origin) = captcha_endpoint("https://mail.proton.me/api");
        assert_eq!(url, "https://mail-api.proton.me/core/v4/captcha");
        assert_eq!(origin, "https://mail-api.proton.me");
    }

    #[test]
    fn captcha_endpoint_localhost_path_form() {
        let (url, _o) = captcha_endpoint("http://127.0.0.1:8080/api");
        assert_eq!(url, "http://127.0.0.1:8080/api/core/v4/captcha");
    }

    #[test]
    fn url_codec() {
        assert_eq!(urlencode("a b/c"), "a%20b%2Fc");
        assert_eq!(urldecode("a%20b%2Fc"), "a b/c");
        assert_eq!(urldecode("x+y"), "x y");
    }

    #[test]
    fn snippet_has_port() {
        assert!(capture_snippet(54321).contains("127.0.0.1:54321/token"));
        assert!(capture_snippet(1).contains("pm_captcha"));
    }

    #[test]
    fn local_server_captures_token() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();

        let handle = std::thread::spawn(move || -> Option<String> {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if Instant::now() >= deadline {
                    return None;
                }
                match listener.accept() {
                    Ok((mut s, _)) => {
                        if let Some(t) = handle_conn(&mut s) {
                            return Some(t);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    Err(_) => return None,
                }
            }
        });

        std::thread::sleep(Duration::from_millis(100));
        let mut c = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        c.write_all(b"GET /token?value=raw%3Atoken+abc HTTP/1.1\r\nHost: x\r\n\r\n")
            .unwrap();
        let mut resp = String::new();
        let _ = c.read_to_string(&mut resp);

        let got = handle.join().unwrap();
        assert_eq!(got.as_deref(), Some("raw:token abc"));
    }

    #[test]
    fn token_request_arriving_after_accept_is_not_dropped() {
        // On macOS an accepted socket inherits the listener's non-blocking mode,
        // so a request line that arrives after `accept` must still be read.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();

        let client = std::thread::spawn(move || {
            let mut c = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
            std::thread::sleep(Duration::from_millis(300));
            let _ = c.write_all(b"GET /token?value=synthetic HTTP/1.1\r\nHost: x\r\n\r\n");
            let mut resp = String::new();
            let _ = c.read_to_string(&mut resp);
        });

        let mut stream = loop {
            match listener.accept() {
                Ok((s, _)) => break s,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => panic!("accept: {e}"),
            }
        };
        assert_eq!(handle_conn(&mut stream).as_deref(), Some("synthetic"));
        drop(stream); // EOF for the client's read_to_string
        client.join().unwrap();
    }

    #[test]
    fn chrome_profile_dir_is_a_dedicated_temp_dir() {
        let dir = chrome_profile_dir("123");
        assert_eq!(dir.parent(), Some(std::env::temp_dir().as_path()));
        assert_eq!(dir.file_name().unwrap(), "protonmail-cli-hv-123");
    }

    fn chal(token: &str, web_url: &str) -> HvChallenge {
        HvChallenge {
            token: token.into(),
            methods: vec!["captcha".into(), "email".into()],
            web_url: web_url.into(),
        }
    }

    #[test]
    fn verify_url_keeps_web_url_host_and_forces_captcha() {
        let c = chal(
            "chal/1 x",
            "https://verify.proton.me/?methods=captcha,email&token=other",
        );
        assert_eq!(
            verify_url(&c),
            "https://verify.proton.me/?methods=captcha&token=chal%2F1%20x"
        );
    }

    #[test]
    fn verify_url_falls_back_for_missing_or_insecure_web_url() {
        let expected = "https://verify.proton.me/?methods=captcha&token=t";
        assert_eq!(verify_url(&chal("t", "")), expected);
        assert_eq!(
            verify_url(&chal("t", "http://attacker.example/?token=t")),
            expected
        );
    }

    #[test]
    fn confirmation_succeeds_on_enter() {
        let (tx, rx) = mpsc::channel();
        tx.send(Confirm::Enter).unwrap();
        assert!(wait_for_confirmation(&rx, Duration::from_secs(5), || false).is_ok());
    }

    #[test]
    fn confirmation_aborts_when_stdin_closes() {
        let (tx, rx) = mpsc::channel();
        tx.send(Confirm::Closed).unwrap();
        let err = wait_for_confirmation(&rx, Duration::from_secs(5), || false).unwrap_err();
        assert!(err.to_string().contains("stdin closed"));

        let (tx, rx) = mpsc::channel::<Confirm>();
        drop(tx);
        assert!(wait_for_confirmation(&rx, Duration::from_secs(5), || false).is_err());
    }

    #[test]
    fn confirmation_times_out_and_closed_chrome_only_warns() {
        let (_tx, rx) = mpsc::channel::<Confirm>();
        let mut polls = 0;
        let started = Instant::now();
        let err = wait_for_confirmation(&rx, Duration::from_millis(50), || {
            polls += 1;
            true
        })
        .unwrap_err();
        assert!(err.to_string().contains("timed out"));
        assert_eq!(polls, 1, "Chrome exit is checked, and warned about, once");
        assert!(started.elapsed() < Duration::from_secs(2));
    }
}
