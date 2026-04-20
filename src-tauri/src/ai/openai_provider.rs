use reqwest::Client;

use crate::error::{AppError, AppResult};
use super::ai_service::safe_truncate;

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: &str, base_url: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            model: model.to_string(),
        }
    }

    pub async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub async fn generate_summary(&self, text: &str, title: Option<&str>) -> AppResult<serde_json::Value> {
        let truncated = safe_truncate(text, 8000);
        let prompt = format!(
            "请对以下内容进行分析，返回JSON格式：\n\
             {{\"summary\": \"200字以内摘要\", \"tags\": [\"标签1\",\"标签2\",\"标签3\"], \"category\": \"一级分类/二级分类\"}}\n\
             分类规则：如果有现成分类树就选最匹配的路径，否则根据内容推断一个分类路径（最多两级）。不确定时category设为null。\n\n\
             标题：{}\n内容：{}",
            title.unwrap_or("未知"),
            truncated,
        );

        let body = serde_json::json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": prompt }],
            "response_format": { "type": "json_object" },
            "temperature": 0.3,
        });

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        let content_str = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("{}");

        let result: serde_json::Value = serde_json::from_str(content_str)
            .unwrap_or_else(|_| serde_json::json!({"summary": "", "tags": []}));

        Ok(result)
    }

    pub async fn raw_completion(&self, prompt: &str) -> AppResult<serde_json::Value> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": prompt }],
            "response_format": { "type": "json_object" },
            "temperature": 0.1,
        });

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let data: serde_json::Value = response.error_for_status()?.json().await?;
        let content_str = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("{}");

        let result: serde_json::Value = serde_json::from_str(content_str)
            .unwrap_or_else(|_| serde_json::json!({"title": "", "content": ""}));

        Ok(result)
    }
}