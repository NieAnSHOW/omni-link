use std::collections::HashMap;

use crate::error::AppResult;
use super::ai_service::{safe_truncate, safe_truncate_with_info, sanitize_input};

const STOP_WORDS: &[&str] = &[
    "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个",
    "上", "也", "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好",
    "自己", "这", "他", "她", "它", "们", "那", "些", "什么", "怎么", "如果", "但是",
    "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
    "have", "has", "had", "do", "does", "did", "will", "would", "could",
    "should", "may", "might", "shall", "can", "need", "dare", "ought",
    "and", "but", "or", "if", "then", "else", "when", "at", "from",
    "to", "in", "on", "with", "for", "of", "not", "this", "that",
];

pub struct FallbackProvider;

impl FallbackProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn is_available(&self) -> bool {
        true
    }

    pub async fn raw_completion(&self, _prompt: &str) -> AppResult<serde_json::Value> {
        Err(crate::error::AppError::Ai("Fallback provider cannot extract content".into()))
    }

    pub async fn generate_summary(&self, text: &str, _title: Option<&str>) -> AppResult<serde_json::Value> {
        let clean_text = safe_truncate(text, 10000);
        let summary = generate_rule_summary(clean_text);
        let tags = extract_keywords(clean_text, 10);

        Ok(serde_json::json!({
            "summary": summary,
            "tags": tags,
        }))
    }

    pub async fn process_content(&self, text: &str, mode: &str, _search_context: Option<&str>) -> AppResult<serde_json::Value> {
        // SECURITY FIX: Sanitize input to prevent any potential issues
        let sanitized_text = sanitize_input(text);

        // TRUNCATION FIX: Track if content was truncated
        let (clean_text, was_truncated) = safe_truncate_with_info(&sanitized_text, 10000);

        let mut result = match mode {
            "organize" => {
                // 简单整理：按段落去空行、添加基本结构
                let paragraphs: Vec<&str> = clean_text
                    .split("\n\n")
                    .map(|p| p.trim())
                    .filter(|p| !p.is_empty())
                    .collect();
                let organized = paragraphs.join("\n\n");
                serde_json::json!({ "body_text": organized })
            }
            "expand" => {
                // Fallback 无法扩展，返回原文
                serde_json::json!({ "body_text": clean_text })
            }
            _ => return Err(crate::error::AppError::Ai(format!("Unknown mode: {}", mode))),
        };

        // Add truncation warning to response
        if was_truncated {
            if let Some(obj) = result.as_object_mut() {
                obj.insert("truncated".to_string(), serde_json::json!(true));
                obj.insert("warning".to_string(), serde_json::json!("内容已截断至 10000 字节"));
            }
        }

        Ok(result)
    }
}

fn extract_keywords(text: &str, top_n: usize) -> Vec<String> {
    let re = regex::Regex::new(r"[^\w一-鿿]").unwrap();
    let cleaned = re.replace_all(text, " ");
    let words: Vec<&str> = cleaned
        .split_whitespace()
        .filter(|w| w.len() > 1 && !STOP_WORDS.contains(&w.to_lowercase().as_str()))
        .collect();

    let mut freq: HashMap<String, usize> = HashMap::new();
    for word in words {
        let lower = word.to_lowercase();
        *freq.entry(lower).or_insert(0) += 1;
    }

    let mut entries: Vec<_> = freq.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    entries.into_iter().take(top_n).map(|(w, _)| w).collect()
}

fn generate_rule_summary(text: &str) -> String {
    let re = regex::Regex::new(r"[。！？.!?\n]").unwrap();
    let sentences: Vec<&str> = re
        .split(text)
        .filter(|s| s.trim().len() > 10)
        .collect();

    if sentences.is_empty() {
        return text.chars().take(300).collect();
    }

    let summary: String = sentences.iter().take(3).cloned().collect::<Vec<_>>().join("。");
    let chars: Vec<char> = summary.chars().take(300).collect();
    let mut result: String = chars.into_iter().collect();
    result.push_str("...");
    result
}
