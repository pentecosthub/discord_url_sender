use super::{dates, models::*, trim};
use regex_lite::Regex;
use std::sync::LazyLock;

static URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://\S+").unwrap());

/// Every message in the channel is expected to be a link; messages without one are ignored.
pub fn extract_url(input: &str) -> Option<String> {
    URL.find(trim(input)).map(|m| m.as_str().to_owned())
}

pub fn processed(markdown: String, message: DiscordMessage, zone: &str) -> Result<ProcessedMessage, String> {
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
            message.id
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
}
