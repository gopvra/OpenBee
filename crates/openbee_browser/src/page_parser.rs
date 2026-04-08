//! Simple HTML-to-text extraction without external dependencies.
//!
//! This module provides basic HTML parsing utilities for extracting visible
//! text, elements by class or tag name, and code blocks. It does **not** use
//! a full HTML parser; instead it operates on simple pattern matching which
//! is sufficient for well-formed AI platform output.

/// Stateless HTML text extraction utilities.
pub struct PageParser;

impl PageParser {
    /// Strip HTML tags and return only the visible text content.
    ///
    /// This is a simplified extractor that:
    /// - Removes everything between `<` and `>`
    /// - Collapses whitespace runs into single spaces
    /// - Decodes basic HTML entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#39;`)
    pub fn html_to_text(html: &str) -> String {
        let mut result = String::with_capacity(html.len());
        let mut in_tag = false;
        let mut in_script = false;
        let mut in_style = false;

        let lower = html.to_lowercase();
        let chars: Vec<char> = html.chars().collect();
        let lower_chars: Vec<char> = lower.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            if !in_tag && chars[i] == '<' {
                in_tag = true;
                // Check for <script> or <style>
                let remaining: String = lower_chars[i..].iter().take(10).collect();
                if remaining.starts_with("<script") {
                    in_script = true;
                } else if remaining.starts_with("<style") {
                    in_style = true;
                } else if remaining.starts_with("</script") {
                    in_script = false;
                } else if remaining.starts_with("</style") {
                    in_style = false;
                }
                i += 1;
                continue;
            }

            if in_tag {
                if chars[i] == '>' {
                    in_tag = false;
                    // Insert a space boundary between tags so that
                    // "<p>a</p><p>b</p>" becomes "a b" rather than "ab".
                    if !in_script && !in_style {
                        result.push(' ');
                    }
                }
                i += 1;
                continue;
            }

            if in_script || in_style {
                i += 1;
                continue;
            }

            // Decode HTML entities
            if chars[i] == '&' {
                let rest: String = chars[i..].iter().take(10).collect();
                if rest.starts_with("&amp;") {
                    result.push('&');
                    i += 5;
                } else if rest.starts_with("&lt;") {
                    result.push('<');
                    i += 4;
                } else if rest.starts_with("&gt;") {
                    result.push('>');
                    i += 4;
                } else if rest.starts_with("&quot;") {
                    result.push('"');
                    i += 6;
                } else if rest.starts_with("&#39;") {
                    result.push('\'');
                    i += 5;
                } else if rest.starts_with("&nbsp;") {
                    result.push(' ');
                    i += 6;
                } else {
                    result.push('&');
                    i += 1;
                }
                continue;
            }

            result.push(chars[i]);
            i += 1;
        }

