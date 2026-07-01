//! Live integration tests against a real Proton account.
//!
//! These are **skipped** unless `PROTON_TEST_USER` / `PROTON_TEST_PASSWORD`
//! are set, so they are safe to leave in the normal `cargo test` run. The
//! send/trash test additionally requires `PROTON_TEST_SEND=1` (it sends a real
//! message to yourself and trashes it).
//!
//! ```bash
//! PROTON_TEST_USER=you@proton.me PROTON_TEST_PASSWORD=... \
//!   PROTON_TEST_TOTP=123456 PROTON_TEST_SEND=1 \
//!   cargo test -p proton-core --test live -- --nocapture --test-threads=1
//! ```

use proton_core::{Client, LoginOptions, SendOptions};

struct Creds {
    user: String,
    pass: String,
    totp: Option<String>,
    mailbox: Option<String>,
}

fn creds() -> Option<Creds> {
    let user = std::env::var("PROTON_TEST_USER").ok()?;
    let pass = std::env::var("PROTON_TEST_PASSWORD").ok()?;
    Some(Creds {
        user,
        pass,
        totp: std::env::var("PROTON_TEST_TOTP").ok(),
        mailbox: std::env::var("PROTON_TEST_MAILBOX_PASSWORD").ok(),
    })
}

async fn login(c: &Creds) -> Client {
    Client::login(LoginOptions {
        username: c.user.clone(),
        password: c.pass.clone(),
        totp: c.totp.clone(),
        mailbox_password: c.mailbox.clone(),
        profile: "live-test".into(),
        base_url: std::env::var("PROTON_TEST_API_URL").ok(),
        app_version: std::env::var("PROTON_TEST_APP_VERSION").ok(),
        user_agent: std::env::var("PROTON_TEST_USER_AGENT").ok(),
        hv: None,
    })
    .await
    .expect("login should succeed")
}

#[tokio::test]
async fn live_login_and_read_inbox() {
    let Some(c) = creds() else {
        eprintln!("skipping live_login_and_read_inbox (PROTON_TEST_USER not set)");
        return;
    };
    let client = login(&c).await;
    assert!(
        client.primary_email().is_some(),
        "should resolve a primary address"
    );

    let (total, msgs) = client
        .list_messages("inbox", 0, 5, false)
        .await
        .expect("list inbox");
    eprintln!("inbox total={total}, fetched={}", msgs.len());

    if let Some(first) = msgs.first() {
        let full = client
            .read_message(&first.id)
            .await
            .expect("read+decrypt first message");
        eprintln!(
            "first message: subject={:?} verdict={:?} body_len={}",
            full.meta.subject,
            full.verdict,
            full.body.len()
        );
    }
}

#[tokio::test]
async fn live_send_read_trash_roundtrip() {
    let Some(c) = creds() else {
        eprintln!("skipping live_send_read_trash_roundtrip (PROTON_TEST_USER not set)");
        return;
    };
    if std::env::var("PROTON_TEST_SEND").ok().as_deref() != Some("1") {
        eprintln!("skipping send roundtrip (set PROTON_TEST_SEND=1 to enable)");
        return;
    }
    let client = login(&c).await;
    let me = client.primary_email().expect("primary email").to_string();

    let token = format!("proton-rs selftest {}", std::process::id());
    let _id = client
        .send(&SendOptions {
            to: vec![me.clone()],
            subject: token.clone(),
            body: format!("Automated round-trip test: {token}"),
            ..Default::default()
        })
        .await
        .expect("send to self");

    // Give the backend a moment, then find it via search.
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let hits = client
        .search_messages(&proton_core::SearchOpts {
            keyword: Some(token.clone()),
            folder: Some("all".into()),
            limit: Some(5),
            ..Default::default()
        })
        .await
        .expect("search");
    assert!(!hits.is_empty(), "sent message should be findable");

    let full = client.read_message(&hits[0].id).await.expect("read back");
    assert!(
        full.body.contains(&token),
        "decrypted body should contain the token"
    );

    // Clean up: trash the found copies.
    let ids: Vec<String> = hits.iter().map(|m| m.id.clone()).collect();
    client.trash_messages(&ids).await.expect("trash");
    eprintln!("round-trip OK; trashed {} message(s)", ids.len());
}
