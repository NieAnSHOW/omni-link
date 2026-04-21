# OmniLink MVP (v0.1) 实现计划

> **架构变更说明**: 原计划采用 Tauri + Node.js Sidecar（Fastify HTTP Server），实际实现改为纯 Tauri + Rust 后端。所有业务逻辑在 Tauri 进程内通过 Rust 实现，前端通过 IPC `invoke()` 调用 Tauri Commands。本计划已更新以反映实际实现。

**Goal:** 实现跨平台链接采集 + 内容解析 + AI 摘要的桌面应用 MVP

**Architecture:** Tauri v2 桌面应用，Vue 3 前端通过 IPC 与 Rust 后端通信。Rust 后端处理所有业务逻辑（数据库、内容解析、AI 分析）。SQLite 作为 MVP 唯一存储。

**Tech Stack:** Tauri v2 + Vue 3 + TypeScript + Vite 6 + Rust (rusqlite + reqwest + readability + scraper)

---

## 文件结构

```
omni-link/
├── package.json                        # 前端依赖 + Tauri CLI
├── vite.config.ts
├── tsconfig.json
├── index.html                          # Vite 入口
├── src/                                # Vue 3 前端
│   ├── main.ts                         # Vue 入口（Pinia + Router）
│   ├── App.vue
│   ├── styles.css
│   ├── router/index.ts                 # Vue Router（hash history）
│   ├── views/
│   │   ├── LinksView.vue               # 链接列表页
│   │   ├── ContentView.vue             # 内容详情页（AI 摘要）
│   │   └── SettingsView.vue            # 设置页（AI 配置）
│   ├── components/
│   │   ├── AppLayout.vue               # 主布局（侧边栏+内容区）
│   │   ├── AddLinkDialog.vue           # 添加链接弹窗
│   │   └── LinkCard.vue                # 链接卡片
│   ├── composables/
│   │   └── useApi.ts                   # Tauri IPC API 封装
│   ├── stores/
│   │   └── links.ts                    # Pinia 状态管理
│   └── types/
│       └── index.ts                    # 共享类型定义
├── src-tauri/                          # Tauri v2 Rust 后端
│   ├── Cargo.toml                      # Rust 依赖
│   ├── tauri.conf.json                 # Tauri 配置
│   ├── capabilities/
│   │   └── default.json                # 权限（core:default, opener:default）
│   ├── src/
│   │   ├── main.rs                     # Rust 入口
│   │   ├── lib.rs                      # Tauri app setup + command 注册
│   │   ├── config.rs                   # 应用配置（~/.omnilink/config.json）
│   │   ├── error.rs                    # 错误类型（thiserror）
│   │   ├── models.rs                   # 数据模型（Link, Content, AiResult 等）
│   │   ├── ai/
│   │   │   ├── mod.rs
│   │   │   ├── ai_service.rs           # AI 服务编排（Provider 路由 + 自动降级）
│   │   │   ├── openai_provider.rs      # OpenAI 兼容 API
│   │   │   ├── ollama_provider.rs      # Ollama 本地模型
│   │   │   └── fallback_provider.rs    # 离线规则引擎（正则 + 关键词提取）
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── link_commands.rs        # 链接 CRUD + 解析触发
│   │   │   ├── content_commands.rs     # 内容详情 + AI 分析
│   │   │   └── settings_commands.rs    # AI 配置读写
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── database.rs             # SQLite 连接（WAL, ~/.omnilink/）
│   │   │   └── schema.rs               # 建表语句（8 表 + 4 索引）
│   │   ├── parser/
│   │   │   ├── mod.rs
│   │   │   ├── identifier.rs           # 平台识别（URL 正则匹配）
│   │   │   ├── fetcher.rs              # HTTP 页面抓取（reqwest）
│   │   │   ├── extractor.rs            # 正文提取（readability + scraper fallback）
│   │   │   └── pipeline.rs             # 解析流水线编排
│   │   └── repositories/
│   │       ├── mod.rs
│   │       ├── link_repo.rs            # 链接数据访问
│   │       ├── content_repo.rs         # 内容数据访问
│   │       ├── ai_result_repo.rs       # AI 结果数据访问
│   │       └── settings_repo.rs        # 设置数据访问
│   └── icons/                          # 应用图标
```

---

## 已完成的功能

