const MIN_CONTENT_LENGTH: usize = 200;

pub struct ContentJudgement {
    pub is_sufficient: bool,
    pub reason: String,
}

pub fn judge_content(title: &str, markdown: &str) -> ContentJudgement {
    let text = markdown.trim();

    if title.is_empty() && text.is_empty() {
        return ContentJudgement {
            is_sufficient: false,
            reason: "both title and content are empty".into(),
        };
    }

    if title.is_empty() {
        return ContentJudgement {
            is_sufficient: false,
            reason: "title is empty".into(),
        };
    }

    let clean_len = text.chars().filter(|c| !c.is_whitespace()).count();
    if clean_len < MIN_CONTENT_LENGTH {
        return ContentJudgement {
            is_sufficient: false,
            reason: format!("content too short: {} chars (min {})", clean_len, MIN_CONTENT_LENGTH),
        };
    }

    ContentJudgement {
        is_sufficient: true,
        reason: String::new(),
    }
}
