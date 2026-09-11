use super::{dates, models::*, sanitize_path_segment, trim, truncate_path_segment};
use regex_lite::Regex;
use std::sync::LazyLock;

static URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://\S+").unwrap());
const MAX_TITLE_LENGTH: usize = 100;

/// Every message in the channel is expected to be a link; messages without one are ignored.
pub fn extract_url(input: &str) -> Option<String> {
    URL.find(trim(input)).map(|m| m.as_str().to_owned())
}

/// Falls back to the message id when there is no usable article title, so
/// every clipping still gets a unique, non-empty file name.
fn file_name_suffix(title: Option<&str>, message_id: &str) -> String {
    let slug = title
        .map(sanitize_path_segment)
        .map(|s| truncate_path_segment(&s, MAX_TITLE_LENGTH))
        .filter(|s| !s.is_empty());
    slug.unwrap_or_else(|| message_id.to_owned())
}

pub fn processed(
    markdown: String,
    title: Option<&str>,
    message: DiscordMessage,
    zone: &str,
) -> Result<ProcessedMessage, String> {
    let author = message.author.as_ref();
    let name = [
        message.member.as_ref().and_then(|m| m.nick.as_deref()),
        author.and_then(|a| a.global_name.as_deref()),
        author.and_then(|a| a.username.as_deref()),
    ]
    .into_iter()
    .flatten()
    .map(trim)
    .find(|s| !s.is_empty())
    .or_else(|| {
        author
            .and_then(|a| a.id.as_deref())
            .filter(|s| !s.is_empty())
    })
    .unwrap_or("Unknown")
    .to_string();
    Ok(ProcessedMessage {
        file_name: format!(
            "{}_{}",
            dates::local(&message.timestamp, zone)?.file_timestamp,
            file_name_suffix(title, &message.id)
        ),
        author_id: author.and_then(|a| a.id.clone()).unwrap_or_default(),
        author_name: name,
        message_id: message.id,
        timestamp: message.timestamp,
        markdown,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_the_first_url_and_ignores_messages_without_one() {
        assert_eq!(
            extract_url("  https://example.com  "),
            Some("https://example.com".to_owned())
        );
        assert_eq!(
            extract_url("check this out https://example.com/path?q=1 thanks"),
            Some("https://example.com/path?q=1".to_owned())
        );
        assert_eq!(extract_url("hello world"), None);
        assert_eq!(extract_url(""), None);
    }

    fn message(id: &str) -> DiscordMessage {
        DiscordMessage {
            id: id.into(),
            content: "https://example.com".into(),
            timestamp: "2026-06-30T15:30:00Z".into(),
            author: None,
            member: None,
        }
    }

    #[test]
    fn file_name_uses_the_sanitized_title_when_one_is_available() {
        let processed_message = processed(
            "content".into(),
            Some("  A: Title / With * Illegal? Chars  "),
            message("123"),
            "UTC",
        )
        .unwrap();

        assert_eq!(
            processed_message.file_name,
            "20260630_153000_A- Title - With - Illegal- Chars"
        );
    }

    #[test]
    fn file_name_falls_back_to_the_message_id_without_a_usable_title() {
        assert_eq!(
            processed("content".into(), None, message("123"), "UTC")
                .unwrap()
                .file_name,
            "20260630_153000_123"
        );
        assert_eq!(
            processed("content".into(), Some("   "), message("124"), "UTC")
                .unwrap()
                .file_name,
            "20260630_153000_124"
        );
    }

    #[test]
    fn file_name_truncates_long_titles_without_a_dangling_separator() {
        let title = "a".repeat(105);
        let processed_message =
            processed("content".into(), Some(&title), message("123"), "UTC").unwrap();

        assert_eq!(
            processed_message.file_name,
            format!("20260630_153000_{}", "a".repeat(100))
        );
    }
}
