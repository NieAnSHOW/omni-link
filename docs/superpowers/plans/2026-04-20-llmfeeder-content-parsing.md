# LLMFeeder 内容解析实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 用 WebView 注入 Readability.js + Turndown.js 替换现有 Rust readability 解析，内容不足时降级到 LLM 提取，前端支持 Markdown 渲染。

**Architecture:** 隐藏 WebView 加载目标页面 → 注入 JS 提取 Markdown → 判断内容充足性 → 不足时降级 LLM → 存 SQLite。前端用 `marked` 库渲染 Markdown。

**Tech Stack:** Tauri v2 WebView, Readability.js (MIT), Turndown.js (MIT), marked (前端 Markdown 渲染)

---

## File Structure

### New Files
- `src-tauri/resources/js/readability.js` — Mozilla Readability.js vendored
- `src-tauri/resources/js/turndown.js` — Turndown.js vendored
- `src-tauri/resources/js/extract.js` — 注入的提取脚本
- `src-tauri/src/parser/webview_extractor.rs` — WebView 创建与 JS 注入
- `src-tauri/src/parser/content_judge.rs` — 内容充足性判断
- `src-tauri/src/parser/llm_extractor.rs` — LLM 降级提取

### Modified Files
- `src-tauri/src/parser/pipeline.rs` — 改用 WebView + 降级链
- `src-tauri/src/parser/extractor.rs` — 保留 metadata 提取辅助
- `src-tauri/src/parser/mod.rs` — 注册新模块
- `src-tauri/src/parser/fetcher.rs` — 保留用于降级获取 HTML
- `src-tauri/src/models.rs` — 添加 content_status 字段
- `src-tauri/src/db/schema.rs` — migration 添加 content_status
- `src-tauri/src/repositories/content_repo.rs` — 支持写入 content_status
- `src/views/ContentView.vue` — Markdown 渲染
- `package.json` — 添加 marked 依赖

---

### Task 1: Vendor JS Libraries

**Files:**
- Create: `src-tauri/resources/js/readability.js`
- Create: `src-tauri/resources/js/turndown.js`
- Create: `src-tauri/resources/js/extract.js`

- [ ] **Step 1: Download Readability.js**

Download `Readability.js` from Mozilla's readability repo (https://github.com/mozilla/readability) and place it at `src-tauri/resources/js/readability.js`. Use the latest stable release. The file is at `Readability.js` in the repo root.

```bash
mkdir -p src-tauri/resources/js
curl -L -o src-tauri/resources/js/readability.js https://raw.githubusercontent.com/mozilla/readability/main/Readability.js
```

- [ ] **Step 2: Download Turndown.js**

Download `turndown.js` from https://github.com/mixmark-io/turndown and place at `src-tauri/resources/js/turndown.js`.

```bash
curl -L -o src-tauri/resources/js/turndown.js https://raw.githubusercontent.com/mixmark-io/turndown/master/dist/turndown.js
```

- [ ] **Step 3: Create extract.js**

Create `src-tauri/resources/js/extract.js` with the extraction logic:

```javascript
(function() {
  try {
    var reader = new Readability(document.cloneNode(true));
    var article = reader.parse();

    if (!article || (!article.title && (!article.textContent || article.textContent.trim().length === 0))) {
      window.__TAURI__.event.emit('extract-result', {
        success: false,
        error: 'Readability failed to extract content'
      });
      return;
    }

    var turndownService = new TurndownService({
      headingStyle: 'atx',
      bulletListMarker: '-',
      codeBlockStyle: 'fenced'
    });
    var markdown = turndownService.turndown(article.content || '');

    var metadata = {
      description: article.excerpt || '',
      author: article.byline || '',
      site_name: article.siteName || '',
      image: '',
    };

    var ogImage = document.querySelector('meta[property="og:image"]');
    if (ogImage) metadata.image = ogImage.getAttribute('content') || '';

    var rawHtml = document.documentElement.outerHTML;

    window.__TAURI__.event.emit('extract-result', {
      success: true,
      title: article.title || '',
      markdown: markdown,
      body_html: article.content || '',
      raw_html: rawHtml,
      metadata: metadata,
      images: Array.from(document.querySelectorAll('img')).map(function(img) {
        return img.getAttribute('src') || '';
      }).filter(function(src) { return src.length > 0; })
    });
  } catch (e) {
    window.__TAURI__.event.emit('extract-result', {
      success: false,
      error: e.message || 'Unknown extraction error'
    });
  }
})();
```

