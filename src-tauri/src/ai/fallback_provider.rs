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

const CATEGORY_RULES: &[(&str, &str)] = &[
    ("rust|golang|python|java|typescript|javascript|编程|开发|api|框架", "技术/后端"),
    ("vue|react|angular|css|html|前端|组件|界面", "技术/前端"),
    ("ai|gpt|llm|模型|机器学习|深度学习|神经网络|transformer", "技术/AI"),
    ("设计|ui|ux|交互|体验|原型|figma", "设计"),
    ("产品|需求|用户|功能|迭代|mvp", "产品"),
    ("阅读|书籍|读书|文章|书评", "阅读"),
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
        let clean_text = &text[..text.len().min(10000)];
        let summary = generate_rule_summary(clean_text);
        let tags = extract_keywords(clean_text, 10);
        let category = classify_by_keywords(clean_text);

        Ok(serde_json::json!({
            "summary": summary,
            "tags": tags,
            "category": category,
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

fn classify_by_keywords(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    for (pattern, category) in CATEGORY_RULES {
        let re = regex::Regex::new(pattern).ok()?;
        if re.is_match(&lower) {
            return Some(category.to_string());
        }
    }
    None
}
