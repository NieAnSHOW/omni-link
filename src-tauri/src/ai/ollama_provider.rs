use reqwest::Client;

use crate::error::AppResult;
use super::ai_service::{safe_truncate, safe_truncate_with_info, sanitize_input};

/// Strip Markdown code block markers from JSON responses
/// Handles both ```json and ``` wrapped responses
fn strip_markdown_json(s: &str) -> &str {
    let trimmed = s.trim();
    if trimmed.starts_with("```json") {
        // Remove ```json at start and ``` at end
        let without_start = trimmed.strip_prefix("```json").unwrap_or(trimmed).trim();
        without_start.strip_suffix("```").unwrap_or(without_start).trim()
    } else if trimmed.starts_with("```") {
        // Remove ``` at start and end
        let without_start = trimmed.strip_prefix("```").unwrap_or(trimmed).trim();
        without_start.strip_suffix("```").unwrap_or(without_start).trim()
    } else {
        trimmed
    }
}

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            model: model.to_string(),
        }
    }

    #[allow(dead_code)]
    pub async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub async fn generate_summary(&self, text: &str, title: Option<&str>) -> AppResult<serde_json::Value> {
        let truncated = safe_truncate(text, 8000);
        let prompt = format!(
            "请对以下内容进行分析，严格返回JSON格式：\n\
             {{\"summary\": \"200字以内摘要\", \"tags\": [\"标签1\",\"标签2\",\"标签3\"]}}\n\n\
             标题：{}\n内容：{}",
            title.unwrap_or("未知"),
            truncated,
        );

        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "format": "json",
        });

        let response = self.client
            .post(format!("{}/api/generate", self.base_url))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        let response_str = data["response"].as_str().unwrap_or("{}");

        let cleaned = strip_markdown_json(response_str);
        let result: serde_json::Value = serde_json::from_str(cleaned)
            .unwrap_or_else(|_| serde_json::json!({"summary": "", "tags": []}));

        Ok(result)
    }

    pub async fn raw_completion(&self, prompt: &str) -> AppResult<serde_json::Value> {
        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "format": "json",
        });

        let response = self.client
            .post(format!("{}/api/generate", self.base_url))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let data: serde_json::Value = response.error_for_status()?.json().await?;
        let response_str = data["response"].as_str().unwrap_or("{}");

        let cleaned = strip_markdown_json(response_str);
        let result: serde_json::Value = serde_json::from_str(cleaned)
            .unwrap_or_else(|_| serde_json::json!({"title": "", "content": ""}));

        Ok(result)
    }

    pub async fn process_content(&self, text: &str, mode: &str, search_context: Option<&str>) -> AppResult<serde_json::Value> {
        // SECURITY FIX: Sanitize inputs to prevent prompt injection
        let sanitized_text = sanitize_input(text);
        let sanitized_search = search_context.map(sanitize_input);

        // TRUNCATION FIX: Track if content was truncated
        let (truncated, was_truncated) = safe_truncate_with_info(&sanitized_text, 12000);

        let prompt = match mode {
            "organize" => format!(
                "你是一个内容整理专家。请对以下文档内容进行整理：\n\
                 - 修正排版问题，使用规范的 Markdown 格式\n\
                 - 去除冗余和重复内容\n\
                 - 补全缺失的标题层级和结构\n\
                 - 保留原文的核心信息和语义，不要删减实质内容\n\
                 - 严格返回 JSON 格式：{{\"body_text\": \"整理后的完整 Markdown 内容\"}}\n\n\
                 内容：{}", truncated
            ),
            "expand" => {
                let search_part = sanitized_search.as_deref().unwrap_or("无额外搜索结果");
                format!(
                    "你是一个内容扩展专家。基于以下原文和搜索结果，对文档进行扩展：\n\
                     - 针对原文中可以深入展开的内容进行补充\n\
                     - 整合搜索结果中的相关信息\n\
                     - 保持原文结构和核心内容不变\n\
                     - 扩展内容自然融入原文，不突兀\n\
                     - 使用 Markdown 格式\n\
                     - 严格返回 JSON 格式：{{\"body_text\": \"扩展后的完整 Markdown 内容\"}}\n\n\
                     原文：{}\n\n搜索结果：{}", truncated, search_part
                )
            }
            _ => return Err(crate::error::AppError::Ai(format!("Unknown mode: {}", mode))),
        };

        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "format": "json",
        });

        let response = self.client
            .post(format!("{}/api/generate", self.base_url))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let data: serde_json::Value = response.error_for_status()?.json().await?;

        // Log the full response for debugging
        eprintln!("Ollama API response: {}", serde_json::to_string_pretty(&data).unwrap_or_else(|_| "Failed to serialize".to_string()));

        // Check for API errors
        if let Some(error) = data.get("error") {
            return Err(crate::error::AppError::Ai(format!("Ollama API error: {}", error)));
        }

        // ERROR HANDLING FIX: Properly propagate parse errors instead of silently returning empty objects
        let response_str = data["response"]
            .as_str()
            .ok_or_else(|| {
                eprintln!("Response structure: {:?}", data);
                crate::error::AppError::Ai("Ollama response missing response field".to_string())
            })?;

        let cleaned = strip_markdown_json(response_str);
        let mut result: serde_json::Value = serde_json::from_str(cleaned)
            .map_err(|e| crate::error::AppError::Ai(format!("Failed to parse Ollama response as JSON: {}", e)))?;

        // Add truncation warning to response
        if was_truncated {
            if let Some(obj) = result.as_object_mut() {
                obj.insert("truncated".to_string(), serde_json::json!(true));
                obj.insert("warning".to_string(), serde_json::json!("内容已截断至 12000 字节"));
            }
        }

        Ok(result)
    }
}