//! HTTP transport: typed request/response, Proton envelope handling, and
//! automatic 401-refresh / 429-rate-limit / 9001-human-verification retries.

mod headers;
mod retry;

use crate::error::{ApiError, Error, HvChallenge, Result};
use async_trait::async_trait;
use futures::future::BoxFuture;
use reqwest::Method;
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Request body.
#[derive(Debug, Clone)]
pub enum Body {
    /// No request body.
    Empty,
    /// JSON request body.
    Json(serde_json::Value),
    /// Raw bytes with an explicit content-type (set via `Request::raw`).
    Bytes(Vec<u8>),
}

/// A transport-level request.
#[derive(Debug, Clone)]
pub struct Request {
    /// HTTP method.
    pub method: Method,
    /// Request path appended to the client's base URL.
    pub path: String,
    /// Query-string parameters.
    pub query: Vec<(String, String)>,
    /// Request body.
    pub body: Body,
    /// Explicit `Content-Type`, if set (e.g. for a raw body).
    pub content_type: Option<String>,
    /// Human-verification token and its type, if attached.
    pub hv: Option<(String, String)>,
    /// Skip the 401→refresh retry (auth-flow endpoints).
    pub skip_refresh: bool,
    /// Do not attach `Authorization` (refresh endpoint).
    pub omit_auth: bool,
    /// Send `x-enforce-unauthsession: true` (session creation).
    pub enforce_unauth: bool,
}

impl Request {
    /// Build an empty request with the given method and path.
    pub fn new(method: Method, path: impl Into<String>) -> Self {
        Request {
            method,
            path: path.into(),
            query: Vec::new(),
            body: Body::Empty,
            content_type: None,
            hv: None,
            skip_refresh: false,
            omit_auth: false,
            enforce_unauth: false,
        }
    }
    /// A `GET` request to `path`.
    pub fn get(path: impl Into<String>) -> Self {
        Self::new(Method::GET, path)
    }
    /// A `POST` request to `path`.
    pub fn post(path: impl Into<String>) -> Self {
        Self::new(Method::POST, path)
    }
    /// A `PUT` request to `path`.
    pub fn put(path: impl Into<String>) -> Self {
        Self::new(Method::PUT, path)
    }
    /// A `DELETE` request to `path`.
    pub fn delete(path: impl Into<String>) -> Self {
        Self::new(Method::DELETE, path)
    }
    /// Set a JSON request body.
    pub fn json(mut self, v: serde_json::Value) -> Self {
        self.body = Body::Json(v);
        self
    }
    /// Set a raw byte body with an explicit content-type.
    pub fn raw(mut self, bytes: Vec<u8>, content_type: impl Into<String>) -> Self {
        self.body = Body::Bytes(bytes);
        self.content_type = Some(content_type.into());
        self
    }
    /// Append a query-string parameter.
    pub fn query<K: Into<String>, V: Into<String>>(mut self, k: K, v: V) -> Self {
        self.query.push((k.into(), v.into()));
        self
    }
    /// Attach a human-verification token and its type.
    pub fn with_hv(mut self, token: impl Into<String>, ttype: impl Into<String>) -> Self {
        self.hv = Some((token.into(), ttype.into()));
        self
    }
    /// Skip the automatic 401→refresh retry (for auth-flow endpoints).
    pub fn no_refresh(mut self) -> Self {
        self.skip_refresh = true;
        self
    }
    /// Do not send the `Authorization` header (for the refresh endpoint).
    pub fn omit_auth(mut self) -> Self {
        self.omit_auth = true;
        self
    }
    /// Send the `x-enforce-unauthsession` header (for session creation).
    pub fn enforce_unauth(mut self) -> Self {
        self.enforce_unauth = true;
        self
    }
}

/// A transport-level response.
#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP status code.
    pub status: u16,
    /// Raw response body bytes.
    pub body: Vec<u8>,
    /// `Retry-After` delay in seconds, if the server sent one.
    pub retry_after: Option<u64>,
}

/// Mutable auth state shared with the client (so refresh can update it).
#[derive(Debug, Clone)]
pub struct AuthState {
    /// Session UID, once authenticated.
    pub uid: Option<String>,
    /// Current access token, once authenticated.
    pub access: Option<SecretString>,
    /// Current refresh token, once authenticated.
    pub refresh: Option<SecretString>,
    /// App-version string sent on every request.
    pub app_version: String,
    /// Optional `User-Agent` (to present as a real client).
    pub user_agent: Option<String>,
}

impl AuthState {
    /// An empty (logged-out) auth state for the given app version.
    pub fn unauthenticated(app_version: impl Into<String>) -> Self {
        AuthState {
            uid: None,
            access: None,
            refresh: None,
            app_version: app_version.into(),
            user_agent: None,
        }
    }
}

