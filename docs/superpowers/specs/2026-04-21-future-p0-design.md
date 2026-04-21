# Future-P0 功能设计

日期：2026-04-21
方案：统一命令模式（方案 A）

---

## 1. 启动页

避免首次启动白屏，展示品牌 splash。

**实现**：在 `App.vue` 中增加 splash 状态层（非 Tauri 多窗口）。

- 新增 `showSplash` ref，初始 `true`
- splash 层：`logo-no-text.png` 水平垂直居中 + "Omni-Link" 标题 + "链接新思维" 副标题
- `onMounted` 中 `Promise.all([minDelay(1000), appReady])`，取较长者完成后 `showSplash = false`
- fade-out 过渡动画（CSS transition opacity 0.3s）
- 深色背景，与品牌色调一致

**改动范围**：仅 `App.vue`，不涉及路由、AppLayout、Rust 后端。

---

## 2. AI 摘要（重命名 + 自动触发）

### 重命名

- `ContentView.vue` 按钮文案："AI 分析" → "AI 摘要"，loading 文案："分析中..." → "摘要生成中..."
- 已有摘要时按钮文案改为"重新生成摘要"
- 后端命令名 `analyze_content_cmd` 不变

### 自动触发

- `parseLink()` 成功后自动调用 `analyzeContent()`
- 新增 `autoAnalyzing` ref 控制自动摘要 loading 状态，与手动 `analyzing` 分开
- loading 文案显示在内容区域顶部："正在生成 AI 摘要..."
- 完成后 toast "AI 摘要完成"
- 手动按钮保留，用户可重新触发

**改动范围**：仅 `ContentView.vue`，后端无改动。

---

## 3. AI 整理/扩展

### 前端交互

- 在"编辑"按钮旁新增"AI 整理"按钮，仅当 `detail.content` 存在时显示
- 点击弹出小型 modal，三个选项：
  - **AI 整理内容** — 清理排版、去除冗余、补全结构
  - **AI 扩展内容** — 基于当前内容 + 搜索结果扩展补充
  - **整理并扩展** — 先整理后扩展
- 选择后关闭 modal，显示对应 loading 文案
- 完成后 `fetchDetail()` 刷新内容 + toast 提示

### 后端：`ai_process_content_cmd`

**参数**：`content_id: i64`, `mode: String`（`"organize"` | `"expand"` | `"both"`）

**处理流程**：

#### organize 模式

1. 从 content 表取 `body_text` + `title`
2. 构造整理 prompt，调用 AI（复用 provider 调度：OpenAI / Ollama / Fallback）
3. AI 返回 `{"body_text": "整理后的 Markdown"}`
4. 用 `marked` 在 Rust 端或前端重新渲染 body_html，更新 content 表

#### expand 模式

1. 从 content 表取 `body_text` + `title`
2. AI 提取扩展关键词（或直接用 title + 核心段落）
3. Shell 调用 `python3 src-tauri/skills/unified-search/dispatcher.py "关键词" --compact`，30s 超时
4. 解析 JSON stdout，提取搜索结果摘要
5. 构造扩展 prompt（原文 + 搜索结果），调用 AI
6. AI 返回 `{"body_text": "扩展后的 Markdown"}`
7. 更新 content 表

#### both 模式

1. 先执行 organize 逻辑，得到整理后文本
2. 用整理后文本执行 expand 逻辑
3. 一次写入 content 表

### AI Prompt 设计

**整理 prompt**：
```
你是一个内容整理专家。请对以下文档内容进行整理：
- 修正排版问题，使用规范的 Markdown 格式
- 去除冗余和重复内容
- 补全缺失的标题层级和结构
- 保留原文的核心信息和语义
- 输出 JSON 格式：{"body_text": "整理后的完整 Markdown 内容"}
```

**扩展 prompt**：
```
你是一个内容扩展专家。基于以下原文和搜索结果，对文档进行扩展：
- 针对原文中可以深入展开的内容进行补充
- 整合搜索结果中的相关信息
- 保持原文结构和核心内容不变
- 扩展内容自然融入原文，不突兀
- 使用 Markdown 格式
- 输出 JSON 格式：{"body_text": "扩展后的完整 Markdown 内容"}

原文：
{body_text}

搜索结果：
{search_results}
```

### Shell 调用 unified-search

- 路径：通过 `std::env::current_dir()` 获取项目根目录，拼接 `src-tauri/skills/unified-search/dispatcher.py`
- 使用 `std::process::Command::new("python3")` 调用
- 参数：`dispatcher.py "查询关键词" --compact`
- 超时：30 秒
- 解析 stdout JSON，提取 `results` 数组中的 `title` + `snippet`/`summary` 字段

### 注册

在 `lib.rs` 的 `invoke_handler` 中注册 `commands::content_commands::ai_process_content_cmd`。

### 前端 API 层

`useApi.ts` 新增：
```typescript
aiProcessContent(contentId: number, mode: string): Promise<{ success: boolean }>
```
调用 `invoke("ai_process_content_cmd", { contentId, mode })`

### 数据流

```
ContentView.vue
  → useApi.aiProcessContent(contentId, mode)
    → invoke("ai_process_content_cmd", { contentId, mode })
      → Rust: 取内容 → 按 mode 执行 → (可选) 搜索 → AI 处理 → 更新 content 表
      ← 返回 { success: true }
  → fetchDetail() 刷新页面
```

---

## 改动文件清单

| 文件 | 改动 |
|------|------|
| `src/App.vue` | 新增 splash 层 + 逻辑 |
| `src/views/ContentView.vue` | 重命名 + 自动摘要 + AI 整理按钮 + modal |
| `src/composables/useApi.ts` | 新增 `aiProcessContent` 方法 |
| `src-tauri/src/commands/content_commands.rs` | 新增 `ai_process_content_cmd` |
| `src-tauri/src/ai/ai_service.rs` | 新增 `organize_content` + `expand_content` 函数 |
| `src-tauri/src/ai/openai_provider.rs` | 新增 `process_content` 方法 |
| `src-tauri/src/ai/ollama_provider.rs` | 新增 `process_content` 方法 |
| `src-tauri/src/ai/fallback_provider.rs` | 新增 `process_content` 方法（规则引擎降级） |
| `src-tauri/src/lib.rs` | 注册新命令 |
