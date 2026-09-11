mod bindings;
pub mod core;

use html_to_markdown::{convert, extract_title};
use wasm_bindgen::prelude::*;

const FRONTMATTER_KEYS: &[&str] = &["title", "source"];

#[wasm_bindgen]
pub fn convert_html(url: &str, html: &str) -> Result<String, JsValue> {
    convert(url, html, FRONTMATTER_KEYS).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn extract_article_title(url: &str, html: &str) -> Result<Option<String>, JsValue> {
    extract_title(url, html).map_err(|error| JsValue::from_str(&error.to_string()))
}
