use std::sync::LazyLock;

use regex::Regex;

static NUMERIC_ENTITY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"&#(x[0-9a-fA-F]+|\d+);").unwrap());

/// Strip HTML tags and decode basic entities, converting block-level breaks to newlines.
pub fn strip_html(html: &str) -> String {
    // Replace block-level closing tags with newlines
    let s = html
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n")
        .replace("</li>", "\n");

    // Strip remaining HTML tags
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }

    // Decode common HTML entities
    let result = result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");

    // Decode numeric HTML entities (decimal and hex)
    NUMERIC_ENTITY_RE.replace_all(&result, |caps: &regex::Captures| {
        let inner = &caps[1];
        let code_point = if let Some(hex) = inner.strip_prefix('x') {
            u32::from_str_radix(hex, 16).ok()
        } else {
            inner.parse::<u32>().ok()
        };
        match code_point.and_then(char::from_u32) {
            Some(ch) => ch.to_string(),
            None => caps[0].to_string(),
        }
    }).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_removes_tags() {
        assert_eq!(strip_html("<b>bold</b>"), "bold");
    }

    #[test]
    fn strip_html_br_to_newline() {
        assert_eq!(strip_html("a<br>b"), "a\nb");
        assert_eq!(strip_html("a<br/>b"), "a\nb");
        assert_eq!(strip_html("a<br />b"), "a\nb");
    }

    #[test]
    fn strip_html_decodes_entities() {
        assert_eq!(strip_html("&amp; &lt; &gt;"), "& < >");
        assert_eq!(strip_html("&quot;hi&#39;"), "\"hi'");
    }

    #[test]
    fn strip_html_decodes_decimal_numeric_entities() {
        assert_eq!(strip_html("&#65;&#66;&#67;"), "ABC");
    }

    #[test]
    fn strip_html_decodes_hex_numeric_entities() {
        assert_eq!(strip_html("&#x41;&#x42;&#x43;"), "ABC");
    }

    #[test]
    fn strip_html_decodes_mixed_numeric_entities() {
        // copyright sign (©) = &#169; and heart (♥) = &#x2665;
        assert_eq!(strip_html("&#169; &#x2665;"), "\u{00A9} \u{2665}");
    }

    #[test]
    fn strip_html_preserves_invalid_numeric_entity() {
        // 0xFFFFFF is not a valid Unicode code point
        assert_eq!(strip_html("&#xFFFFFF;"), "&#xFFFFFF;");
    }
}
