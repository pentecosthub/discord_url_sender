//! Obsidian-independent domain logic. JavaScript owns host I/O and object identity.
pub mod channels;
pub mod dates;
pub mod discord;
pub mod messages;
pub mod models;
pub mod settings;
pub mod storage;
pub mod sync;

/// ECMAScript trim/whitespace, kept explicit for existing settings compatibility.
pub fn js_whitespace(c: char) -> bool {
    matches!(c, '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}
pub fn trim(value: &str) -> &str {
    value.trim_matches(js_whitespace)
}

/// Collapse forbidden filesystem characters and whitespace runs so arbitrary
/// text (a channel name, an article title) is safe to use as a path segment.
pub fn sanitize_path_segment(value: &str) -> String {
    let mut result = String::new();
    let mut unsafe_run = false;
    let mut space_run = false;
    for c in trim(value).chars() {
        let unsafe_char = "\\/:*?\"<>|#^".contains(c);
        if unsafe_char {
            if !unsafe_run {
                result.push('-');
            }
        } else if c == '[' || c == ']' {
            result.push('-');
        } else if js_whitespace(c) {
            if !space_run {
                result.push(' ');
            }
        } else {
            result.push(c);
        }
        unsafe_run = unsafe_char;
        space_run = js_whitespace(c);
    }
    result.trim_matches('-').into()
}

/// Cap a sanitized path segment at a character count, dropping any dangling
/// separator the cut leaves behind.
pub fn truncate_path_segment(value: &str, max_chars: usize) -> String {
    value
        .chars()
        .take(max_chars)
        .collect::<String>()
        .trim_end_matches(['-', ' '])
        .to_string()
}
