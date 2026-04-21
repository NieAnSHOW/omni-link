use crate::ai::ai_service;
use crate::config::AiConfig;
use crate::error::AppResult;

pub struct LlmExtractResult {
    pub title: String,
    pub markdown: String,
}

pub async fn extract_via_llm(config: &AiConfig, html: &str, url: &str) -> AppResult<LlmExtractResult> {
    let truncated = truncate_html(html, 50000);
    let prompt = format!(
        "你是一个网页内容提取器。请从以下HTML中提取文章的标题和正文内容。\n\
         要求：\n\
         1. 只提取核心文章内容，忽略导航栏、侧边栏、广告、评论区等\n\
         2. 正文输出为 Markdown 格式\n\
         3. 返回 JSON: {{\"title\": \"标题\", \"content\": \"Markdown正文\"}}\n\n\
         URL: {}\n\n\
         HTML:\n{}",
        url, truncated
    );

    let result = ai_service::extract_content(config, &prompt).await?;

    let title = result["title"].as_str().unwrap_or("").to_string();
    let content = result["content"].as_str().unwrap_or("").to_string();

    Ok(LlmExtractResult {
        title,
        markdown: content,
    })
}

fn truncate_html(html: &str, max_bytes: usize) -> String {
    if html.len() <= max_bytes {
        return html.to_string();
    }

    for tag in &["article", "main", "body"] {
        let open = format!("<{}", tag);
        if let Some(start) = html.find(&open) {
            let subset = &html[start..];
            if subset.len() <= max_bytes {
                return subset.to_string();
            }
        }
    }

    let mut end = max_bytes;
    while !html.is_char_boundary(end) {
        end -= 1;
    }
    html[..end].to_string()
}
