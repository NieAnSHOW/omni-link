# 日志系统实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 OmniLink 添加基于 tracing 的日志系统,支持按等级分文件、自动轮转和配置化管理

**Architecture:** 使用 tracing + tracing-appender 实现异步日志系统,在应用启动时初始化,按等级(ERROR/WARN/INFO/DEBUG)分文件存储到 `~/.omnilink/logs/`,支持 10MB 轮转和 5 个归档保留

**Tech Stack:** tracing 0.1, tracing-subscriber 0.3, tracing-appender 0.2

---

## 文件结构规划

**新增文件:**
- `src-tauri/src/logger/mod.rs` - 日志系统初始化和配置
- `src-tauri/src/logger/filter.rs` - 自定义日志过滤器,按等级路由

**修改文件:**
- `src-tauri/Cargo.toml` - 添加 tracing 相关依赖
- `src-tauri/src/lib.rs` - 添加 logger 模块声明,在 setup 中初始化日志
- `src-tauri/src/config.rs` - 添加 log_level 配置字段
- `src-tauri/src/commands/link_commands.rs` - 添加日志记录
- `src-tauri/src/parser/pipeline.rs` - 添加解析流程日志
- `src-tauri/src/ai/ai_service.rs` - 添加 AI 调用日志

---

### Task 1: 添加依赖项

**Files:**
- Modify: `src-tauri/Cargo.toml:28`

- [ ] **Step 1: 添加 tracing 依赖**

在 `[dependencies]` 部分添加:

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
tracing-appender = "0.2"
```

- [ ] **Step 2: 验证依赖添加成功**

Run: `cd src-tauri && cargo check`
Expected: 编译成功,无错误

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "build: 添加 tracing 日志依赖"
```

---

### Task 2: 扩展配置结构

**Files:**
- Modify: `src-tauri/src/config.rs:7-10`

- [ ] **Step 1: 添加 LogConfig 结构体**

在 `AppConfig` 结构体定义之前添加:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogConfig {
    pub level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            level: "DEBUG".to_string(),
        }
    }
}
```

- [ ] **Step 2: 在 AppConfig 中添加 log 字段**

修改 `AppConfig` 结构体:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub ai: AiConfig,
    #[serde(default)]
    pub log: LogConfig,
}
```

- [ ] **Step 3: 更新 AppConfig::default()**

修改 `impl Default for AppConfig`:

```rust
impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            ai: AiConfig {
                provider: "fallback".into(),
                openai: OpenAiConfig {
                    api_key: String::new(),
                    base_url: "https://api.openai.com/v1".into(),
                    model: "gpt-4o-mini".into(),
                },
                ollama: OllamaConfig {
                    base_url: "http://localhost:11434".into(),
                    model: "llama3.2".into(),
                },
            },
            log: LogConfig::default(),
        }
    }
}
```

- [ ] **Step 4: 验证配置结构编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/config.rs
git commit -m "feat(config): 添加日志配置结构"
```

---

### Task 3: 创建日志过滤器模块

**Files:**
- Create: `src-tauri/src/logger/filter.rs`

- [ ] **Step 1: 创建 filter.rs 文件**

创建文件并添加内容:

```rust
use tracing::Level;
use tracing_subscriber::filter::LevelFilter;

/// 将字符串日志等级转换为 LevelFilter
pub fn parse_log_level(level: &str) -> LevelFilter {
    match level.to_uppercase().as_str() {
        "ERROR" => LevelFilter::ERROR,
        "WARN" => LevelFilter::WARN,
        "INFO" => LevelFilter::INFO,
        "DEBUG" => LevelFilter::DEBUG,
        "TRACE" => LevelFilter::TRACE,
        _ => {
            eprintln!("Invalid log level '{}', using DEBUG", level);
            LevelFilter::DEBUG
        }
    }
}

