# LLMFeeder 风格网页内容解析设计

## 背景

当前 omni-link 使用 Rust `readability` crate 通过 HTTP 请求获取静态 HTML 来提取网页内容。这种方式无法处理 JavaScript 动态渲染的页面（如知乎、掘金等 SPA），导致内容提取质量不稳定。

LLMFeeder（https://github.com/jatinkrmalik/LLMFeeder）是一个浏览器扩展，通过 Readability.js + Turndown.js 将网页内容转为干净的 Markdown。本设计将其解析方式集成到 omni-link 中。

## 目标

1. 使用 WebView 注入 Readability.js + Turndown.js 提取网页内容，转为 Markdown
2. 当提取内容不足时，降级到 LLM 从原始 HTML 提取内容
3. Markdown 结果存入现有 `body_text` 字段，数据库 schema 不变

## 架构

### 新解析管道

```
URL → 创建隐藏 WebView 加载页面 → 注入 readability.js + turndown.js
     → 获取 Markdown + 原始 HTML → 内容充足性判断
         ├─ 充足 → 存 SQLite (body_text = Markdown)
         └─ 不足 → 原始 HTML 发送 AI provider → LLM 提取 → 存 SQLite
```

### 替代原有管道

```
URL → reqwest 获取 HTML → readability crate 提取 → 存 SQLite
```

## 组件设计

### 1. WebView 提取器 (`webview_extractor.rs`)

负责创建隐藏 WebView 窗口并注入 JS 提取内容。

**JS 库管理：**
- `readability.js` 和 `turndown.js` 两个文件 vendor 到 `src-tauri/resources/js/`
- MIT 协议，可直接使用

**隐藏窗口创建流程：**
1. `WebviewWindowBuilder` 创建不可见窗口（`visible: false`，1280x800）
2. 导航到目标 URL
3. 监听页面加载完成（`on_page_load`）
4. 注入脚本顺序：readability.js → turndown.js → extract.js
5. extract.js 执行提取并回传结果

**注入脚本 (`extract.js`) 职责：**
- 调用 `new Readability(document).parse()` 获取文章内容
- 用 `new TurndownService().turndown(html)` 转为 Markdown
- 提取 metadata（title, description, author, site_name）
- 提取 `document.documentElement.outerHTML` 作为原始 HTML（供降级使用）
- 通过 Tauri IPC 将结果回传 Rust

**并发控制：**
- 同时最多 2 个隐藏 WebView 实例
- 使用信号量控制并发

**超时：**
- 页面加载：15 秒（可配置）
- JS 执行：5 秒

### 2. 内容充足性判断 (`content_judge.rs`)

判断提取内容是否足够用于后续 AI 分析。

**判断规则：**
- 去除空白后文本长度 < 200 字符 → 不足
- title 为空 → 不足
- 内容仅包含导航/版权文本 → 不足

**配置：**
- 阈值（默认 200）可通过 config.json 调整

### 3. LLM 降级提取 (`llm_extractor.rs`)

当 WebView 提取内容不足时，使用 LLM 从原始 HTML 提取。

**实现：**
- 复用现有 `ai_service.rs` 的 provider 模式（OpenAI / Ollama）
- 将原始 HTML 截断到 50K 字符后发送给 LLM
- HTML 超长时优先保留 `<article>`、`<main>`、`<body>` 区域
- prompt 要求 LLM 提取标题和正文，输出 Markdown
- 返回的 Markdown 直接存入 `body_text`

### 4. 管道协调 (`pipeline.rs` 更新)

协调新的提取流程和降级逻辑：

```
1. WebView 提取 → 得到 Markdown + 原始 HTML
2. content_judge 判断
   ├─ 充足 → 完成
   └─ 不足 → LLM 提取 → 存入
3. LLM 也失败 → 标记 content_status = "failed"
```

## 文件变更

### 新增文件

| 文件 | 说明 |
|------|------|
| `src-tauri/resources/js/readability.js` | Mozilla Readability.js (vendored, MIT) |
| `src-tauri/resources/js/turndown.js` | Turndown.js (vendored, MIT) |
| `src-tauri/resources/js/extract.js` | 注入的提取脚本 |
| `src-tauri/src/parser/webview_extractor.rs` | WebView 创建与 JS 注入 |
| `src-tauri/src/parser/content_judge.rs` | 内容充足性判断 |
| `src-tauri/src/parser/llm_extractor.rs` | LLM 降级提取 |

### 修改文件

| 文件 | 变更 |
|------|------|
| `src-tauri/src/parser/pipeline.rs` | 改用 WebView 提取 + 降级链 |
| `src-tauri/src/parser/extractor.rs` | 保留 metadata 提取，主要逻辑迁移至 webview_extractor |
| `src-tauri/src/parser/fetcher.rs` | 保留用于非 WebView 场景（favicon 等） |
| `src-tauri/Cargo.toml` | 可能需要更新 webview 相关依赖 |
| `src-tauri/src/models.rs` | 添加 content_status 字段 |

### 不变文件

- `identifier.rs` — 平台识别逻辑不变
- `ai_service.rs` / 各 provider — 复用现有 AI 调用

## 前端：Markdown 渲染

`body_text` 现在存储 Markdown 而非纯文本，`ContentView.vue` 需要支持 Markdown 渲染。

**变更文件：** `src/views/ContentView.vue`

**当前行为（第 38 行）：**
- `{{ detail.content.body_text }}` — 纯文本插值，Markdown 标记会原样显示

**新行为：**
- 引入 Markdown 渲染库（如 `marked`），将 `body_text` 解析为 HTML 后渲染
- 使用 `v-html` 输出，配合 CSS 样式确保排版美观
- 移除 `.content-body` 上的 `white-space: pre-wrap`（Markdown 渲染后自带换行）

**新增依赖：**
- `marked`（轻量 Markdown 解析库，~20KB，MIT 协议）
- 可选：`highlight.js` 用于代码块语法高亮

**渲染优先级：**
1. `body_text` 存在 → Markdown 渲染
2. `body_html` 存在 → 直接 `v-html` 渲染
3. 都不存在 → 显示"无可显示内容"

## 存储策略

- WebView 提取的 Markdown 直接存入现有 `body_text` 字段
- `body_html` 字段继续保留（存 Readability 输出的 HTML）
- 新增 `content_status` 字段（TEXT，默认 "success"）用于跟踪解析状态：`success` / `llm_fallback` / `failed`
- 需要一次数据库 migration 添加该字段

## 错误处理

| 场景 | 处理 |
|------|------|
| 页面加载超时 | 返回错误，不降级到 LLM（HTML 不完整） |
| JS 注入失败 | 降级到 LLM，使用 reqwest 获取的原始 HTML |
| LLM 提取失败 | 标记 content_status = "failed"，提示用户 |
| WebView 创建失败 | 降级到 LLM，使用 reqwest 获取的原始 HTML |