/// Callback invoked after a successful token refresh (uid, access, refresh) —
/// used by the Client to persist rotated tokens.
pub type RefreshPersist = Arc<dyn Fn(&str, &str, &str) + Send + Sync>;
/// Resolver for human-verification challenges (returns token, token-type).
pub type HvResolver =
    Arc<dyn Fn(HvChallenge) -> BoxFuture<'static, Result<(String, String)>> + Send + Sync>;

/// The async transport interface consumed by the API layer.
#[async_trait]
pub trait Doer: Send + Sync {
    /// Execute a request and return the raw response (after envelope/retry handling).
    async fn do_raw(&self, req: Request) -> Result<Response>;
    /// Execute a request and deserialize the response body into `T`.
    async fn decode<T: DeserializeOwned>(&self, req: Request) -> Result<T>;
}

/// Default HTTP client over `reqwest`.
pub struct HttpClient {
    client: reqwest::Client,
    base_url: String,
    auth: Arc<RwLock<AuthState>>,
    on_refresh: Option<RefreshPersist>,
    hv: Option<HvResolver>,
}

impl HttpClient {
    /// Create a client with a fresh unauthenticated state for `base_url`.
    pub fn new(base_url: impl Into<String>, app_version: impl Into<String>) -> Self {
        Self::with_state(
            base_url,
            Arc::new(RwLock::new(AuthState::unauthenticated(app_version))),
        )
    }

    /// Create a client sharing the given auth state (so refresh updates propagate).
    pub fn with_state(base_url: impl Into<String>, auth: Arc<RwLock<AuthState>>) -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .expect("reqwest client");
        HttpClient {
            client,
            base_url: base_url.into(),
            auth,
            on_refresh: None,
            hv: None,
        }
    }

    /// The shared auth-state handle (clone to observe token rotation).
    pub fn auth_state(&self) -> Arc<RwLock<AuthState>> {
        self.auth.clone()
    }
    /// Register a callback invoked with rotated tokens after each refresh.
    pub fn set_refresh_persist(&mut self, cb: RefreshPersist) {
        self.on_refresh = Some(cb);
    }
    /// Register a resolver to satisfy human-verification (9001) challenges.
    pub fn set_hv_resolver(&mut self, cb: HvResolver) {
        self.hv = Some(cb);
    }

    /// Set the active session's UID and access/refresh tokens.
    pub async fn set_tokens(&self, uid: String, access: SecretString, refresh: SecretString) {
        let mut a = self.auth.write().await;
        a.uid = Some(uid);
        a.access = Some(access);
        a.refresh = Some(refresh);
    }

    /// Present a custom `User-Agent` on every request.
    pub async fn set_user_agent(&self, ua: String) {
        self.auth.write().await.user_agent = Some(ua);
    }

    fn url(&self, req: &Request) -> String {
        let mut u = format!("{}{}", self.base_url, req.path);
        if !req.query.is_empty() {
            let qs: Vec<String> = req
                .query
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect();
            u.push('?');
            u.push_str(&qs.join("&"));
        }
        u
    }

    async fn execute_once(&self, req: &Request) -> Result<Response> {
        let auth = self.auth.read().await.clone();
        let mut hdrs = headers::build_headers(&auth, req);
        if req.omit_auth {
            hdrs.remove(reqwest::header::AUTHORIZATION);
        }
        let url = self.url(req);
        let body_kind = match &req.body {
            Body::Empty => "empty",
            Body::Json(_) => "json",
            Body::Bytes(_) => "bytes",
        };
        tracing::debug!(target: "proton_core::http", method = %req.method, %url, body = body_kind, hv = req.hv.is_some(), "→ request");
        tracing::trace!(target: "proton_core::http", headers = ?hdrs.keys().map(|k| k.as_str()).collect::<Vec<_>>(), "request headers");

        let mut builder = self.client.request(req.method.clone(), &url).headers(hdrs);
        builder = match &req.body {
            Body::Empty => builder,
            Body::Json(v) => builder.body(serde_json::to_vec(v)?),
            Body::Bytes(b) => builder.body(b.clone()),
        };
        let started = std::time::Instant::now();
        let resp = builder.send().await?;
        let status = resp.status().as_u16();
        let retry_after = resp
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| retry::parse_retry_after(Some(s)));
        let body = resp.bytes().await?.to_vec();
        tracing::debug!(target: "proton_core::http", status, bytes = body.len(), ms = started.elapsed().as_millis() as u64, "← response");
        Ok(Response {
            status,
            body,
            retry_after,
        })
    }

    /// Refresh the access/refresh tokens via `POST /auth/v4/refresh`.
    async fn refresh_tokens(&self) -> Result<()> {
        let (uid, refresh) = {
            let a = self.auth.read().await;
            match (&a.uid, &a.refresh) {
                (Some(u), Some(r)) => (u.clone(), r.expose_secret().to_string()),
                _ => return Err(Error::Unauthorized),
            }
        };
        let body = serde_json::json!({
            "UID": uid,
            "RefreshToken": refresh,
            "ResponseType": "token",
            "GrantType": "refresh_token",
            "RedirectURI": "https://protonmail.ch",
            "State": "0",
        });
        let req = Request::post("/auth/v4/refresh")
            .json(body)
            .no_refresh()
            .omit_auth();
        let resp = self.execute_once(&req).await?;
        if !(200..300).contains(&resp.status) {
            return Err(Error::Unauthorized);
        }
        #[derive(serde::Deserialize)]
        struct RefreshResp {
            #[serde(rename = "AccessToken")]
            access_token: String,
            #[serde(rename = "RefreshToken")]
            refresh_token: String,
        }
        let parsed: RefreshResp = serde_json::from_slice(&resp.body)?;
        {
            let mut a = self.auth.write().await;
            a.access = Some(SecretString::from(parsed.access_token.clone()));
            a.refresh = Some(SecretString::from(parsed.refresh_token.clone()));
        }
        if let Some(cb) = &self.on_refresh {
            cb(&uid, &parsed.access_token, &parsed.refresh_token);
        }
        Ok(())
    }
}

