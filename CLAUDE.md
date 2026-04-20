# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

OmniLink — 跨平台链接采集 + 内容解析 + AI 知识结构化的桌面应用（Tauri v2 + Vue 3 + Rust）。当前处于 v0.1 MVP 阶段，UI 语言为中文。

## Development Commands

```bash
npm install              # 安装前端依赖
npm run tauri dev        # 开发模式（Vite + Rust 后端热重载）
npm run tauri build      # 生产构建
npm run dev              # 仅前端 Vite dev server（端口 1420）
npm run build            # 前端类型检查 + Vite 构建（不含 Rust）
```

无测试框架、无 linter、无 formatter 配置。

## Architecture

前端通过 Tauri IPC `invoke()` 调用 Rust 后端的 Tauri Commands，无 Sidecar 额外进程。

### Frontend (`src/`)

- **路由**：Vue Router，hash 模式（Tauri 必须），路径 `/links`、`/links/:id`、`/tags`、`/settings`
- **状态管理**：Pinia stores（Composition API 风格）— `useLinksStore`、`useTagsStore`、`useCategoriesStore`
- **API 层**：`composables/useApi.ts` 封装所有 `invoke()` 调用，类型对应 `types/index.ts`
- **样式**：纯 scoped CSS，无框架/Tailwind，主色调 `#6366f1`
- TypeScript strict 模式，`noUnusedLocals` + `noUnusedParameters` 启用

### Backend (`src-tauri/src/`)

分层架构：

- **commands/** — Tauri 命令入口（link、content、settings、tag、category），在 `lib.rs` 注册
- **repositories/** — 每个 entity 一个 `_repo.rs`，纯 SQL（rusqlite 参数化查询，无 ORM）
- **parser/** — 解析流水线：`identifier.rs`（URL 平台识别）→ `fetcher.rs`（HTTP 抓取）→ `extractor.rs`（readability + scraper 正文提取）→ `pipeline.rs`（编排）
- **ai/** — Provider 模式：`ai_service.rs` 调度 → `openai_provider.rs` / `ollama_provider.rs` / `fallback_provider.rs`（规则引擎降级）
- **db/** — SQLite 初始化（WAL 模式、外键启用）和 schema 管理（`schema.rs` 含建表 + 迁移）
- **models.rs** — 数据模型，前端 `types/index.ts` 需与之保持一致

### Database

SQLite 存储于 `~/.omnilink/omnilink.db`，7 张表：links、contents、tags、content_tags、categories、ai_results、user_settings、import_history。Schema 和迁移在 `db/schema.rs`。

配置文件位于 `~/.omnilink/config.json`。

## Key Conventions

- 新增 Tauri command 需在 `lib.rs` 的 `.invoke_handler(tauri::generate_handler![...])` 中注册
- 前端 TypeScript 类型（`types/index.ts`）必须与 Rust 模型（`models.rs`）保持同步
- 所有前端调用通过 `composables/useApi.ts`，不直接使用 `invoke()`
- Rust 依赖使用 rustls-tls（无 OpenSSL），crate type 为 `staticlib` + `cdylib` + `rlib`
