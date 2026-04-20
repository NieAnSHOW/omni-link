# OmniLink 设计文档

**日期**: 2026-04-19
**状态**: 已确认

## Context

用户在多个平台收藏链接和内容后，面临两个核心痛点：(1) 收藏后很少再查阅，尤其是长视频；(2) 信息碎片化分散在各平台，查找困难。OmniLink 旨在解决这些问题，提供统一的跨平台链接采集、内容解析和知识结构化能力。

## 目标用户

个人知识管理用户，自部署，开源。

## 技术栈

Tauri v2 + Vue 3 + Vite + Rust 后端（Tauri 进程内） + SQLite（rusqlite, MVP）

## 架构设计：Tauri + Rust Backend

```
┌─────────────────────────────────────────────┐
│               Tauri v2 Application          │
│  ┌──────────┐  IPC   ┌───────────────────┐ │
│  │  Vue UI   │◄──────►│  Rust Backend     │ │
│  │ (前端界面) │invoke()│  - Tauri Commands │ │
│  └──────────┘        │  - 内容解析引擎    │ │
│                      │  - AI 集成层       │ │
│                      │  - 定时任务调度     │ │
│                      └──────┬────────────┘ │
│                             │               │
│                      ┌──────┴──────┐        │
│                      ▼             ▼        │
│                ┌──────────┐  ┌─────────┐    │
│                │  SQLite  │  │ WebDAV  │    │
│                │(主存储MVP)│  │ (备份)   │    │
│                └──────────┘  └─────────┘    │
└─────────────────────────────────────────────┘
         ▲           ▲           ▲
         │           │           │
    浏览器扩展    手机分享     API导入
```

**关键决策**:
- Rust 后端直接运行在 Tauri 进程内，前端通过 IPC `invoke()` 调用 Tauri Commands
- 无需额外进程（无 Sidecar），减少部署复杂度和进程间通信开销
- MVP 阶段使用 SQLite 作为唯一存储（rusqlite bundled, WAL 模式）
- 后续迭代引入 MongoDB 用于结构化内容和灵活查询
- WebDAV 用于数据备份和恢复

## 链接采集（5 种入口）

### 1. 手动输入
- UI 输入框粘贴链接，自动识别平台并触发解析
- 支持批量输入（多行文本自动提取 URL）

### 2. 浏览器扩展（v0.2）
- Chrome/Firefox 扩展，一键收藏当前页面
- 弹窗可选：标签、分类、备注
- 通过 Tauri IPC 或本地 HTTP 接口发送到 OmniLink

### 3. 手机分享投送（v0.4）
- 简单 PWA 页面，手机分享菜单发送到 OmniLink
- 同局域网投送或通过中继服务

### 4. 自动化导入（v0.2）
- 浏览器书签（HTML 格式）
- Pocket/Instapaper/Notion 导出数据
- RSS/Atom 订阅源定时抓取

### 5. 文档文件导入（v0.2）
- 支持 PDF、Markdown、TXT、EPUB
- 拖拽到 UI 或指定目录自动监听

**统一入库流程**: 链接/文件 → 标准化采集模型 → 去重检测 → 入队解析 → 存储

## 内容解析引擎

### 解析流水线
```
原始链接/文件
  → 平台识别（URL 匹配规则）
  → 内容抓取（Headless Browser / HTTP）
  → 正文提取（Readability 算法）
  → 结构化输出（标题/正文/图片/元数据）
  → AI 增强（摘要/标签/分类）
  → 存储
```

### 平台适配

| 类型 | 抓取方式 | 示例 |
|------|---------|------|
| 通用网页 | readability + scraper（Rust） | 博客、新闻 |
| SPA/动态页面 | 后续支持（WebView 渲染） | 知乎、掘金 |
| 视频平台 | 平台 API + 字幕提取 | YouTube、B站 |
| 社交媒体 | 平台 API / 爬取 | Twitter/X、微博 |
| 文档文件 | 文件解析器 | PDF、Markdown |

### 视频特殊处理
- 元数据：标题、简介、封面、时长、作者
- 字幕/转录：YouTube API 获取字幕，B站第三方接口
- AI 视频理解：关键帧提取 → 多模态模型 → 时间轴标注
- 输出：章节划分 + 关键点 + 时间戳

## AI 能力层

### Provider 抽象模式
- 统一接口：generateSummary / extractTags / classifyContent / analyzeVideo / generateNote
- 支持云端（OpenAI、Claude）、本地（Ollama）、自定义 endpoint
- 离线降级：无 AI 时用规则引擎（正则+关键词提取）

### 功能与优先级

| 功能 | 优先级 |
|------|--------|
| 内容摘要 | P0 |
| 智能标签 | P0 |
| 自动分类 | P1 |
| 视频理解 | P1 |
| 笔记生成 | P1 |
| 语义搜索 | P2 |

## 数据存储

### MVP（SQLite）
- links：原始链接（url, platform, source, status, created_at）
- contents：解析后结构化内容（title, body_html, body_text, images, metadata）
- notes：用户笔记（关联 content_id）
- tags：标签（name, color, type: auto/manual）
- categories：分类树（支持嵌套，parent_id）
- ai_results：AI 处理结果（summary, tags, classification）
- user_settings：用户配置
- import_history：导入历史（去重依据）

### 后续（MongoDB + SQLite）
- MongoDB 存储内容和笔记（灵活查询、大数据量）
- SQLite 保留用户配置和同步状态
- 数据模型设计预留 MongoDB 迁移兼容性

### WebDAV 备份
- 定期导出为 JSON 通过 WebDAV 上传
- 增量备份（仅同步变更）
- 一键恢复

## 迭代计划

### v0.1 MVP — 核心采集 + 解析
- 手动输入链接 + 批量添加
- 通用网页内容抓取与解析（readability + scraper）
- AI 摘要 + 标签（支持云端/本地 Provider + 离线降级）
- SQLite 存储（rusqlite, WAL 模式）
- Vue 基础 UI（链接列表、内容详情、设置页）
- Rust 后端通过 Tauri IPC Commands 提供服务

### v0.2 — 扩展采集
- 浏览器扩展（Chrome）
- 文档文件导入（PDF/Markdown）
- 自动化导入（书签、RSS）
- SPA 页面解析（Puppeteer）

### v0.3 — 视频与高级 AI
- 视频平台解析（YouTube/B站）
- 视频字幕提取
- 视频理解 AI（关键帧 + 时间轴）
- 笔记生成
- 语义搜索（向量数据库）

### v0.4 — 多端与备份
- 手机分享投送（PWA）
- WebDAV 备份/恢复
- MongoDB 迁移
- 社交媒体平台适配