/// 判断日志事件是否应该写入指定等级的文件
pub fn should_log_to_file(event_level: &Level, file_level: &Level) -> bool {
    event_level <= file_level
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_level() {
        assert_eq!(parse_log_level("ERROR"), LevelFilter::ERROR);
        assert_eq!(parse_log_level("error"), LevelFilter::ERROR);
        assert_eq!(parse_log_level("WARN"), LevelFilter::WARN);
        assert_eq!(parse_log_level("INFO"), LevelFilter::INFO);
        assert_eq!(parse_log_level("DEBUG"), LevelFilter::DEBUG);
        assert_eq!(parse_log_level("invalid"), LevelFilter::DEBUG);
    }

    #[test]
    fn test_should_log_to_file() {
        assert!(should_log_to_file(&Level::ERROR, &Level::ERROR));
        assert!(should_log_to_file(&Level::ERROR, &Level::DEBUG));
        assert!(!should_log_to_file(&Level::DEBUG, &Level::ERROR));
        assert!(should_log_to_file(&Level::INFO, &Level::INFO));
    }
}
```

- [ ] **Step 2: 运行测试验证过滤器逻辑**

Run: `cd src-tauri && cargo test logger::filter::tests`
Expected: 所有测试通过

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/logger/filter.rs
git commit -m "feat(logger): 添加日志等级过滤器"
```

---

### Task 4: 创建日志初始化模块

**Files:**
- Create: `src-tauri/src/logger/mod.rs`

- [ ] **Step 1: 创建 mod.rs 基础结构**

创建文件并添加模块声明和导入:

```rust
mod filter;

use crate::config::AppConfig;
use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// 获取日志目录路径
pub fn log_dir() -> PathBuf {
    crate::config::data_dir().join("logs")
}

/// 初始化日志系统
pub fn init_logger(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 创建日志目录
    let log_path = log_dir();
    std::fs::create_dir_all(&log_path)?;

    // 解析日志等级
    let log_level = filter::parse_log_level(&config.log.level);

    // 创建按等级分文件的 appender
    let error_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER) // 手动控制轮转
        .filename_prefix("error")
        .filename_suffix("log")
        .max_log_files(5)
        .build(&log_path)?;

    let warn_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("warn")
        .filename_suffix("log")
        .max_log_files(5)
        .build(&log_path)?;

    let info_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("info")
        .filename_suffix("log")
        .max_log_files(5)
        .build(&log_path)?;

    let debug_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("debug")
        .filename_suffix("log")
        .max_log_files(5)
        .build(&log_path)?;

    // 创建格式化层
    let error_layer = fmt::layer()
        .with_writer(error_appender)
        .with_ansi(false)
        .with_target(true)
        .with_filter(EnvFilter::new("error"));

    let warn_layer = fmt::layer()
        .with_writer(warn_appender)
        .with_ansi(false)
        .with_target(true)
        .with_filter(EnvFilter::new("warn"));

    let info_layer = fmt::layer()
        .with_writer(info_appender)
        .with_ansi(false)
        .with_target(true)
        .with_filter(EnvFilter::new("info"));

    let debug_layer = fmt::layer()
        .with_writer(debug_appender)
        .with_ansi(false)
        .with_target(true)
        .with_filter(log_level);

    // 组合所有层并初始化
    tracing_subscriber::registry()
        .with(error_layer)
        .with(warn_layer)
        .with(info_layer)
        .with(debug_layer)
        .init();

    tracing::info!("Logger initialized with level: {}", config.log.level);

    Ok(())
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/logger/mod.rs
git commit -m "feat(logger): 添加日志初始化模块"
```

---

### Task 5: 在 lib.rs 中集成日志系统

**Files:**
- Modify: `src-tauri/src/lib.rs:1-8`
- Modify: `src-tauri/src/lib.rs:15-28`

- [ ] **Step 1: 添加 logger 模块声明**

在模块声明部分添加:

```rust
mod ai;
mod commands;
mod config;
mod db;
mod error;
mod logger;
mod models;
mod parser;
mod repositories;
```

- [ ] **Step 2: 在 setup 中初始化日志系统**

修改 `run()` 函数中的 `.setup()` 部分:

```rust
.setup(|app| {
    let conn = db::database::init_connection()?;
    db::schema::init_schema(&conn)?;
    config::migrate_ai_config_from_db(&conn)?;
    let app_config = config::load_config()?;

    // 初始化日志系统
    if let Err(e) = logger::init_logger(&app_config) {
        eprintln!("Failed to initialize logger: {}", e);
    }

    app.manage(DbState(std::sync::Mutex::new(conn)));
    app.manage(ConfigState(std::sync::Mutex::new(app_config)));

    Ok(())
})
```

