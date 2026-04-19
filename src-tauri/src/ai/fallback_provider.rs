use std::collections::HashMap;

use crate::error::AppResult;

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

    pub async fn generate_summary(&self, text: &str, _title: Option<&str>) -> AppResult<serde_json::Value> {
        let clean_text = &text[..text.len().min(10000)];
        let summary = generate_rule_summary(clean_text);
        let tags = extract_keywords(clean_text, 10);

        Ok(serde_json::json!({
            "summary": summary,
            "tags": tags,
        }))
    }
}

fn extract_keywords(text: &str, top_n: usize) -> Vec<String> {
    let re = regex::Regex::new(r"[^\w\u4e00-\u9fff]").unwrap();
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
