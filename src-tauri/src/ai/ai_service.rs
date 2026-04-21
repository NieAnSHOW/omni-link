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

/// Sanitize user input to prevent prompt injection attacks
/// Escapes special characters and removes potential instruction delimiters
pub fn sanitize_input(text: &str) -> String {
    text.replace("```", "'''")
        .replace("</s>", "")
        .replace("<|im_end|>", "")
        .replace("<|endoftext|>", "")
        .replace("###", "")
        .replace("---SYSTEM---", "")
        .replace("---USER---", "")
        .replace("---ASSISTANT---", "")
}

/// Truncate with metadata about whether truncation occurred
pub fn safe_truncate_with_info(text: &str, max_bytes: usize) -> (String, bool) {
    if text.len() <= max_bytes {
        return (text.to_string(), false);
    }
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    (text[..end].to_string(), true)
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

pub async fn organize_content(config: &AiConfig, text: &str) -> AppResult<serde_json::Value> {
    let result = match config.provider.as_str() {
        "openai" => {
            let provider = OpenAiProvider::new(
                &config.openai.api_key,
                &config.openai.base_url,
                &config.openai.model,
            );
            provider.process_content(text, "organize", None).await
        }
        "ollama" => {
            let provider = OllamaProvider::new(
                &config.ollama.base_url,
                &config.ollama.model,
            );
            provider.process_content(text, "organize", None).await
        }
        _ => {
            let provider = FallbackProvider::new();
            provider.process_content(text, "organize", None).await
        }
    };

    match result {
        Ok(val) => Ok(val),
        Err(_) => {
            let fallback = FallbackProvider::new();
            fallback.process_content(text, "organize", None).await
        }
    }
}

pub async fn expand_content(config: &AiConfig, text: &str, title: Option<&str>) -> AppResult<serde_json::Value> {
    let search_context = search_related_content(title, text).await.unwrap_or_default();

    let result = match config.provider.as_str() {
        "openai" => {
            let provider = OpenAiProvider::new(
                &config.openai.api_key,
                &config.openai.base_url,
                &config.openai.model,
            );
            provider.process_content(text, "expand", Some(&search_context)).await
        }
        "ollama" => {
            let provider = OllamaProvider::new(
                &config.ollama.base_url,
                &config.ollama.model,
            );
            provider.process_content(text, "expand", Some(&search_context)).await
        }
        _ => {
            let provider = FallbackProvider::new();
            provider.process_content(text, "expand", Some(&search_context)).await
        }
    };

    match result {
        Ok(val) => Ok(val),
        Err(_) => {
            let fallback = FallbackProvider::new();
            fallback.process_content(text, "expand", Some(&search_context)).await
        }
    }
}

async fn search_related_content(title: Option<&str>, text: &str) -> Result<String, String> {
    use std::process::Command;

    let title_part = title.unwrap_or("");
    let text_preview: String = text.chars().take(100).collect();
    let query = if title_part.is_empty() {
        text_preview
    } else {
        format!("{} {}", title_part, text_preview)
    };

    let dispatcher_path = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("src-tauri/skills/unified-search/dispatcher.py");

    let output = Command::new("python3")
        .arg(&dispatcher_path)
        .arg(&query)
        .arg("--compact")
        .output()
        .map_err(|e| format!("Failed to execute dispatcher: {}", e))?;

    if !output.status.success() {
        return Ok(String::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or(serde_json::json!({}));

    let empty_vec = vec![];
    let results = json["results"].as_array().unwrap_or(&empty_vec);
    let mut summaries = Vec::new();
    for (i, r) in results.iter().take(5).enumerate() {
        let t = r["title"].as_str().unwrap_or("");
        let s = r["snippet"].as_str().or_else(|| r["summary"].as_str()).unwrap_or("");
        summaries.push(format!("{}. {} - {}", i + 1, t, s));
    }

    Ok(summaries.join("\n"))
}