- [ ] **Step 3: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 4: 测试日志初始化**

Run: `cd src-tauri && cargo run`
Expected: 应用启动,在 `~/.omnilink/logs/` 目录下创建日志文件

- [ ] **Step 5: 验证日志文件创建**

Run: `ls -la ~/.omnilink/logs/`
Expected: 看到 error.log, warn.log, info.log, debug.log 文件

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(logger): 在应用启动时初始化日志系统"
```

---

### Task 6: 在链接命令中添加日志

**Files:**
- Modify: `src-tauri/src/commands/link_commands.rs`

- [ ] **Step 1: 在 create_link 中添加日志**

在 `create_link` 函数中添加日志记录:

```rust
#[tauri::command]
pub async fn create_link(
    url: String,
    db: tauri::State<'_, DbState>,
) -> Result<models::Link, String> {
    tracing::info!("Creating link: {}", url);
    
    let conn = db.0.lock().map_err(|e| {
        tracing::error!("Failed to acquire database lock: {}", e);
        format!("Database lock error: {}", e)
    })?;

    let link = repositories::link_repo::create_link(&conn, &url).map_err(|e| {
        tracing::error!("Failed to create link {}: {}", url, e);
        e.to_string()
    })?;

    tracing::info!("Link created successfully: id={}, url={}", link.id, link.url);
    Ok(link)
}
```

- [ ] **Step 2: 在 delete_link 中添加日志**

在 `delete_link` 函数中添加日志记录:

```rust
#[tauri::command]
pub async fn delete_link(
    id: i64,
    db: tauri::State<'_, DbState>,
) -> Result<(), String> {
    tracing::info!("Deleting link: id={}", id);
    
    let conn = db.0.lock().map_err(|e| {
        tracing::error!("Failed to acquire database lock: {}", e);
        format!("Database lock error: {}", e)
    })?;

    repositories::link_repo::delete_link(&conn, id).map_err(|e| {
        tracing::error!("Failed to delete link {}: {}", id, e);
        e.to_string()
    })?;

    tracing::info!("Link deleted successfully: id={}", id);
    Ok(())
}
```

- [ ] **Step 3: 在 get_links 中添加调试日志**

在 `get_links` 函数开始处添加:

```rust
#[tauri::command]
pub async fn get_links(
    db: tauri::State<'_, DbState>,
) -> Result<Vec<models::Link>, String> {
    tracing::debug!("Fetching all links");
    
    let conn = db.0.lock().map_err(|e| {
        tracing::error!("Failed to acquire database lock: {}", e);
        format!("Database lock error: {}", e)
    })?;

    let links = repositories::link_repo::get_all_links(&conn).map_err(|e| {
        tracing::error!("Failed to fetch links: {}", e);
        e.to_string()
    })?;

    tracing::debug!("Fetched {} links", links.len());
    Ok(links)
}
```

- [ ] **Step 4: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 5: 测试日志记录**

Run: `cd src-tauri && cargo run`
然后在应用中创建、查看、删除链接

- [ ] **Step 6: 验证日志内容**

Run: `tail -f ~/.omnilink/logs/info.log`
Expected: 看到链接操作的日志记录

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands/link_commands.rs
git commit -m "feat(logger): 在链接命令中添加日志记录"
```

---

### Task 7: 在解析流程中添加日志

**Files:**
- Modify: `src-tauri/src/parser/pipeline.rs`

- [ ] **Step 1: 读取当前 pipeline.rs 文件**

Run: `cat src-tauri/src/parser/pipeline.rs`
Expected: 了解当前解析流程的结构

- [ ] **Step 2: 在解析开始处添加日志**

在解析函数开始处添加:

```rust
tracing::info!("Starting content parsing for URL: {}", url);
tracing::debug!("Parser pipeline initialized");
```

- [ ] **Step 3: 在平台识别后添加日志**

在平台识别逻辑后添加:

```rust
tracing::debug!("Platform identified: {:?}", platform_type);
```