        // Collapse whitespace
        collapse_whitespace(&result)
    }

    /// Extract the text content of all elements whose `class` attribute
    /// contains `class_name`.
    ///
    /// This performs a simple substring search for `class="...class_name..."`.
    pub fn extract_by_class(html: &str, class_name: &str) -> Vec<String> {
        let mut results = Vec::new();
        let search = format!("class=\"{}", class_name);

        let mut pos = 0;
        while let Some(class_start) = html[pos..].find(&search) {
            let abs_start = pos + class_start;

            // Walk backwards to find the opening `<`
            let tag_start = match html[..abs_start].rfind('<') {
                Some(s) => s,
                None => {
                    pos = abs_start + search.len();
                    continue;
                }
            };

            // Find the closing `>` of this opening tag
            let tag_end = match html[abs_start..].find('>') {
                Some(e) => abs_start + e + 1,
                None => {
                    pos = abs_start + search.len();
                    continue;
                }
            };

            // Extract the tag name to find the matching closing tag
            let tag_content = &html[tag_start + 1..abs_start];
            let tag_name = tag_content
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_lowercase();

            if tag_name.is_empty() {
                pos = tag_end;
                continue;
            }

            let close_tag = format!("</{}>", tag_name);
            if let Some(close_pos) = html[tag_end..].find(&close_tag) {
                let inner = &html[tag_end..tag_end + close_pos];
                results.push(Self::html_to_text(inner));
            }

            pos = tag_end;
        }

        results
    }

    /// Extract the text content of all elements with the given tag name.
    pub fn extract_by_tag(html: &str, tag: &str) -> Vec<String> {
        let mut results = Vec::new();
        let open = format!("<{}", tag.to_lowercase());
        let close = format!("</{}>", tag.to_lowercase());
        let lower = html.to_lowercase();

        let mut pos = 0;
        while let Some(start) = lower[pos..].find(&open) {
            let abs_start = pos + start;

            // Find end of opening tag
            let tag_end = match lower[abs_start..].find('>') {
                Some(e) => abs_start + e + 1,
                None => break,
            };

            // Find closing tag
            if let Some(end) = lower[tag_end..].find(&close) {
                let inner = &html[tag_end..tag_end + end];
                results.push(Self::html_to_text(inner));
            }

            pos = tag_end;
        }

        results
    }

    /// Extract the content of all `<code>` or `<pre>` blocks.
    pub fn extract_code_blocks(html: &str) -> Vec<String> {
        let mut blocks = Vec::new();
        blocks.extend(Self::extract_by_tag(html, "code"));
        blocks.extend(Self::extract_by_tag(html, "pre"));
        blocks
    }

    /// Return the text of the *last* element whose opening tag contains
    /// `pattern` as a substring. Useful for grabbing the most recent AI
    /// response on a page.
    pub fn last_matching(html: &str, pattern: &str) -> Option<String> {
        let lower = html.to_lowercase();
        let pattern_lower = pattern.to_lowercase();

        let mut last_result: Option<String> = None;
        let mut pos = 0;

        while let Some(idx) = lower[pos..].find(&pattern_lower) {
            let abs = pos + idx;

            // Find opening `<` before pattern
            let tag_start = match lower[..abs].rfind('<') {
                Some(s) => s,
                None => {
                    pos = abs + pattern.len();
                    continue;
                }
            };

            // Find `>` after pattern
            let tag_end = match lower[abs..].find('>') {
                Some(e) => abs + e + 1,
                None => break,
            };

            // Extract tag name
            let tag_content = &lower[tag_start + 1..abs];
            let tag_name = tag_content.split_whitespace().next().unwrap_or("");
            if tag_name.is_empty() {
                pos = tag_end;
                continue;
            }

            let close = format!("</{}>", tag_name);
            if let Some(close_pos) = lower[tag_end..].find(&close) {
                let inner = &html[tag_end..tag_end + close_pos];
                last_result = Some(Self::html_to_text(inner));
            }

            pos = tag_end;
        }

        last_result
    }
}

/// Collapse runs of whitespace (spaces, tabs, newlines) into single spaces
/// and trim leading/trailing whitespace.
fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_ws = true; // treat start as whitespace to trim leading

    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_ws {
                result.push(' ');
            }
            prev_ws = true;
        } else {
            result.push(ch);
            prev_ws = false;
        }
    }

    // Trim trailing space
    if result.ends_with(' ') {
        result.pop();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_text_basic() {
        let html = "<p>Hello <b>world</b></p>";
        assert_eq!(PageParser::html_to_text(html), "Hello world");
    }

    #[test]
    fn test_html_to_text_entities() {
        let html = "5 &gt; 3 &amp; 2 &lt; 4";
        assert_eq!(PageParser::html_to_text(html), "5 > 3 & 2 < 4");
    }

    #[test]
    fn test_html_to_text_strips_script() {
        let html = "<p>before</p><script>alert('xss')</script><p>after</p>";
        assert_eq!(PageParser::html_to_text(html), "before after");
    }

    #[test]
    fn test_extract_by_tag() {
        let html = "<div><p>first</p><p>second</p></div>";
        let ps = PageParser::extract_by_tag(html, "p");
        assert_eq!(ps, vec!["first", "second"]);
    }

    #[test]
    fn test_extract_code_blocks() {
        let html = "<p>text</p><code>let x = 1;</code><pre>fn main() {}</pre>";
        let blocks = PageParser::extract_code_blocks(html);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0], "let x = 1;");
        assert_eq!(blocks[1], "fn main() {}");
    }

    #[test]
    fn test_extract_by_class() {
        let html = r#"<div class="response">Hello AI</div><div class="other">Bye</div>"#;
        let results = PageParser::extract_by_class(html, "response");
        assert_eq!(results, vec!["Hello AI"]);
    }

    #[test]
    fn test_last_matching() {
        let html = r#"<div class="msg">first</div><div class="msg">second</div>"#;
        let last = PageParser::last_matching(html, "msg");
        assert_eq!(last, Some("second".to_string()));
    }
}
