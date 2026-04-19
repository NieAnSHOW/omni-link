use std::collections::HashMap;

use scraper::{Html, Selector};

use crate::error::AppResult;

pub struct ExtractedContent {
    pub title: String,
    pub body_html: String,
    pub body_text: String,
    pub images: Vec<String>,
    pub metadata: HashMap<String, String>,
}

pub fn extract_content(html: &str, url: &str) -> AppResult<ExtractedContent> {
    match try_readability(html, url) {
        Some(content) => Ok(content),
        None => Ok(fallback_extract(html)),
    }
}

fn try_readability(html: &str, url: &str) -> Option<ExtractedContent> {
    use readability::extractor::extract;

    let mut input = html.as_bytes();
    let parsed_url = reqwest::Url::parse(url).ok()?;
    let extracted = extract(&mut input, &parsed_url).ok()?;

    let title = extracted.title;
    let body_text = extracted.text;

    if title.is_empty() && body_text.trim().is_empty() {
        return None;
    }

    let body_html = extracted.content;
    let images = extract_images(&body_html);
    let metadata = extract_metadata(html);

    Some(ExtractedContent {
        title,
        body_html,
        body_text,
        images,
        metadata,
    })
}

fn fallback_extract(html: &str) -> ExtractedContent {
    let document = Html::parse_document(html);

    let title = Selector::parse("title").ok()
        .and_then(|sel| document.select(&sel).next())
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();

    let body = Selector::parse("body").ok()
        .and_then(|sel| document.select(&sel).next())
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();

    ExtractedContent {
        title,
        body_html: String::new(),
        body_text: body.trim().to_string(),
        images: Vec::new(),
        metadata: HashMap::new(),
    }
}

fn extract_images(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let sel = Selector::parse("img").unwrap();
    document
        .select(&sel)
        .filter_map(|el| el.value().attr("src").map(String::from))
        .filter(|src| !src.is_empty())
        .collect()
}

fn extract_metadata(html: &str) -> HashMap<String, String> {
    let document = Html::parse_document(html);
    let mut meta = HashMap::new();

    let get_meta = |property: &str, name: &str| -> Option<String> {
        let prop_sel = Selector::parse(&format!("meta[property=\"{}\"]", property)).ok()?;
        if let Some(el) = document.select(&prop_sel).next() {
            if let Some(content) = el.value().attr("content") {
                return Some(content.to_string());
            }
        }
        let name_sel = Selector::parse(&format!("meta[name=\"{}\"]", name)).ok()?;
        if let Some(el) = document.select(&name_sel).next() {
            if let Some(content) = el.value().attr("content") {
                return Some(content.to_string());
            }
        }
        None
    };

    if let Some(desc) = get_meta("og:description", "description") {
        meta.insert("description".into(), desc);
    }
    if let Some(img) = get_meta("og:image", "og:image") {
        meta.insert("image".into(), img);
    }
    if let Some(author) = get_meta("author", "author")
        .or_else(|| get_meta("og:site_name", "og:site_name"))
    {
        meta.insert("author".into(), author);
    }

    meta
}
