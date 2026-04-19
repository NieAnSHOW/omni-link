use reqwest::Client;

use crate::error::AppResult;

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

    pub async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub async fn generate_summary(&self, text: &str, title: Option<&str>) -> AppResult<serde_json::Value> {
        let truncated = &text[..text.len().min(8000)];
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

        let result: serde_json::Value = serde_json::from_str(response_str)
            .unwrap_or_else(|_| serde_json::json!({"summary": "", "tags": []}));

        Ok(result)
    }
}
