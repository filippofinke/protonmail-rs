//! Error types for proton-core.

use std::fmt;

/// A human-verification challenge returned by the API (code 9001).
#[derive(Debug, Clone)]
pub struct HvChallenge {
    /// Opaque verification token to echo back once the challenge is solved.
    pub token: String,
    /// Supported verification methods (e.g. `captcha`, `email`, `sms`).
    pub methods: Vec<String>,
    /// URL where the user completes the challenge in a browser.
    pub web_url: String,
}

/// A structured Proton API error (non-success `Code`, or a non-2xx HTTP status).
#[derive(Debug, Clone)]
pub struct ApiError {
    /// HTTP status code of the response.
    pub http_status: u16,
    /// Proton API `Code` from the response body.
    pub code: i64,
    /// Human-readable error message from the API.
    pub message: String,
    /// Raw response body, retained for diagnostics.
    pub raw_body: String,
}

impl ApiError {
    /// Map to a CLI process exit code (mirrors the Go reference).
    pub fn exit_code(&self) -> i32 {
        match self.http_status {
            401 | 403 => 2,
            404 => 3,
            409 | 422 => 4,
            s if s >= 500 => 5,
            _ => 1,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[HTTP {}] {}: {}",
            self.http_status, self.code, self.message
        )
    }
}

impl std::error::Error for ApiError {}

/// The crate-wide error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A structured Proton API error (non-success code or non-2xx status).
    #[error(transparent)]
    Api(#[from] ApiError),

    /// The session is expired or invalid.
    #[error("unauthorized (session expired or invalid)")]
    Unauthorized,

    /// The API demands human verification before proceeding.
    #[error("human verification required")]
    HumanVerification(HvChallenge),

    /// A transport-level HTTP failure.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON (de)serialization failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// An OpenPGP encryption, decryption, or key operation failed.
    #[error("crypto error: {0}")]
    Crypto(String),

    /// An SRP authentication step failed.
    #[error("srp error: {0}")]
    Srp(String),

    /// A session load, save, or token-refresh operation failed.
    #[error("session error: {0}")]
    Session(String),

    /// A local cache read or write failed.
    #[error("cache error: {0}")]
    Cache(String),

    /// A filesystem or other I/O operation failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A referenced entity of the given kind was not found.
    #[error("{kind} not found")]
    NotFound {
        /// The kind of entity that was not found (e.g. `message`).
        kind: String,
    },

    /// A reference matched more than one entity (count of matches).
    #[error("ambiguous reference: {0} matches")]
    Ambiguous(usize),

    /// Any other error, carrying a message.
    #[error("{0}")]
    Other(String),
}

/// Convenience exit-code mapping for any error (defaults to 1 for non-API).
impl Error {
    /// Map this error to a CLI process exit code (defaults to 1 for non-API errors).
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Api(e) => e.exit_code(),
            Error::Unauthorized => 2,
            Error::NotFound { .. } => 3,
            Error::Ambiguous(_) => 4,
            _ => 1,
        }
    }
}

/// Crate-wide `Result` alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    fn api(status: u16) -> ApiError {
        ApiError {
            http_status: status,
            code: 0,
            message: "x".into(),
            raw_body: String::new(),
        }
    }

    #[test]
    fn exit_codes_map_status() {
        assert_eq!(api(401).exit_code(), 2);
        assert_eq!(api(403).exit_code(), 2);
        assert_eq!(api(404).exit_code(), 3);
        assert_eq!(api(409).exit_code(), 4);
        assert_eq!(api(422).exit_code(), 4);
        assert_eq!(api(500).exit_code(), 5);
        assert_eq!(api(503).exit_code(), 5);
        assert_eq!(api(400).exit_code(), 1);
    }

    #[test]
    fn error_exit_code_delegates() {
        assert_eq!(Error::Api(api(404)).exit_code(), 3);
        assert_eq!(Error::Unauthorized.exit_code(), 2);
        assert_eq!(
            Error::NotFound {
                kind: "message".into()
            }
            .exit_code(),
            3
        );
        assert_eq!(Error::Ambiguous(3).exit_code(), 4);
        assert_eq!(Error::Other("x".into()).exit_code(), 1);
    }
}