- [ ] **Step 4: 在内容提取成功后添加日志**

在内容提取成功后添加:

```rust
tracing::info!("Content extracted successfully: url={}, length={}", url, content.len());
tracing::debug!("Content preview: {}...", &content[..content.len().min(100)]);
```

- [ ] **Step 5: 在解析失败时添加错误日志**

在错误处理部分添加:

```rust
tracing::error!("Failed to parse content from {}: {}", url, error);
```

- [ ] **Step 6: 在降级处理时添加警告日志**

如果有降级逻辑,添加:

```rust
tracing::warn!("Falling back to alternative parser for {}: {}", url, reason);
```

- [ ] **Step 7: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 8: 测试解析日志**

Run: `cd src-tauri && cargo run`
然后在应用中解析一个链接

- [ ] **Step 9: 验证日志内容**

Run: `tail -f ~/.omnilink/logs/info.log ~/.omnilink/logs/debug.log`
Expected: 看到解析流程的详细日志

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/parser/pipeline.rs
git commit -m "feat(logger): 在解析流程中添加日志记录"
```

---

### Task 8: 在 AI 服务中添加日志

**Files:**
- Modify: `src-tauri/src/ai/ai_service.rs`

- [ ] **Step 1: 在 AI 调用开始处添加日志**

在 AI 处理函数开始处添加:

```rust
tracing::info!("Starting AI processing for content");
tracing::debug!("AI provider: {}, content_length: {}", provider, content.len());
```

- [ ] **Step 2: 在 API 调用前添加调试日志**

在实际调用 API 前添加:

```rust
tracing::debug!("Calling AI API: model={}, max_tokens={}", model, max_tokens);
```

- [ ] **Step 3: 在 API 调用成功后添加日志**

在成功获取响应后添加:

```rust
tracing::info!("AI processing completed successfully");
tracing::debug!("AI response length: {}", response.len());
```

- [ ] **Step 4: 在 API 调用失败时添加错误日志**

在错误处理部分添加:

```rust
tracing::error!("AI API call failed: {}", error);
```

- [ ] **Step 5: 在 Provider 切换时添加警告日志**

如果有 provider 切换逻辑,添加:

```rust
tracing::warn!("Switching AI provider from {} to {}: {}", old_provider, new_provider, reason);
```

- [ ] **Step 6: 在降级到规则引擎时添加警告日志**

在降级到 fallback provider 时添加:

```rust
tracing::warn!("Falling back to rule-based engine: {}", reason);
```

- [ ] **Step 7: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 8: 测试 AI 日志**

Run: `cd src-tauri && cargo run`
然后在应用中触发 AI 处理

- [ ] **Step 9: 验证日志内容**

Run: `tail -f ~/.omnilink/logs/info.log ~/.omnilink/logs/debug.log`
Expected: 看到 AI 处理的详细日志

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/ai/ai_service.rs
git commit -m "feat(logger): 在 AI 服务中添加日志记录"
```

---

### Task 9: 在数据库操作中添加调试日志

**Files:**
- Modify: `src-tauri/src/repositories/link_repo.rs`

- [ ] **Step 1: 在 SQL 执行前添加调试日志**

在关键 SQL 执行前添加(仅在 DEBUG 等级):

```rust
tracing::debug!("Executing SQL: INSERT INTO links (url, created_at) VALUES (?, ?)");
```

- [ ] **Step 2: 在数据库错误时添加错误日志**

在错误处理部分添加:

```rust
tracing::error!("Database error in create_link: {}", error);
```

- [ ] **Step 3: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/repositories/link_repo.rs
git commit -m "feat(logger): 在数据库操作中添加调试日志"
```

---

### Task 10: 添加应用生命周期日志

**Files:**
- Modify: `src-tauri/src/lib.rs:15-28`

- [ ] **Step 1: 在应用启动时记录版本信息**

在 `setup` 函数中日志初始化后添加:

```rust
tracing::info!("OmniLink application starting");
tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));
tracing::info!("Config loaded from: {:?}", config::config_path());
```

- [ ] **Step 2: 在数据库初始化后添加日志**

在 `init_schema` 调用后添加:

```rust
tracing::info!("Database initialized successfully");
```

- [ ] **Step 3: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 4: 测试启动日志**

Run: `cd src-tauri && cargo run`

- [ ] **Step 5: 验证启动日志**

Run: `cat ~/.omnilink/logs/info.log`
Expected: 看到应用启动、版本号、配置路径等信息

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(logger): 添加应用生命周期日志"
```

