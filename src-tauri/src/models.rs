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
    pub content_status: Option<String>,
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
    pub content_status: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiResult {
    pub id: i64,
    pub content_id: i64,
    pub summary: Option<String>,
    pub tags: serde_json::Value,
    pub provider: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiResultParsed {
    pub summary: String,
    pub tags: Vec<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkDetail {
    pub link: Link,
    pub content: Option<ContentParsed>,
    pub ai: Option<AiResultParsed>,
    pub tags: Option<Vec<TagWithCount>>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub tag_type: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagWithCount {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub tag_type: String,
    pub content_count: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTagInput {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContentInput {
    pub id: i64,
    pub title: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
}
