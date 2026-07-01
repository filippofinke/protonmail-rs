//! Free helpers shared across the tool modules and the server handler.
//!
//! These are pure (or near-pure) and unit-tested directly.

use rmcp::handler::server::wrapper::Json;
use rmcp::ErrorData;
use serde_json::{json, Value};

use proton_core::Verdict;

/// Structured tool output. A JSON object root is required by the MCP output
/// schema, so we return a `Map` (which schemars renders as `type: object`)
/// rather than a bare `Value` (whose schema has no root type).
pub(crate) type Out = Json<serde_json::Map<String, Value>>;

/// Whether a confirm-gated, destructive action should actually be performed.
///
/// It runs when the server allows writes globally, or the call explicitly
/// passes `confirm: true`. Otherwise the tool returns a dry-run preview.
pub(crate) fn should_perform(allow_writes: bool, confirm: Option<bool>) -> bool {
    allow_writes || confirm == Some(true)
}

/// Truncate `s` to at most `max` characters, appending an ellipsis if cut.
pub(crate) fn excerpt(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let head: String = s.chars().take(max).collect();
        format!("{head}…")
    } else {
        s.to_string()
    }
}

/// Build the standard "not authenticated" error.
pub(crate) fn not_authenticated(profile: &str) -> ErrorData {
    ErrorData::invalid_request(
        format!("Not authenticated — run `protonmail-cli login` (profile {profile})"),
        None,
    )
}

/// Wrap a JSON value as structured tool output. Object values pass through;
/// any other value is nested under a `result` key.
pub(crate) fn obj(v: Value) -> Out {
    match v {
        Value::Object(m) => Json(m),
        other => {
            let mut m = serde_json::Map::new();
            m.insert("result".to_string(), other);
            Json(m)
        }
    }
}

/// Standard dry-run preview payload for a confirm-gated action.
pub(crate) fn dry_run(action: &str, preview: Value) -> Out {
    obj(json!({
        "dry_run": true,
        "action": action,
        "preview": preview,
        "note": "Action NOT performed. Re-run with `confirm: true` (or start proton-mcp with --allow-writes).",
    }))
}

/// Serialize any `Serialize` value to JSON, mapping failures to an MCP error.
pub(crate) fn to_value<T: serde::Serialize>(v: &T) -> Result<Value, ErrorData> {
    serde_json::to_value(v).map_err(|e| ErrorData::internal_error(e.to_string(), None))
}

/// Human-readable warning for a non-verified signature verdict.
pub(crate) fn signature_warning(verdict: Verdict) -> Option<&'static str> {
    match verdict {
        Verdict::Verified => None,
        Verdict::Unsigned => Some("Message body is not signed."),
        Verdict::Unverified => {
            Some("Signature could not be verified (sender public key unavailable).")
        }
        Verdict::Invalid => Some("Signature is INVALID — the message may have been tampered with."),
    }
}