---

### Task 11: 实现日志轮转功能

**Files:**
- Modify: `src-tauri/src/logger/mod.rs`

- [ ] **Step 1: 更新 RollingFileAppender 配置**

修改 appender 创建代码,添加文件大小限制:

```rust
let error_appender = RollingFileAppender::builder()
    .rotation(Rotation::NEVER)
    .filename_prefix("error")
    .filename_suffix("log")
    .max_log_files(5)
    .max_file_size(10 * 1024 * 1024) // 10MB
    .build(&log_path)?;

let warn_appender = RollingFileAppender::builder()
    .rotation(Rotation::NEVER)
    .filename_prefix("warn")
    .filename_suffix("log")
    .max_log_files(5)
    .max_file_size(10 * 1024 * 1024)
    .build(&log_path)?;

let info_appender = RollingFileAppender::builder()
    .rotation(Rotation::NEVER)
    .filename_prefix("info")
    .filename_suffix("log")
    .max_log_files(5)
    .max_file_size(10 * 1024 * 1024)
    .build(&log_path)?;

let debug_appender = RollingFileAppender::builder()
    .rotation(Rotation::NEVER)
    .filename_prefix("debug")
    .filename_suffix("log")
    .max_log_files(5)
    .max_file_size(10 * 1024 * 1024)
    .build(&log_path)?;
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/logger/mod.rs
git commit -m "feat(logger): 实现日志文件轮转功能"
```

---

### Task 12: 测试日志系统完整功能

**Files:**
- Test: 整个日志系统

- [ ] **Step 1: 清理旧日志文件**

Run: `rm -rf ~/.omnilink/logs/*`
Expected: 日志目录清空

- [ ] **Step 2: 启动应用**

Run: `cd src-tauri && cargo run`
Expected: 应用正常启动

- [ ] **Step 3: 验证日志文件创建**

Run: `ls -lh ~/.omnilink/logs/`
Expected: 看到 4 个日志文件(error.log, warn.log, info.log, debug.log)

- [ ] **Step 4: 执行各种操作**

在应用中执行:
- 创建链接
- 解析内容
- AI 处理
- 删除链接

- [ ] **Step 5: 验证 INFO 日志内容**

Run: `cat ~/.omnilink/logs/info.log`
Expected: 看到应用启动、链接操作、解析成功、AI 处理等日志

- [ ] **Step 6: 验证 DEBUG 日志内容**

Run: `cat ~/.omnilink/logs/debug.log`
Expected: 看到详细的调试信息(SQL 语句、API 调用参数等)

- [ ] **Step 7: 测试日志等级配置**

修改 `~/.omnilink/config.json`,将 `log.level` 改为 "INFO"

- [ ] **Step 8: 重启应用验证等级过滤**

Run: `cd src-tauri && cargo run`
然后执行操作

- [ ] **Step 9: 验证 DEBUG 日志不再记录**

Run: `tail ~/.omnilink/logs/debug.log`
Expected: 没有新的 DEBUG 日志(因为等级设置为 INFO)

- [ ] **Step 10: 恢复 DEBUG 等级**

将 `~/.omnilink/config.json` 中的 `log.level` 改回 "DEBUG"

---

### Task 13: 测试日志轮转功能

**Files:**
- Test: 日志轮转

- [ ] **Step 1: 创建测试脚本生成大量日志**

创建临时测试代码,在某个命令中循环记录大量日志:

```rust
for i in 0..100000 {
    tracing::info!("Test log message {}: {}", i, "x".repeat(100));
}
```

- [ ] **Step 2: 运行测试**

Run: `cd src-tauri && cargo run`
触发包含测试代码的命令

- [ ] **Step 3: 验证日志文件轮转**

Run: `ls -lh ~/.omnilink/logs/`
Expected: 看到 info.log 和归档文件 info.log.1, info.log.2 等

