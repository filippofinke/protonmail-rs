//! Small helpers for retry/rate-limit handling.

/// Max seconds we will wait on a 429 before giving up.
pub const MAX_RATE_LIMIT_WAIT: u64 = 30;

/// Parse a `Retry-After` header value (integer seconds only), capped.
pub fn parse_retry_after(value: Option<&str>) -> Option<u64> {
    let secs: u64 = value?.trim().parse().ok()?;
    Some(secs.min(MAX_RATE_LIMIT_WAIT))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_caps() {
        assert_eq!(parse_retry_after(Some("5")), Some(5));
        assert_eq!(parse_retry_after(Some("999")), Some(30));
        assert_eq!(parse_retry_after(Some("0")), Some(0));
        assert_eq!(parse_retry_after(Some("abc")), None);
        assert_eq!(parse_retry_after(None), None);
    }
}
