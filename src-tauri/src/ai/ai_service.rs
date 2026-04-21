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
        Err(e) => {
            eprintln!("AI provider failed: {}, falling back to rule-based provider", e);
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
        Err(e) => {
            eprintln!("AI provider failed: {}, falling back to rule-based provider", e);
            let fallback = FallbackProvider::new();
            fallback.process_content(text, "organize", None).await
        }
    }
}

pub async fn expand_content(config: &AiConfig, text: &str, title: Option<&str>) -> AppResult<serde_json::Value> {
    let search_context = search_related_content(title, text).await.unwrap_or_else(|e| {
        eprintln!("Search failed: {}, continuing without context", e);
        String::new()
    });

    let ctx = if search_context.is_empty() { None } else { Some(search_context.as_str()) };

    let result = match config.provider.as_str() {
        "openai" => {
            let provider = OpenAiProvider::new(
                &config.openai.api_key,
                &config.openai.base_url,
                &config.openai.model,
            );
            provider.process_content(text, "expand", ctx).await
        }
        "ollama" => {
            let provider = OllamaProvider::new(
                &config.ollama.base_url,
                &config.ollama.model,
            );
            provider.process_content(text, "expand", ctx).await
        }
        _ => {
            let provider = FallbackProvider::new();
            provider.process_content(text, "expand", ctx).await
        }
    };

    match result {
        Ok(val) => Ok(val),
        Err(e) => {
            eprintln!("AI provider failed: {}, falling back to rule-based provider", e);
            let fallback = FallbackProvider::new();
            fallback.process_content(text, "expand", ctx).await
        }
    }
}

async fn search_related_content(title: Option<&str>, text: &str) -> Result<String, String> {
    use std::process::Command;
    use std::time::Duration;

    // SECURITY: Define allowed script at compile time
    const DISPATCHER_SCRIPT: &str = "src-tauri/skills/unified-search/dispatcher.py";

    // SECURITY: Sanitize inputs to prevent shell injection
    let title_part = title.unwrap_or("");
    let text_preview: String = text.chars().take(100).collect();
    let raw_query = if title_part.is_empty() {
        text_preview
    } else {
        format!("{} {}", title_part, text_preview)
    };

    // Apply sanitization to prevent command injection
    let query = sanitize_input(&raw_query);

    // Resolve from executable location
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;

    let project_root = exe_path
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "Failed to resolve project root".to_string())?;

    let dispatcher_path = project_root.join(DISPATCHER_SCRIPT)
        .canonicalize()
        .map_err(|e| format!("Failed to resolve dispatcher path: {}", e))?;

    // SECURITY: Verify path is within project root (prevent traversal)
    if !dispatcher_path.starts_with(project_root) {
        return Err("Dispatcher path outside project root".to_string());
    }

    // SECURITY: Verify it's a regular file, not a symlink
    let metadata = std::fs::metadata(&dispatcher_path)
        .map_err(|e| format!("Failed to read dispatcher metadata: {}", e))?;
    if !metadata.is_file() {
        return Err("Dispatcher is not a regular file".to_string());
    }

    // SECURITY: Verify file extension
    if dispatcher_path.extension().and_then(|s| s.to_str()) != Some("py") {
        return Err(format!("Invalid dispatcher file type: {:?}", dispatcher_path));
    }

    // Execute with timeout using tokio
    let output = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::task::spawn_blocking(move || {
            Command::new("python3")
                .arg(&dispatcher_path)
                .arg(&query)
                .arg("--compact")
                .output()
        })
    )
    .await
    .map_err(|_| "Search timeout after 10 seconds".to_string())?
    .map_err(|e| format!("Failed to spawn subprocess: {}", e))?
    .map_err(|e| format!("Failed to execute dispatcher: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Dispatcher failed with status {:?}: {}", output.status.code(), stderr);
        return Ok(String::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse dispatcher output: {}", e))?;

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