- [ ] **Step 4: 验证归档文件数量限制**

Run: `ls ~/.omnilink/logs/info.log* | wc -l`
Expected: 最多 6 个文件(1 个当前 + 5 个归档)

- [ ] **Step 5: 移除测试代码**

删除步骤 1 中添加的测试代码

- [ ] **Step 6: Commit**

```bash
git add .
git commit -m "test: 验证日志轮转功能"
```

---

### Task 14: 更新文档

**Files:**
- Modify: `docs/omni-link/P0-Future.md`

- [ ] **Step 1: 更新 P0-Future.md**

将"新增操作日志"标记为已完成:

```markdown
# P0-Future

## 新功能规划

### ✅ 新增操作日志

src-tauri 后端服务日志按照等级划分输出至应用目录 /logs 文件夹中

**实现状态**: 已完成
**实现日期**: 2026-04-21
**相关文档**: 
- 设计文档: `docs/superpowers/specs/2026-04-21-logging-system-design.md`
- 实施计划: `docs/superpowers/plans/2026-04-21-logging-system.md`

**功能说明**:
- 支持 ERROR/WARN/INFO/DEBUG 四级日志
- 按等级分文件存储到 `~/.omnilink/logs/`
- 单文件 10MB 自动轮转,保留 5 个归档
- 通过 `~/.omnilink/config.json` 配置日志等级
- 记录链接管理、内容解析、AI 处理、数据库操作等关键操作
```

- [ ] **Step 2: Commit**

```bash
git add docs/omni-link/P0-Future.md
git commit -m "docs: 标记日志系统功能为已完成"
```

---

### Task 15: 最终验证和清理

**Files:**
- Test: 完整系统

- [ ] **Step 1: 完整编译检查**

Run: `cd src-tauri && cargo build --release`
Expected: 编译成功,无警告

- [ ] **Step 2: 运行完整测试套件**

Run: `cd src-tauri && cargo test`
Expected: 所有测试通过

- [ ] **Step 3: 启动应用进行手动测试**

Run: `cd src-tauri && cargo run`
执行完整的用户流程:
1. 添加链接
2. 解析内容
3. AI 处理
4. 查看链接列表
5. 删除链接

- [ ] **Step 4: 验证所有日志文件**

Run: 
```bash
echo "=== ERROR LOG ===" && cat ~/.omnilink/logs/error.log
echo "=== WARN LOG ===" && cat ~/.omnilink/logs/warn.log
echo "=== INFO LOG ===" && cat ~/.omnilink/logs/info.log
echo "=== DEBUG LOG ===" && tail -50 ~/.omnilink/logs/debug.log
```
Expected: 各等级日志内容正确,格式统一

- [ ] **Step 5: 验证配置文件**

Run: `cat ~/.omnilink/config.json`
Expected: 包含 `log` 字段,默认值为 `{"level": "DEBUG"}`

- [ ] **Step 6: 最终 Commit**

```bash
git add .
git commit -m "feat: 完成日志系统实现"
```

---

## 验收标准

完成以上所有任务后,系统应满足:

1. ✅ 日志文件按等级分别存储在 `~/.omnilink/logs/` 目录
2. ✅ 支持 ERROR/WARN/INFO/DEBUG 四级日志
3. ✅ 日志格式为: 时间戳 + 等级 + 模块 + 消息
4. ✅ 单文件达到 10MB 自动轮转,保留 5 个归档
5. ✅ 通过配置文件控制日志等级,默认 DEBUG
6. ✅ 记录关键操作:链接管理、内容解析、AI 处理、数据库操作
7. ✅ 应用启动时记录版本号和配置信息
8. ✅ 错误情况下记录详细的错误堆栈
9. ✅ 日志写入失败不影响业务逻辑
10. ✅ 编译无警告,所有测试通过

## 注意事项

1. **敏感信息保护**: 不要记录完整的 API Key、密码等敏感信息
2. **性能考虑**: 日志写入是异步的,不会阻塞主线程
3. **磁盘空间**: 总磁盘占用最多 240MB(4 等级 × 6 文件 × 10MB)
4. **错误处理**: 日志初始化失败时打印到 stderr,但不阻断应用启动

