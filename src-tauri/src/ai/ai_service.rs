use crate::config::AiConfig;
use crate::error::AppResult;

use super::fallback_provider::FallbackProvider;
use super::ollama_provider::OllamaProvider;
use super::openai_provider::OpenAiProvider;

pub fn safe_truncate(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        return text;
    }
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

pub async fn extract_content(config: &AiConfig, prompt: &str) -> AppResult<serde_json::Value> {
    let result = match config.provider.as_str() {
        "openai" => {
            let provider = OpenAiProvider::new(
                &config.openai.api_key,
                &config.openai.base_url,
                &config.openai.model,
            );
            provider.raw_completion(prompt).await
        }
        "ollama" => {
            let provider = OllamaProvider::new(
                &config.ollama.base_url,
                &config.ollama.model,
            );
            provider.raw_completion(prompt).await
        }
        _ => Err(crate::error::AppError::Ai("No AI provider configured".into())),
    };

    match result {
        Ok(val) => Ok(val),
        Err(_) => {
            let fallback = FallbackProvider::new();
            fallback.raw_completion(prompt).await
        }
    }
}

pub async fn analyze_content(config: &AiConfig, text: &str, title: Option<&str>) -> AppResult<serde_json::Value> {
    let result = match config.provider.as_str() {
        "openai" => {
            let provider = OpenAiProvider::new(
                &config.openai.api_key,
                &config.openai.base_url,
                &config.openai.model,
            );
            provider.generate_summary(text, title).await
        }
        "ollama" => {
            let provider = OllamaProvider::new(
                &config.ollama.base_url,
                &config.ollama.model,
            );
            provider.generate_summary(text, title).await
        }
        _ => {
            let provider = FallbackProvider::new();
            provider.generate_summary(text, title).await
        }
    };

    match result {
        Ok(val) => Ok(val),
        Err(_) => {
            let fallback = FallbackProvider::new();
            fallback.generate_summary(text, title).await
        }
    }
}
