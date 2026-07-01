//! HTML sanitization for message bodies (XSS-safe rendering).

/// Sanitize an HTML message body: strips `<script>`/`<style>`, event handlers,
/// `javascript:` URLs, forms, and other active content, keeping safe formatting.
///
/// To block tracking pixels / remote images, rewrites `<img src>` and CSS
/// backgrounds are dropped by removing `src` on remote images is left to the
/// caller; here we keep images but neutralize active content. (A remote-image
/// proxy is a separate, deferred feature.)
pub fn sanitize(html: &str) -> String {
    ammonia::clean(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_active_content_keeps_formatting() {
        let dirty = r#"<b>hi</b><script>alert(1)</script><a href="javascript:evil()">x</a><img src="x" onerror="evil()"><p onclick="bad()">t</p>"#;
        let clean = sanitize(dirty);
        assert!(clean.contains("<b>hi</b>"));
        assert!(!clean.contains("<script"));
        assert!(!clean.to_lowercase().contains("javascript:"));
        assert!(!clean.to_lowercase().contains("onerror"));
        assert!(!clean.to_lowercase().contains("onclick"));
    }

    #[test]
    fn keeps_safe_links() {
        let clean = sanitize(r#"<a href="https://proton.me">link</a>"#);
        assert!(clean.contains("https://proton.me"));
    }
}