- [ ] **Step 4: Verify files exist**

```bash
ls -la src-tauri/resources/js/
```

Expected: three files — readability.js, turndown.js, extract.js

- [ ] **Step 5: Commit**

```bash
git add src-tauri/resources/js/
git commit -m "feat: vendor readability.js, turndown.js, and extract.js"
```

---

### Task 2: Database Migration — content_status Field

**Files:**
- Modify: `src-tauri/src/db/schema.rs`
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/repositories/content_repo.rs`

- [ ] **Step 1: Add migration in schema.rs**

In `src-tauri/src/db/schema.rs`, update the `migrate` function to add `content_status` column:

```rust
fn migrate(conn: &Connection) -> AppResult<()> {
    // existing: category_id migration
    let has_category_id: bool = conn
        .prepare("SELECT category_id FROM links LIMIT 0")
        .is_ok();
    if !has_category_id {
        conn.execute_batch(
            "ALTER TABLE links ADD COLUMN category_id INTEGER;
             CREATE INDEX IF NOT EXISTS idx_links_category ON links(category_id);",
        )?;
    } else {
        conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_links_category ON links(category_id);")?;
    }

    // new: content_status migration
    let has_content_status: bool = conn
        .prepare("SELECT content_status FROM contents LIMIT 0")
        .is_ok();
    if !has_content_status {
        conn.execute_batch("ALTER TABLE contents ADD COLUMN content_status TEXT DEFAULT 'success';")?;
    }

    Ok(())
}
```

- [ ] **Step 2: Update Content model in models.rs**

In `src-tauri/src/models.rs`, add `content_status` to the `Content` struct:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    pub id: i64,
    pub link_id: i64,
    pub title: Option<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub images: String,
    pub metadata: String,
    pub content_status: Option<String>,
    pub created_at: String,
}
```

Also add it to `ContentParsed`:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentParsed {
    pub id: i64,
    pub link_id: i64,
    pub title: Option<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub images: Vec<String>,
    pub metadata: serde_json::Value,
    pub content_status: Option<String>,
    pub created_at: String,
}
```

- [ ] **Step 3: Update content_repo.rs**

Update `ContentRow`, `create_content`, and `row_to_content` in `src-tauri/src/repositories/content_repo.rs`:

```rust
pub struct ContentRow {
    pub id: i64,
    pub link_id: i64,
    pub title: Option<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub images: String,
    pub metadata: String,
    pub content_status: Option<String>,
    pub created_at: String,
}

pub fn create_content(
    conn: &Connection,
    link_id: i64,
    title: Option<&str>,
    body_html: Option<&str>,
    body_text: Option<&str>,
    images: &[String],
    metadata: &serde_json::Value,
    content_status: Option<&str>,
) -> AppResult<ContentRow> {
    let images_json = serde_json::to_string(images)?;
    let metadata_json = serde_json::to_string(metadata)?;
    let status = content_status.unwrap_or("success");

    conn.execute(
        "INSERT INTO contents (link_id, title, body_html, body_text, images, metadata, content_status) VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![link_id, title, body_html, body_text, images_json, metadata_json, status],
    )?;

    let id = conn.last_insert_rowid();
    get_content_by_id(conn, id)?.ok_or_else(|| crate::error::AppError::Db(rusqlite::Error::QueryReturnedNoRows))
}
```

Update `row_to_content`:

```rust
fn row_to_content(row: &rusqlite::Row) -> rusqlite::Result<ContentRow> {
    Ok(ContentRow {
        id: row.get("id")?,
        link_id: row.get("link_id")?,
        title: row.get("title")?,
        body_html: row.get("body_html")?,
        body_text: row.get("body_text")?,
        images: row.get("images")?,
        metadata: row.get("metadata")?,
        content_status: row.get("content_status")?,
        created_at: row.get("created_at")?,
    })
}
```

- [ ] **Step 4: Verify compilation**

```bash
cd src-tauri && cargo check 2>&1 | head -30
```

Expected: no errors related to content_status. There may be warnings about unused imports after pipeline changes (that's fine, Task 5 fixes pipeline).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db/schema.rs src-tauri/src/models.rs src-tauri/src/repositories/content_repo.rs
git commit -m "feat: add content_status field to contents table"
```

