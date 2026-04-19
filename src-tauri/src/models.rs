use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub platform: Option<String>,
    pub source: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    pub id: i64,
    pub link_id: i64,
    pub title: Option<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub images: String,
    pub metadata: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentParsed {
    pub id: i64,
    pub link_id: i64,
    pub title: Option<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub images: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiResult {
    pub id: i64,
    pub content_id: i64,
    pub summary: Option<String>,
    pub tags: serde_json::Value,
    pub classification: Option<String>,
    pub provider: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiResultParsed {
    pub summary: String,
    pub tags: Vec<String>,
    pub classification: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkDetail {
    pub link: Link,
    pub content: Option<ContentParsed>,
    pub ai: Option<AiResultParsed>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinksResponse {
    pub links: Vec<Link>,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkInput {
    pub url: String,
    #[allow(dead_code)]
    pub title: Option<String>,
    #[allow(dead_code)]
    pub source: Option<String>,
}
