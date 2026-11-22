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

/// Convert an HTML body to readable plain text (for quoting an HTML message in
/// a plain-text reply). Block elements become line breaks, `<style>`/`<script>`
/// /`<head>` content is dropped, links keep their target, and entities are
/// decoded. Not a full renderer — just enough to avoid quoting raw markup.
pub fn to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(lt) = rest.find('<') {
        out.push_str(&decode_entities(&rest[..lt]));
        rest = &rest[lt..];
        // Comments.
        if rest.starts_with("<!--") {
            rest = rest.find("-->").map_or("", |e| &rest[e + 3..]);
            continue;
        }
        let Some(gt) = rest.find('>') else {
            // Unterminated tag: keep the remainder as text.
            out.push_str(&decode_entities(rest));
            rest = "";
            break;
        };
        let tag = &rest[1..gt];
        rest = &rest[gt + 1..];
        let closing = tag.starts_with('/');
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        match name.as_str() {
            "style" | "script" | "head" | "title" if !closing => {
                let end = format!("</{name}");
                let lower = rest.to_ascii_lowercase();
                rest = match lower.find(&end) {
                    Some(i) => rest[i..].find('>').map_or("", |g| &rest[i + g + 1..]),
                    None => "",
                };
            }
            "br" => out.push('\n'),
            "p" | "div" | "tr" | "table" | "ul" | "ol" | "blockquote" | "pre" | "h1" | "h2"
            | "h3" | "h4" | "h5" | "h6" | "hr" => {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
            }
            "li" if !closing => {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("- ");
            }
            "td" | "th" if closing => out.push('\t'),
            "a" if !closing => {
                if let Some(href) = attr(tag, "href") {
                    if href.starts_with("http") {
                        // Emit the target after the link text, once the tag closes.
                        let lower = rest.to_ascii_lowercase();
                        if let Some(i) = lower.find("</a") {
                            let text = to_text(&rest[..i]);
                            let text = text.trim();
                            out.push_str(text);
                            if text != href {
                                out.push_str(&format!(" <{href}>"));
                            }
                            rest = rest[i..].find('>').map_or("", |g| &rest[i + g + 1..]);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out.push_str(&decode_entities(rest));
    // Tidy whitespace: trim line ends, collapse runs of blank lines.
    let mut tidy = String::with_capacity(out.len());
    let mut blank = 0;
    for line in out.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
            tidy.push('\n');
        } else {
            blank = 0;
            tidy.push_str(line);
            tidy.push('\n');
        }
    }
    tidy.trim().to_string()
}

/// Extract a (quoted or bare) attribute value from a tag's inner text.
fn attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let i = lower.find(&format!("{name}="))? + name.len() + 1;
    let v = &tag[i..];
    let v = match v.chars().next()? {
        q @ ('"' | '\'') => v[1..].split(q).next()?,
        _ => v.split(|c: char| c.is_whitespace()).next()?,
    };
    Some(decode_entities(v))
}

/// Decode the common named entities plus numeric (`&#NN;` / `&#xHH;`) ones.
fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let decoded = rest.find(';').filter(|&e| e <= 10).and_then(|e| {
            let ent = &rest[1..e];
            let ch = match ent {
                "nbsp" => Some(' '),
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                _ => ent
                    .strip_prefix("#x")
                    .or_else(|| ent.strip_prefix("#X"))
                    .and_then(|h| u32::from_str_radix(h, 16).ok())
                    .or_else(|| ent.strip_prefix('#').and_then(|d| d.parse().ok()))
                    .and_then(char::from_u32),
            }?;
            Some((ch, e + 1))
        });
        match decoded {
            Some((ch, len)) => {
                out.push(ch);
                rest = &rest[len..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Escape text for safe inclusion in HTML (used when quoting a plain-text
/// message in an HTML reply).
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_text_strips_markup_and_decodes_entities() {
        let html = r#"<html><head><style>p{color:red}</style></head><body>
            <div>Hello&nbsp;Bob,</div><div><br></div>
            <p>Price &lt; 5 &amp; &#8364;3</p>
            <ul><li>one</li><li>two</li></ul>
            <a href="https://proton.me">site</a><!-- c --></body></html>"#;
        let t = to_text(html);
        assert!(!t.contains("&nbsp;"));
        assert!(!t.contains("color:red"));
        assert!(t.starts_with("Hello Bob,"), "{t}");
        assert!(t.contains("Price < 5 & €3"), "{t}");
        assert!(t.contains("- one\n- two"), "{t}");
        assert!(t.contains("site <https://proton.me>"), "{t}");
    }

    #[test]
    fn escape_html() {
        assert_eq!(escape("a<b>&\"c"), "a&lt;b&gt;&amp;&quot;c");
    }

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
