pub mod dom;
pub mod error;
mod frontmatters;
mod parser;
mod renderers;
mod utils;

use error::ConvertError;
use frontmatters::{FrontMatter, get_frontmatter_extractors, serialize_yaml_string};

/// Convert HTML to Markdown with front-matter extraction
///
/// # Arguments
///
/// * 'url'  - The URL of the HTML content (used for context, e.g., links).
/// * `html` - The HTML content to convert.
/// * `keys` - The keys for front-matter extraction (e.g., "title", "tags", "date").
///
/// # Returns
///
/// * `Result<String, ConvertError>` - The converted Markdown content with front-matter (YAML format), or an error.
///
/// # Example
///
/// ```rust
/// let url = "https://example.com";
/// let html = "<h1>Title</h1><p>Content</p>";
/// let keys = ["title"];
/// let markdown = html_to_markdown::convert(url, html, &keys);
/// assert!(markdown.is_ok());
/// assert!(markdown.unwrap().contains("---\ntitle: \"Title\"\n---\n\n# Title\n\nContent"));
/// ```
///
/// if you don't need front-matter, you can pass an empty slice for `keys`.
///
/// ```rust
/// let url = "https://example.com";
/// let html = "<h1>Title</h1><p>Content</p>";
/// let keys: Vec<&str> = vec![];
/// let markdown = html_to_markdown::convert(url, html, &keys);
/// assert!(markdown.is_ok());
/// assert!(markdown.unwrap().contains("# Title\n\nContent"));
/// ```
///
pub fn convert(url: &str, html: &str, keys: &[&str]) -> Result<String, ConvertError> {
    // If you want to fetch HTML content from a URL,
    // you can use an HTTP client library like `reqwest` here.
    // (Obsidian need this API, `requestUrl`, so html content is passed directly)
    // example:
    // let client = reqwest::Client::new();
    // let html = client.request(reqwest::Method::GET, url)?
    //                  .send()
    //                  .await?
    //                  .error_for_status()?;

    // parse HTML
    let dom = parser::parse_html(html)?;

    // front-matter
    let extractors = get_frontmatter_extractors(keys);
    let frontmatter_entries: Vec<String> = extractors
        .into_iter()
        .filter_map(|(key, extractor)| {
            extractor
                .extract(url, &dom)
                .map(|val| format!("{key}: {}", serialize_yaml_string(&val)))
        })
        .collect();

    let mut markdown = String::new();
    if !frontmatter_entries.is_empty() {
        markdown.push_str("---\n");
        for entry in frontmatter_entries {
            markdown.push_str(&entry);
            markdown.push('\n');
        }
        markdown.push_str("---\n\n");
    }

    // render body
    let mut ctx = renderers::Context::default();
    let start_id = dom
        .find_article()
        .or_else(|| dom.find_body())
        .unwrap_or(dom.document);
    let body = renderers::render_node(url, &dom, start_id, &mut ctx)?;
    markdown.push_str(&body);
    Ok(markdown)
}

/// Extract just the page title, without rendering the rest of the document.
///
/// Reuses the same extractor `convert` uses for the `title` front-matter key
/// (`<title>`, `<meta name="title">`, OGP/Twitter title, then first heading).
pub fn extract_title(url: &str, html: &str) -> Result<Option<String>, ConvertError> {
    let dom = parser::parse_html(html)?;
    Ok(frontmatters::title::EXTRACTOR.extract(url, &dom))
}

#[cfg(test)]
mod tests {
    use super::{convert, extract_title};

    #[test]
    fn serializes_frontmatter_values_as_quoted_yaml_strings() {
        let markdown = convert(
            "https://example.com/a:b#fragment",
            r#"<html><head><title>Mapping: #1 &quot;draft&quot; \ path</title></head><body>Content</body></html>"#,
            &["title", "source"],
        )
        .unwrap();

        assert_eq!(
            markdown,
            "---\ntitle: \"Mapping: #1 \\\"draft\\\" \\\\ path\"\nsource: \"https://example.com/a:b#fragment\"\n---\n\nContent"
        );
    }

    #[test]
    fn escapes_line_breaks_in_frontmatter_values() {
        let markdown = convert(
            "https://example.com/first\nsecond\rthird\tfourth",
            "<html><body>Content</body></html>",
            &["source"],
        )
        .unwrap();

        assert_eq!(
            markdown,
            "---\nsource: \"https://example.com/first\\u000Asecond\\u000Dthird\\u0009fourth\"\n---\n\nContent"
        );
    }

    #[test]
    fn extracts_title_without_rendering_the_body() {
        let title = extract_title(
            "https://example.com",
            "<html><head><title>Page Title</title></head><body><p>Content</p></body></html>",
        )
        .unwrap();

        assert_eq!(title, Some("Page Title".to_string()));
        assert_eq!(extract_title("https://example.com", "<p>No title</p>").unwrap(), None);
    }
}