/// Outcome of inspecting a response for the Proton error envelope.
enum Detected {
    Ok,
    Api(ApiError),
    Hv(HvChallenge),
}

fn detect(status: u16, body: &[u8]) -> Detected {
    let json: Option<serde_json::Value> = serde_json::from_slice(body).ok();
    let code = json
        .as_ref()
        .and_then(|j| j.get("Code"))
        .and_then(|c| c.as_i64());

    // Human verification (may arrive on HTTP 200 in the auth flow, or 422).
    if code == Some(9001) {
        if let Some(details) = json.as_ref().and_then(|j| j.get("Details")) {
            let methods = details
                .get("HumanVerificationMethods")
                .and_then(|m| m.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            return Detected::Hv(HvChallenge {
                token: details
                    .get("HumanVerificationToken")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                methods,
                web_url: details
                    .get("WebUrl")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }
    }

    let success = matches!(code, Some(1000) | Some(1001));
    let http_ok = (200..300).contains(&status);

    if http_ok && (success || code.is_none()) {
        return Detected::Ok;
    }
    if http_ok && code == Some(0) {
        return Detected::Ok;
    }

    let message = json
        .as_ref()
        .and_then(|j| j.get("Error"))
        .and_then(|v| v.as_str())
        .unwrap_or("request failed")
        .to_string();
    Detected::Api(ApiError {
        http_status: status,
        code: code.unwrap_or(0),
        message,
        raw_body: String::from_utf8_lossy(body).into_owned(),
    })
}

#[async_trait]
impl Doer for HttpClient {
    async fn do_raw(&self, req: Request) -> Result<Response> {
        let mut work = req.clone();
        let mut refreshed = false;
        let mut rate_limited = false;
        let mut hv_tried = false;

        loop {
            let resp = self.execute_once(&work).await?;
            match detect(resp.status, &resp.body) {
                Detected::Ok => return Ok(resp),
                Detected::Api(e) if e.http_status == 401 && !work.skip_refresh && !refreshed => {
                    tracing::debug!(target: "proton_core::http", "401 — refreshing access token and retrying");
                    self.refresh_tokens().await?;
                    refreshed = true;
                    continue;
                }
                Detected::Api(e) if e.http_status == 401 => return Err(Error::Unauthorized),
                Detected::Api(e) if e.http_status == 429 && !rate_limited => {
                    let wait = resp.retry_after.unwrap_or(0);
                    tracing::debug!(target: "proton_core::http", wait_s = wait, "429 rate-limited — backing off then retrying");
                    if wait > 0 {
                        tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                    }
                    rate_limited = true;
                    continue;
                }
                Detected::Hv(chal) if self.hv.is_some() && !hv_tried => {
                    tracing::info!(target: "proton_core::auth", methods = ?chal.methods, "9001 human verification required — invoking resolver");
                    let resolver = self.hv.as_ref().unwrap().clone();
                    let (tok, ttype) = resolver(chal).await?;
                    tracing::info!(target: "proton_core::auth", token_type = %ttype, "human verification token obtained — retrying");
                    work.hv = Some((tok, ttype));
                    hv_tried = true;
                    continue;
                }
                Detected::Hv(chal) => return Err(Error::HumanVerification(chal)),
                Detected::Api(e) => return Err(Error::Api(e)),
            }
        }
    }

    async fn decode<T: DeserializeOwned>(&self, req: Request) -> Result<T> {
        let resp = self.do_raw(req).await?;
        Ok(serde_json::from_slice(&resp.body)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client(base: &str) -> HttpClient {
        HttpClient::new(base, "Other")
    }

    #[derive(Deserialize, Debug)]
    struct ValueResp {
        #[serde(rename = "Value")]
        value: i64,
    }

    #[tokio::test]
    async fn decodes_success_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/x"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"Code":1000,"Value":42})),
            )
            .mount(&server)
            .await;
        let c = client(&server.uri());
        let v: ValueResp = c.decode(Request::get("/x")).await.unwrap();
        assert_eq!(v.value, 42);
    }

    #[tokio::test]
    async fn maps_error_code() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/x"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"Code":2501,"Error":"nope"})),
            )
            .mount(&server)
            .await;
        let c = client(&server.uri());
        let err = c.decode::<ValueResp>(Request::get("/x")).await.unwrap_err();
        match err {
            Error::Api(e) => {
                assert_eq!(e.code, 2501);
                assert_eq!(e.message, "nope");
            }
            other => panic!("expected Api, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn retries_once_on_429() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/x"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/x"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"Code":1000,"Value":7})),
            )
            .mount(&server)
            .await;
        let c = client(&server.uri());
        let v: ValueResp = c.decode(Request::get("/x")).await.unwrap();
        assert_eq!(v.value, 7);
    }

    #[tokio::test]
    async fn refreshes_then_retries_on_401() {
        let server = MockServer::start().await;
        // First protected call → 401.
        Mock::given(method("GET"))
            .and(path("/protected"))
            .respond_with(
                ResponseTemplate::new(401)
                    .set_body_json(serde_json::json!({"Code":401,"Error":"expired"})),
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;
        // Refresh endpoint → new tokens.
        Mock::given(method("POST"))
            .and(path("/auth/v4/refresh"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"Code":1000,"AccessToken":"new-acc","RefreshToken":"new-ref"}),
            ))
            .mount(&server)
            .await;
        // Retried protected call → success.
        Mock::given(method("GET"))
            .and(path("/protected"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"Code":1000,"Value":99})),
            )
            .mount(&server)
            .await;

        let c = client(&server.uri());
        c.set_tokens(
            "uid".into(),
            SecretString::from("old-acc"),
            SecretString::from("old-ref"),
        )
        .await;
        let v: ValueResp = c.decode(Request::get("/protected")).await.unwrap();
        assert_eq!(v.value, 99);
        // Tokens were rotated.
        let a = c.auth.read().await;
        assert_eq!(a.access.as_ref().unwrap().expose_secret(), "new-acc");
    }

    #[tokio::test]
    async fn unauthorized_when_refresh_fails() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/protected"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({"Code":401})))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/auth/v4/refresh"))
            .respond_with(
                ResponseTemplate::new(422).set_body_json(serde_json::json!({"Code":10013})),
            )
            .mount(&server)
            .await;
        let c = client(&server.uri());
        c.set_tokens(
            "uid".into(),
            SecretString::from("a"),
            SecretString::from("r"),
        )
        .await;
        let err = c
            .decode::<ValueResp>(Request::get("/protected"))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Unauthorized));
    }

    #[tokio::test]
    async fn resolves_human_verification_then_retries_with_headers() {
        let server = MockServer::start().await;
        // Retry that carries the HV headers → success (higher priority).
        Mock::given(method("GET"))
            .and(path("/x"))
            .and(wiremock::matchers::header(
                "x-pm-human-verification-token",
                "chal:resp",
            ))
            .and(wiremock::matchers::header(
                "x-pm-human-verification-token-type",
                "captcha",
            ))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"Code":1000,"Value":77})),
            )
            .with_priority(1)
            .mount(&server)
            .await;
        // First (no HV header) → 9001 challenge.
        Mock::given(method("GET"))
            .and(path("/x"))
            .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
                "Code": 9001,
                "Error": "human verification required",
                "Details": {
                    "HumanVerificationToken": "chal",
                    "HumanVerificationMethods": ["captcha"],
                    "WebUrl": "https://verify.proton.me/?methods=captcha&token=chal"
                }
            })))
            .mount(&server)
            .await;

        let mut c = client(&server.uri());
        let seen: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
        let seen2 = seen.clone();
        c.set_hv_resolver(Arc::new(move |chal: HvChallenge| {
            let seen2 = seen2.clone();
            Box::pin(async move {
                seen2.write().await.push(chal.token.clone());
                // value format = "<challenge>:<captcha-response>"
                Ok((format!("{}:resp", chal.token), "captcha".to_string()))
            })
        }));

        let v: ValueResp = c.decode(Request::get("/x")).await.unwrap();
        assert_eq!(v.value, 77);
        // resolver was invoked with the challenge token from Details.
        assert_eq!(seen.read().await.as_slice(), &["chal".to_string()]);
    }
}