### 数据库层（Rust SQLite）
- [x] SQLite 连接管理（WAL 模式，`~/.omnilink/omnilink.db`）
- [x] Schema：8 张表（links, contents, tags, content_tags, categories, ai_results, user_settings, import_history）
- [x] 4 个索引（status, platform, created_at, contents.link_id）
- [x] Repository 层：link, content, ai_result, settings 各自的 CRUD

### 内容解析引擎
- [x] 平台识别：正则匹配 YouTube, Bilibili, Twitter/X, Weibo, Zhihu, Juejin, GitHub
- [x] 页面抓取：reqwest + 浏览器 UA + 15s 超时
- [x] 正文提取：readability 主提取 + scraper 回退 + 图片/元数据提取
- [x] 解析流水线：平台识别 → 抓取 → 提取 → 状态更新 → 存储

### AI 能力层
- [x] Provider 抽象：统一接口（summary + tags）
- [x] OpenAI Provider：reqwest 调用 /chat/completions，支持自定义 base URL
- [x] Ollama Provider：reqwest 调用 /api/generate
- [x] Fallback Provider：规则引擎（前 3 句摘要 + 停用词过滤关键词提取）
- [x] AI 服务编排：路由到配置 Provider，失败自动降级到 Fallback

### Tauri Commands（10 个）
- [x] `get_links` — 分页链接列表（支持 status 筛选）
- [x] `create_link` — 创建/批量创建链接（URL 去重）
- [x] `get_link` — 单个链接查询
- [x] `delete_link` — 删除链接
- [x] `parse_link_cmd` — 触发内容解析流水线
- [x] `get_link_detail` — 链接 + 内容 + AI 结果组合查询
- [x] `analyze_content_cmd` — AI 内容分析
- [x] `get_settings` — 获取所有设置
- [x] `get_ai_config` — 获取 AI Provider 配置
- [x] `update_ai_config` — 更新 AI Provider 配置

### 前端 UI
- [x] 路由：`/links`（列表）、`/links/:id`（详情）、`/settings`（设置）
- [x] AppLayout：侧边栏 + 内容区
- [x] LinksView：筛选标签、链接卡片列表、添加弹窗
- [x] ContentView：内容正文、AI 摘要、解析/分析按钮
- [x] SettingsView：AI Provider 选择 + 配置表单
- [x] useApi composable：Tauri IPC 封装

---

## 待完成的功能

### P0 — 基础分类与标签管理
- [ ] 标签自动创建（AI 分析时自动入库）
- [ ] 标签列表展示与管理
- [ ] 手动标签编辑
- [ ] 分类树（支持嵌套 parent_id）

### P1 — 体验优化
- [ ] 解析进度实时反馈（Tauri Events 推送）
- [ ] 链接搜索（URL / 标题模糊搜索）
- [ ] 错误重试（解析失败自动/手动重试）
- [ ] 数据导出（JSON / CSV）

---

## Rust 技术栈详情

| Crate | 版本 | 用途 |
|-------|------|------|
| tauri | 2 | 桌面应用框架 |
| rusqlite | 0.32 (bundled) | SQLite 驱动 |
| reqwest | 0.12 (rustls-tls) | HTTP 客户端 |
| readability | 0.3 | 正文提取（Mozilla Readability Rust 移植） |
| scraper | 0.22 | HTML 解析（CSS 选择器） |
| regex | 1 | 正则匹配（平台识别） |
| serde / serde_json | 1 | 序列化/反序列化 |
| tokio | 1 (full) | 异步运行时 |
| thiserror | 2 | 错误类型派生 |
| dirs | 6 | 平台目录 |
| chrono | 0.4 | 日期时间 |
| tauri-plugin-opener | 2 | 外部 URL 打开 |

---

## 验证清单

| 功能 | 验证方式 |
|------|---------|
| 应用启动 | `npm run tauri dev` 打开窗口 |
| 添加链接 | 输入 URL → 列表显示新链接 |
| 内容解析 | 点击解析 → 状态变为已解析 → 详情页显示正文 |
| AI 摘要 | 配置 Provider → 点击分析 → 显示摘要和标签 |
| 设置持久化 | 重启应用后 AI 配置仍保留（~/.omnilink/config.json） |
| 原文跳转 | 详情页点击"查看原文" → 浏览器打开链接 |