---

### Task 3: Content Judge Module

**Files:**
- Create: `src-tauri/src/parser/content_judge.rs`

- [ ] **Step 1: Create content_judge.rs**

```rust
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
```

- [ ] **Step 2: Verify compilation**

```bash
cd src-tauri && cargo check 2>&1 | head -20
```

Expected: compiles (not yet registered in mod.rs, that's Task 5)

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/parser/content_judge.rs
git commit -m "feat: add content judge module for sufficiency check"
```

---

### Task 4: WebView Extractor Module

**Files:**
- Create: `src-tauri/src/parser/webview_extractor.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add tauri webview dependency in Cargo.toml**

In `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
tauri = { version = "2", features = ["unstable"] }
```

The `unstable` feature enables `WebviewWindowBuilder::on_page_load` and `webview.evaluate_script`.

- [ ] **Step 2: Create webview_extractor.rs**

```rust
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

use serde::Deserialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize, Clone)]
pub struct WebviewExtractResult {
    pub success: bool,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub markdown: String,
    #[serde(default)]
    pub body_html: String,
    #[serde(default)]
    pub raw_html: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default)]
    pub images: Vec<String>,
}

pub async fn extract_via_webview(app: &AppHandle, url: &str) -> AppResult<WebviewExtractResult> {
    let (tx, rx) = mpsc::channel::<WebviewExtractResult>();
    let tx = std::sync::Mutex::new(Some(tx));

    let label = format!("extractor-{}", std::process::id());

    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(url.parse().map_err(|e: url::ParseError| AppError::Parse(e.to_string()))?))
        .title("Content Extractor")
        .visible(false)
        .inner_size(1280.0, 800.0)
        .on_page_load(move |webview, payload| {
            if payload.event() == tauri::webview::PageLoadEvent::Finished {
                let resources_dir = webview.app_handle().path().resource_dir();
                let js_dir = match resources_dir {
                    Ok(dir) => dir.join("js"),
                    Err(_) => return,
                };

                let readability_js = std::fs::read_to_string(js_dir.join("readability.js")).unwrap_or_default();
                let turndown_js = std::fs::read_to_string(js_dir.join("turndown.js")).unwrap_or_default();
                let extract_js = std::fs::read_to_string(js_dir.join("extract.js")).unwrap_or_default();

                let combined = format!(
                    "(()=>{{{};{};{};}})();",
                    readability_js, turndown_js, extract_js
                );

                let _ = webview.evaluate_js(&combined);
            }
        })
        .build()
        .map_err(|e| AppError::Parse(format!("Failed to create webview: {}", e)))?;

    // Listen for extraction result
    let app_clone = app.clone();
    let label_clone = label.clone();
    let tx_clone = tx.lock().unwrap().take();
    app.listen("extract-result", move |event| {
        if let Ok(result) = serde_json::from_str::<WebviewExtractResult>(event.payload()) {
            if let Some(tx) = tx_clone.as_ref() {
                let _ = tx.send(result);
            }
            // Destroy the webview window
            if let Some(w) = app_clone.get_webview_window(&label_clone) {
                let _ = w.destroy();
            }
        }
    });

    // Wait for result with timeout
    let result = rx.recv_timeout(Duration::from_secs(20))
        .map_err(|_| {
            let _ = window.destroy();
            AppError::Parse("Webview extraction timed out".into())
        })?;

    if !result.success {
        return Err(AppError::Parse(result.error.unwrap_or_else(|| "Unknown extraction error".into())));
    }

    Ok(result)
}
```

- [ ] **Step 3: Verify compilation**

```bash
cd src-tauri && cargo check 2>&1 | head -30
```

Expected: may show warnings about unused code. No errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/parser/webview_extractor.rs src-tauri/Cargo.toml
git commit -m "feat: add webview extractor module"
```

---

### Task 5: LLM Extractor Module

**Files:**
- Create: `src-tauri/src/parser/llm_extractor.rs`

- [ ] **Step 1: Create llm_extractor.rs**

This module uses the existing AI provider pattern to extract content from raw HTML when the WebView extraction is insufficient.

```rust
use crate::ai::ai_service;
use crate::config::AiConfig;
use crate::error::AppResult;

pub struct LlmExtractResult {
    pub title: String,
    pub markdown: String,
}

pub async fn extract_via_llm(config: &AiConfig, html: &str, url: &str) -> AppResult<LlmExtractResult> {
    let truncated = truncate_html(html, 50000);
    let prompt = format!(
        "你是一个网页内容提取器。请从以下HTML中提取文章的标题和正文内容。\n\
         要求：\n\
         1. 只提取核心文章内容，忽略导航栏、侧边栏、广告、评论区等\n\
         2. 正文输出为 Markdown 格式\n\
         3. 返回 JSON: {{\"title\": \"标题\", \"content\": \"Markdown正文\"}}\n\n\
         URL: {}\n\n\
         HTML:\n{}",
        url, truncated
    );

    let ai_config = AiConfig {
        provider: config.provider.clone(),
        openai: config.openai.clone(),
        ollama: config.ollama.clone(),
    };

    let result = ai_service::extract_content(&ai_config, &prompt).await?;

    let title = result["title"].as_str().unwrap_or("").to_string();
    let content = result["content"].as_str().unwrap_or("").to_string();

    Ok(LlmExtractResult {
        title,
        markdown: content,
    })
}

fn truncate_html(html: &str, max_chars: usize) -> String {
    if html.len() <= max_chars {
        return html.to_string();
    }

    // Try to keep <article>, <main>, <body> content first
    for tag in &["article", "main", "body"] {
        let open = format!("<{}", tag);
        if let Some(start) = html.find(&open) {
            let subset = &html[start..];
            if subset.len() <= max_chars {
                return subset.to_string();
            }
        }
    }

    html[..max_chars].to_string()
}
```

- [ ] **Step 2: Add extract_content to ai_service.rs**

In `src-tauri/src/ai/ai_service.rs`, add a new function for content extraction (separate from summary generation):

```rust
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
```

- [ ] **Step 3: Add raw_completion to providers**

In `src-tauri/src/ai/openai_provider.rs`, add a `raw_completion` method to `OpenAiProvider`:

```rust
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

    let data: serde_json::Value = response.json().await?;
    let content_str = data["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("{}");

    let result: serde_json::Value = serde_json::from_str(content_str)
        .unwrap_or_else(|_| serde_json::json!({"title": "", "content": ""}));

    Ok(result)
}
```

Similarly add to `src-tauri/src/ai/ollama_provider.rs` (follow the existing pattern for `generate_summary` but use the raw prompt).

Similarly add to `src-tauri/src/ai/fallback_provider.rs` — return an error since fallback can't do content extraction:

```rust
pub async fn raw_completion(&self, _prompt: &str) -> AppResult<serde_json::Value> {
    Err(crate::error::AppError::Ai("Fallback provider cannot extract content".into()))
}
```

- [ ] **Step 4: Verify compilation**

```bash
cd src-tauri && cargo check 2>&1 | head -30
```

Expected: no errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/parser/llm_extractor.rs src-tauri/src/ai/
git commit -m "feat: add LLM extractor module for fallback content extraction"
```

---

### Task 6: Wire Up Pipeline

**Files:**
- Modify: `src-tauri/src/parser/pipeline.rs`
- Modify: `src-tauri/src/parser/mod.rs`
- Modify: `src-tauri/src/commands/link_commands.rs`

- [ ] **Step 1: Update mod.rs**

Replace `src-tauri/src/parser/mod.rs` with:

```rust
pub mod identifier;
pub mod fetcher;
pub mod extractor;
pub mod content_judge;
pub mod webview_extractor;
pub mod llm_extractor;
pub mod pipeline;
```

- [ ] **Step 2: Rewrite pipeline.rs**

Replace `src-tauri/src/parser/pipeline.rs` entirely:

```rust
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::config::ConfigState;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::repositories::{content_repo, link_repo};

use super::{content_judge, identifier, llm_extractor, webview_extractor};

#[derive(Serialize)]
pub struct ParseResult {
    pub link_id: i64,
    pub platform: String,
    pub content: Option<ContentSummary>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ContentSummary {
    pub title: String,
    pub images_count: usize,
}

pub async fn parse_link(
    app: &AppHandle,
    state: &State<'_, DbState>,
    config_state: &State<'_, ConfigState>,
    link_id: i64,
) -> AppResult<ParseResult> {
    let (url, existing_title) = {
        let conn = state.0.lock().unwrap();
        let link = link_repo::get_link_by_id(&conn, link_id)?
            .ok_or_else(|| AppError::NotFound(format!("Link {} not found", link_id)))?;
        link_repo::update_link_status(&conn, link_id, "parsing")?;
        (link.url, link.title)
    };

    let platform = identifier::identify_platform(&url).to_string();

    // Step 1: WebView extraction
    let extract_result = match webview_extractor::extract_via_webview(app, &url).await {
        Ok(result) => result,
        Err(e) => {
            let conn = state.0.lock().unwrap();
            link_repo::update_link_status(&conn, link_id, "failed")?;
            return Ok(ParseResult {
                link_id,
                platform,
                content: None,
                error: Some(format!("WebView extraction failed: {}", e)),
            });
        }
    };

    let title = if extract_result.title.is_empty() {
        existing_title.as_deref()
    } else {
        Some(extract_result.title.as_str())
    };

    // Step 2: Judge content sufficiency
    let judgement = content_judge::judge_content(
        title.unwrap_or(""),
        &extract_result.markdown,
    );

    let (final_markdown, final_status) = if judgement.is_sufficient {
        (extract_result.markdown.clone(), "success")
    } else {
        // Step 3: LLM fallback
        let config = config_state.0.lock().unwrap();
        match llm_extractor::extract_via_llm(&config, &extract_result.raw_html, &url).await {
            Ok(llm_result) => (llm_result.markdown, "llm_fallback"),
            Err(_) => (extract_result.markdown.clone(), "failed"),
        }
    };

    // Step 4: Store results
    let meta_value = serde_json::to_value(&extract_result.metadata)?;
    let images_json = serde_json::to_string(&extract_result.images)?;

    {
        let conn = state.0.lock().unwrap();
        content_repo::create_content(
            &conn,
            link_id,
            title,
            Some(&extract_result.body_html),
            Some(&final_markdown),
            &extract_result.images,
            &meta_value,
            Some(final_status),
        )?;

        if let Some(ref t) = title {
            if !t.is_empty() {
                link_repo::update_link_title(&conn, link_id, t)?;
            }
        }
        link_repo::update_link_status(&conn, link_id, "parsed")?;
    }

    Ok(ParseResult {
        link_id,
        platform,
        content: Some(ContentSummary {
            title: title.unwrap_or("").to_string(),
            images_count: extract_result.images.len(),
        }),
        error: None,
    })
}
```

- [ ] **Step 3: Update link_commands.rs**

In `src-tauri/src/commands/link_commands.rs`, update `parse_link_cmd` to pass `AppHandle` and `ConfigState`:

```rust
#[tauri::command]
pub async fn parse_link_cmd(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    config: State<'_, ConfigState>,
    id: i64,
) -> AppResult<serde_json::Value> {
    let result = pipeline::parse_link(&app, &state, &config, id).await?;
    Ok(serde_json::to_value(result)?)
}
```

- [ ] **Step 4: Verify compilation**

```bash
cd src-tauri && cargo check 2>&1 | head -30
```

Expected: no errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/parser/
git commit -m "feat: rewrite pipeline with WebView + LLM fallback"
```

---

### Task 7: Frontend Markdown Rendering

**Files:**
- Modify: `package.json`
- Modify: `src/views/ContentView.vue`

- [ ] **Step 1: Install marked**

```bash
npm install marked
```

- [ ] **Step 2: Update ContentView.vue template**

In `src/views/ContentView.vue`, replace the content-body section (lines 37-41):

```html
<div v-if="detail.content" class="content-body">
  <div v-if="detail.content.body_text" class="markdown-content" v-html="renderedMarkdown"></div>
  <div v-else-if="detail.content.body_html" v-html="detail.content.body_html" class="html-content"></div>
  <p v-else class="empty-state">无可显示内容</p>
</div>
```

- [ ] **Step 3: Update ContentView.vue script**

Add `marked` import and computed property. In the `<script setup>` section:

```typescript
import { marked } from 'marked';

const renderedMarkdown = computed(() => {
  if (!detail.value?.content?.body_text) return '';
  return marked(detail.value.content.body_text);
});
```

- [ ] **Step 4: Update ContentView.vue styles**

Replace `.content-body` styles and add markdown-specific styles:

```css
.content-body {
  line-height: 1.8;
  font-size: 15px;
  color: #334155;
}
.markdown-content {
  overflow-y: auto;
}
.markdown-content :deep(h1) { font-size: 24px; margin: 24px 0 12px; border-bottom: 1px solid #e2e8f0; padding-bottom: 8px; }
.markdown-content :deep(h2) { font-size: 20px; margin: 20px 0 10px; border-bottom: 1px solid #e2e8f0; padding-bottom: 6px; }
.markdown-content :deep(h3) { font-size: 18px; margin: 16px 0 8px; }
.markdown-content :deep(h4) { font-size: 16px; margin: 14px 0 6px; }
.markdown-content :deep(p) { margin: 8px 0; }
.markdown-content :deep(ul), .markdown-content :deep(ol) { padding-left: 24px; margin: 8px 0; }
.markdown-content :deep(li) { margin: 4px 0; }
.markdown-content :deep(blockquote) { border-left: 4px solid #6366f1; padding: 4px 16px; margin: 12px 0; color: #64748b; background: #f8fafc; }
.markdown-content :deep(code) { background: #f1f5f9; padding: 2px 6px; border-radius: 4px; font-size: 13px; }
.markdown-content :deep(pre) { background: #1e293b; color: #e2e8f0; padding: 16px; border-radius: 8px; overflow-x: auto; margin: 12px 0; }
.markdown-content :deep(pre code) { background: none; padding: 0; color: inherit; }
.markdown-content :deep(a) { color: #6366f1; text-decoration: none; }
.markdown-content :deep(a:hover) { text-decoration: underline; }
.markdown-content :deep(img) { max-width: 100%; border-radius: 8px; margin: 8px 0; }
.markdown-content :deep(table) { border-collapse: collapse; width: 100%; margin: 12px 0; }
.markdown-content :deep(th), .markdown-content :deep(td) { border: 1px solid #e2e8f0; padding: 8px 12px; text-align: left; }
.markdown-content :deep(th) { background: #f8fafc; }
```

Remove the old `.text-content` style (no longer needed).

- [ ] **Step 5: Verify frontend builds**

```bash
npm run build 2>&1 | tail -10
```

Expected: build succeeds

- [ ] **Step 6: Commit**

```bash
git add package.json package-lock.json src/views/ContentView.vue
git commit -m "feat: add Markdown rendering in ContentView with marked library"
```
