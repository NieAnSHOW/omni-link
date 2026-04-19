# OmniLink

跨平台链接采集 + 内容解析 + 知识结构化 + 视频理解

## 需求痛点

当用户阅读到不错的文章或是笔记内容，通常是直接收藏在单一应用中，大概率会存在两种问题：

1. 用户会很少再查阅这其中的内容，或是没有足够的耐心重新阅读其中的内容，尤其是视频内容是，长视频的情况下足以让用户放弃再次阅读
2. 由于获取这些信息的应用或是平台各不相同，用户收藏的信息过于碎片化，要是想查阅某篇内容需要来回找很多地方

## 项目愿景

OmniLink 是一个面向个人知识管理的跨平台桌面应用，核心解决链接收藏后的信息碎片化和二次阅读问题。

- **跨平台采集**：链接可来自手动输入、浏览器扩展、手机分享、自动化导入、文档文件
- **智能解析**：自动识别平台、抓取正文、提取结构化内容
- **AI 增强**：支持云端 API（OpenAI/Claude）和本地模型（Ollama），生成摘要、标签、分类
- **视频理解**：提取字幕、关键帧分析、时间轴标注（v0.3）
- **自部署 & 开源**：本地优先，数据完全可控

## 技术栈

> 项目强调跨平台 + 自部署

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri v2 (Rust) |
| 前端 | Vue 3 + TypeScript + Vite |
| 后端 | Rust（Tauri 进程内，IPC 通信） |
| 存储 | SQLite（rusqlite, WAL 模式） |
| 内容解析 | readability + scraper（Rust） |
| HTTP 客户端 | reqwest（rustls-tls） |
| AI | reqwest 调用 OpenAI/Ollama API + 规则引擎降级 |

## 架构设计

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

**关键决策**：
- Rust 后端直接运行在 Tauri 进程内，前端通过 IPC `invoke()` 调用 Tauri Commands
- 无需额外进程（无 Sidecar），减少部署复杂度和进程间通信开销
- MVP 阶段使用 SQLite 作为唯一存储（rusqlite bundled）
- 后续迭代引入 MongoDB 用于结构化内容和灵活查询
- WebDAV 用于数据备份和恢复

## 链接采集

5 种采集入口，统一入库流程：

| 入口 | 版本 | 说明 |
|------|------|------|
| 手动输入 | v0.1 | UI 内粘贴链接，自动识别平台并触发解析 |
| 浏览器扩展 | v0.2 | Chrome/Firefox 扩展一键收藏 |
| 手机分享 | v0.4 | 分享菜单发送到 OmniLink（PWA） |
| 自动化导入 | v0.2 | 浏览器书签、Pocket/Notion 导出、RSS |
| 文档文件 | v0.2 | PDF、Markdown、TXT、EPUB |

## 内容解析

解析流水线：`链接 → 平台识别 → 内容抓取（reqwest） → 正文提取（readability + scraper） → 结构化输出 → AI 增强 → 存储`

| 内容类型 | 抓取方式 | 示例 |
|----------|---------|------|
| 通用网页 | readability + scraper（Rust） | 博客、新闻 |
| SPA/动态页面 | 后续支持（WebView 渲染） | 知乎、掘金 |
| 视频平台 | 平台 API + 字幕提取 | YouTube、B站 |
| 社交媒体 | 平台 API / 爬取 | Twitter/X、微博 |
| 文档文件 | 文件解析器 | PDF、Markdown |

## AI 能力

Provider 抽象设计，支持用户自由选择：

| Provider | 说明 |
|----------|------|
| OpenAI / Claude | 云端 API，需配置 API Key |
| Ollama | 本地运行，隐私优先 |
| 离线降级 | 规则引擎（正则 + 关键词提取），无 AI 也可用 |

AI 功能及优先级：

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
- `links` — 原始链接
- `contents` — 解析后结构化内容
- `tags` / `content_tags` — 标签及关联
- `categories` — 分类树（支持嵌套）
- `ai_results` — AI 处理结果
- `user_settings` — 用户配置
- `import_history` — 导入历史（去重）

### 后续（MongoDB + SQLite）
- MongoDB 存储内容和笔记（灵活查询、大数据量）
- SQLite 保留用户配置和同步状态
- 数据模型设计预留 MongoDB 迁移兼容性

### WebDAV 备份
- 定期导出为 JSON 通过 WebDAV 上传
- 增量备份 + 一键恢复

## 迭代计划

### v0.1 MVP — 核心采集 + 解析（当前）
- [x] 项目初始化（Tauri v2 + Vue 3 + Rust 后端）
- [x] 手动输入链接 + 批量添加
- [x] 通用网页内容抓取与解析（readability + scraper）
- [x] AI 摘要 + 标签（Provider 抽象，支持云端/本地/离线）
- [x] SQLite 存储全部数据（rusqlite, WAL 模式）
- [x] Vue 基础 UI（链接列表、内容详情、设置页）
- [ ] 基础分类与标签管理

### v0.2 — 扩展采集
- [ ] 浏览器扩展（Chrome）
- [ ] 文档文件导入（PDF/Markdown）
- [ ] 自动化导入（书签、RSS）
- [ ] SPA 页面解析（Puppeteer）

### v0.3 — 视频与高级 AI
- [ ] 视频平台解析（YouTube/B站）
- [ ] 视频字幕提取 + AI 视频理解
- [ ] 笔记生成
- [ ] 语义搜索（向量数据库）

### v0.4 — 多端与备份
- [ ] 手机分享投送（PWA）
- [ ] WebDAV 备份/恢复
- [ ] MongoDB 迁移
- [ ] 社交媒体平台适配

## 开发

```bash
# 安装前端依赖
npm install

# 开发模式（Vite + Rust 后端热重载）
npm run tauri dev

# 构建
npm run tauri build
```

## License

MIT
