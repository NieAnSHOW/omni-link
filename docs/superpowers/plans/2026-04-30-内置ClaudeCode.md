# 内置 Claude Code 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 OmniLink 内置 Claude Code CLI，提供可交互式底部面板终端和内置 Skills 系统。

**Architecture:** 新建 `claude` 模块（CliDownloader + ClaudeManager + SkillsManager），与现有 `terminal` 模块（persona rewrite）平行运作。前端新增 BottomPanel + InteractiveTerminal 底部面板，集成到 AppLayout。

**Tech Stack:** Rust (portable_pty, reqwest, serde), Vue 3 (xterm.js, Composition API), Tauri v2 IPC

> **注意：** 项目无测试框架（per CLAUDE.md），因此任务中不包含测试步骤。每个任务以编译验证 + 手动验证代替。

---

## File Structure

### Backend (Rust) — 新增

| 文件 | 职责 |
|------|------|
| `src-tauri/src/claude/mod.rs` | 模块入口，re-export |
| `src-tauri/src/claude/cli_downloader.rs` | 镜像源管理、CLI 下载、版本检测 |
| `src-tauri/src/claude/manager.rs` | PTY 会话管理、环境变量注入 |
| `src-tauri/src/claude/skills.rs` | Skills 部署到 ~/.omnilink/skills/ |
| `src-tauri/src/commands/claude_commands.rs` | Tauri 命令入口 |

### Backend (Rust) — 修改

| 文件 | 改动 |
|------|------|
| `src-tauri/src/config.rs` | 新增 ClaudeConfig 结构体，扩展 AppConfig |
| `src-tauri/src/lib.rs` | 注册 claude 模块 + 新命令 + 新 State |

### Frontend (Vue) — 新增

| 文件 | 职责 |
|------|------|
| `src/components/layout/BottomPanel.vue` | 底部面板容器，拖拽分割条 |
| `src/components/claude/InteractiveTerminal.vue` | 可交互 xterm.js 终端 |
| `src/components/settings/ClaudeSettings.vue` | Claude Code 配置区块 |
| `src/composables/useClaude.ts` | Claude IPC 封装 |

### Frontend (Vue) — 修改

| 文件 | 改动 |
|------|------|
| `src/types/persona.ts` | 新增 ClaudeConfig、ClaudeSession 类型 |
| `src/composables/useApi.ts` | 新增 Claude 配置 API |
| `src/views/SettingsView.vue` | 集成 ClaudeSettings 组件 |
| `src/components/AppLayout.vue` | 集成 BottomPanel |

---

### Task 1: 扩展 Rust 配置结构体

**Files:**
- Modify: `src-tauri/src/config.rs:37-83`

- [ ] **Step 1: 在 config.rs 中添加 ClaudeConfig 结构体**

在 `OllamaConfig` 结构体（第 63 行）之后添加：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaudeConfig {
    #[serde(default = "default_claude_provider")]
    pub provider: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_claude_base_url")]
    pub base_url: String,
    #[serde(default = "default_claude_model")]
    pub model: String,
}

fn default_claude_provider() -> String {
    "anthropic".into()
}

fn default_claude_base_url() -> String {
    "https://api.anthropic.com".into()
}

fn default_claude_model() -> String {
    "claude-sonnet-4-20250514".into()
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        ClaudeConfig {
            provider: default_claude_provider(),
            api_key: String::new(),
            base_url: default_claude_base_url(),
            model: default_claude_model(),
        }
    }
}
```

- [ ] **Step 2: 扩展 AppConfig 添加 claude_code 字段**

将 `AppConfig` 结构体（第 37-42 行）修改为：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub ai: AiConfig,
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub claude_code: ClaudeConfig,
}
```

`Default` 实现中也需要添加 `claude_code: ClaudeConfig::default()`。

- [ ] **Step 3: 编译验证**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: 编译成功（可能有 unused warnings，无 error）

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/config.rs
git commit -m "feat(config): 添加 ClaudeConfig 结构体，扩展 AppConfig"
```

---

### Task 2: 创建 claude 模块骨架 + Skills Manager

**Files:**
- Create: `src-tauri/src/claude/mod.rs`
- Create: `src-tauri/src/claude/skills.rs`
- Modify: `src-tauri/src/lib.rs:1-11`

- [ ] **Step 1: 创建 claude/mod.rs**

```rust
pub mod cli_downloader;
pub mod manager;
pub mod skills;
```

- [ ] **Step 2: 创建 claude/skills.rs**

```rust
use std::fs;
use std::path::PathBuf;
use crate::error::AppResult;

pub fn skills_deploy_dir() -> PathBuf {
    crate::config::data_dir().join("skills")
}

pub fn deploy_skills() -> AppResult<()> {
    let target_dir = skills_deploy_dir();
    fs::create_dir_all(&target_dir)?;

    let builtin_skills = [
        ("fangqikiki-perspective.md", include_str!("../../skills/fangqikiki-perspective.md")),
        ("leitanzhang-perspective.md", include_str!("../../skills/leitanzhang-perspective.md")),
        ("huashu-perspective.md", include_str!("../../skills/huashu-perspective.md")),
        ("nuwa-skill.md", include_str!("../../skills/nuwa-skill.md")),
    ];

    for (filename, content) in builtin_skills {
        let target_path = target_dir.join(filename);
        fs::write(&target_path, content)?;
    }

    tracing::info!("Skills deployed to {}", target_dir.display());
    Ok(())
}
```

- [ ] **Step 3: 创建空骨架 cli_downloader.rs 和 manager.rs**

`src-tauri/src/claude/cli_downloader.rs`:
```rust
// CLI 下载管理 — Task 3 实现
```

`src-tauri/src/claude/manager.rs`:
```rust
// Claude 会话管理 — Task 5 实现
```

- [ ] **Step 4: 在 lib.rs 注册 claude 模块并在 setup 中调用 deploy_skills**

在 `lib.rs` 第 11 行后添加 `mod claude;`，在 setup 函数中 `persona::initialize_builtin_skills()` 之后添加 `claude::skills::deploy_skills()?;`。

- [ ] **Step 5: 编译验证**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: 编译成功

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/claude/ src-tauri/src/lib.rs
git commit -m "feat(claude): 创建 claude 模块骨架 + Skills 部署"
```

---

### Task 3: 实现 CLI 下载器

**Files:**
- Modify: `src-tauri/src/claude/cli_downloader.rs`

- [ ] **Step 1: 实现镜像源列表和 CLI 路径管理**

将 `cli_downloader.rs` 替换为完整实现：

```rust
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use crate::config::data_dir;
use crate::error::{AppError, AppResult};

pub struct CliDownloader {
    pub active_mirror: Mutex<Option<String>>,
}

impl CliDownloader {
    pub fn new() -> Self {
        Self {
            active_mirror: Mutex::new(None),
        }
    }
}

pub fn cli_base_dir() -> PathBuf {
    data_dir().join("claude-cli")
}

pub fn current_cli_path() -> PathBuf {
    cli_base_dir().join("current/claude")
}

pub fn is_installed() -> bool {
    let path = current_cli_path();
    if !path.exists() {
        return false;
    }
    std::process::Command::new(path.as_os_str())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_installed_version() -> Option<String> {
    let path = current_cli_path();
    if !path.exists() {
        return None;
    }
    std::process::Command::new(path.as_os_str())
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|v| v.trim().to_string())
}

fn mirror_urls() -> Vec<(&'static str, String)> {
    let version = "1.0.42"; // 后续迭代：从 GitHub API 获取最新版本号
    let binary = if cfg!(target_os = "macos") {
        "claude-macos-x64"
    } else if cfg!(target_os = "linux") {
        "claude-linux-x64"
    } else {
        "claude-windows-x64.exe"
    };
    let release_path = format!(
        "/anthropics/claude-code/releases/download/cli-v{}/{}",
        version, binary
    );
    vec![
        ("ghfast.top", format!("https://ghfast.top{}", release_path)),
        ("ghproxy.net", format!("https://ghproxy.net{}", release_path)),
        ("gh-proxy.com", format!("https://gh-proxy.com{}", release_path)),
        ("github.com", format!("https://github.com{}", release_path)),
    ]
}

pub async fn download_cli(
    downloader: &CliDownloader,
    app_handle: tauri::AppHandle,
) -> AppResult<PathBuf> {
    let base_dir = cli_base_dir();
    let version = "1.0.42";
    let version_dir = base_dir.join(format!("v{}", version));
    fs::create_dir_all(&version_dir)?;

    let cli_path = version_dir.join("claude");

    if cli_path.exists() {
        let current_link = base_dir.join("current");
        let _ = fs::remove_file(&current_link);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&version_dir, &current_link)?;
        return Ok(cli_path);
    }

    let mirrors = mirror_urls();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| AppError::Parse(e.to_string()))?;

    let mut last_error = None;

    for (name, url) in &mirrors {
        {
            let mut active = downloader.active_mirror.lock().unwrap();
            *active = Some(name.to_string());
        }
        let _ = app_handle.emit("cli-download-progress", serde_json::json!({
            "status": "downloading",
            "mirror": name,
            "url": url,
        }));

        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let bytes = resp.bytes().await.map_err(|e| AppError::Parse(e.to_string()))?;
                fs::write(&cli_path, &bytes)?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&cli_path, fs::Permissions::from_mode(0o755))?;
                }

                let current_link = base_dir.join("current");
                let _ = fs::remove_file(&current_link);
                #[cfg(unix)]
                std::os::unix::fs::symlink(&version_dir, &current_link)?;

                let _ = app_handle.emit("cli-download-progress", serde_json::json!({
                    "status": "completed",
                    "mirror": name,
                }));

                tracing::info!("Claude CLI downloaded from {} to {}", name, cli_path.display());
                return Ok(cli_path);
            }
            Ok(resp) => {
                last_error = Some(format!("{} returned {}", name, resp.status()));
            }
            Err(e) => {
                last_error = Some(format!("{} failed: {}", name, e));
            }
        }
    }

    let _ = app_handle.emit("cli-download-progress", serde_json::json!({
        "status": "failed",
        "error": last_error.clone(),
    }));

    Err(AppError::Parse(format!(
        "所有镜像源均不可用: {}",
        last_error.unwrap_or_default()
    )))
}
```

- [ ] **Step 2: 编译验证**

Run: `cd src-tauri && cargo check 2>&1 | tail -10`
Expected: 编译成功（reqwest 已在依赖中）

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/claude/cli_downloader.rs
git commit -m "feat(claude): 实现 CLI 下载器（多源自动切换）"
```

---

### Task 4: 实现 Claude Manager（PTY 会话管理）

**Files:**
- Modify: `src-tauri/src/claude/manager.rs`

- [ ] **Step 1: 实现 ClaudeManager**

将 `manager.rs` 替换为完整实现：

```rust
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

use crate::claude::cli_downloader::current_cli_path;
use crate::claude::skills::skills_deploy_dir;
use crate::config::load_config;
use crate::error::{AppError, AppResult};

pub struct ClaudeSession {
    pub id: String,
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    pub child: Box<dyn portable_pty::Child + Send + Sync>,
    pub reader: Option<Box<dyn std::io::Read + Send>>,
    pub reader_handle: Option<std::thread::JoinHandle<()>>,
}

pub struct ClaudeManager {
    sessions: Arc<Mutex<HashMap<String, ClaudeSession>>>,
}

impl ClaudeManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_session(
        &self,
        app_handle: tauri::AppHandle,
        session_id: String,
    ) -> AppResult<()> {
        let cli_path = current_cli_path();
        if !cli_path.exists() {
            return Err(AppError::Parse("Claude CLI 未安装".into()));
        }

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::Parse(e.to_string()))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| AppError::Parse(e.to_string()))?;

        let skills_dir = skills_deploy_dir();
        let config = load_config()?;

        let mut cmd = CommandBuilder::new(cli_path.as_os_str());
        if skills_dir.exists() {
            cmd.arg("--add-dir");
            cmd.arg(skills_dir.as_os_str());
        }

        cmd.env("ANTHROPIC_API_KEY", &config.claude_code.api_key);
        if !config.claude_code.base_url.is_empty() {
            cmd.env("ANTHROPIC_BASE_URL", &config.claude_code.base_url);
        }
        if !config.claude_code.model.is_empty() {
            cmd.env("ANTHROPIC_MODEL", &config.claude_code.model);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| AppError::Parse(e.to_string()))?;

        let session = ClaudeSession {
            id: session_id.clone(),
            master: pair.master,
            child,
            reader: Some(reader),
            reader_handle: None,
        };

        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id, session);

        tracing::info!("Claude PTY session created: {}", session_id);
        Ok(())
    }

    pub fn start_output_loop(
        &self,
        app_handle: tauri::AppHandle,
        session_id: &str,
    ) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            let reader = match session.reader.take() {
                Some(r) => r,
                None => return Ok(()),
            };

            let sid = session_id.to_string();
            let mut child_killer = session.child.clone_killer();
            let handle = std::thread::spawn(move || {
                let mut reader = reader;
                let mut buf = [0u8; 4096];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => {
                            let _ = app_handle.emit(
                                "claude-session-status",
                                serde_json::json!({"sessionId": sid, "status": "exited"}),
                            );
                            break;
                        }
                        Ok(n) => {
                            let output = String::from_utf8_lossy(&buf[..n]).to_string();
                            let _ = app_handle.emit("claude-pty-output", output);
                        }
                        Err(_) => break,
                    }
                }
                let _ = child_killer.kill();
            });
            session.reader_handle = Some(handle);
        }
        Ok(())
    }

    pub fn write_input(&self, session_id: &str, data: &str) -> AppResult<()> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            let mut writer = session
                .master
                .take_writer()
                .map_err(|e| AppError::Parse(e.to_string()))?;
            write!(writer, "{}", data)?;
            writer.flush()?;
        }
        Ok(())
    }

    pub fn resize_session(&self, session_id: &str, rows: u16, cols: u16) -> AppResult<()> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            session.master
                .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
                .map_err(|e| AppError::Parse(e.to_string()))?;
        }
        Ok(())
    }

    pub fn close_session(&self, session_id: &str) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(mut session) = sessions.remove(session_id) {
            let _ = session.child.kill();
            if let Some(handle) = session.reader_handle.take() {
                let _ = handle.join();
            }
        }
        Ok(())
    }

    pub fn close_all(&self) {
        let mut sessions = self.sessions.lock().unwrap();
        for (_, mut session) in sessions.drain() {
            let _ = session.child.kill();
            if let Some(handle) = session.reader_handle.take() {
                let _ = handle.join();
            }
        }
    }
}
```

- [ ] **Step 2: 编译验证**

Run: `cd src-tauri && cargo check 2>&1 | tail -10`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/claude/manager.rs
git commit -m "feat(claude): 实现 ClaudeManager（交互式 PTY 会话管理）"
```

---

### Task 5: 注册 Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/claude_commands.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 claude_commands.rs**

```rust
use std::sync::Mutex;
use tauri::{AppHandle, State};
use crate::claude::cli_downloader::{self, CliDownloader};
use crate::claude::manager::ClaudeManager;
use crate::config::{self, ClaudeConfig, ConfigState};
use crate::error::AppResult;

pub struct ClaudeManagerState(pub Mutex<ClaudeManager>);
pub struct CliDownloaderState(pub Mutex<CliDownloader>);

#[tauri::command]
pub async fn check_claude_installed() -> AppResult<serde_json::Value> {
    let installed = cli_downloader::is_installed();
    let version = cli_downloader::get_installed_version();
    Ok(serde_json::json!({
        "installed": installed,
        "version": version,
    }))
}

#[tauri::command]
pub async fn install_claude_cli(
    app_handle: AppHandle,
    downloader: State<'_, CliDownloaderState>,
) -> AppResult<serde_json::Value> {
    let downloader = downloader.0.lock().unwrap();
    let path = cli_downloader::download_cli(&downloader, app_handle).await?;
    Ok(serde_json::json!({
        "path": path.to_string_lossy(),
    }))
}

#[tauri::command]
pub async fn get_claude_cli_path() -> AppResult<serde_json::Value> {
    let path = cli_downloader::current_cli_path();
    Ok(serde_json::json!({
        "path": path.to_string_lossy().to_string(),
        "exists": path.exists(),
    }))
}

#[tauri::command]
pub async fn start_claude_session(
    app_handle: AppHandle,
    manager: State<'_, ClaudeManagerState>,
) -> AppResult<String> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let mgr = manager.0.lock().unwrap();
    mgr.create_session(app_handle, session_id.clone())?;
    Ok(session_id)
}

#[tauri::command]
pub async fn start_claude_output(
    session_id: String,
    app_handle: AppHandle,
    manager: State<'_, ClaudeManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.start_output_loop(app_handle, &session_id)
}

#[tauri::command]
pub async fn write_claude_input(
    session_id: String,
    data: String,
    manager: State<'_, ClaudeManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.write_input(&session_id, &data)
}

#[tauri::command]
pub async fn resize_claude_terminal(
    session_id: String,
    rows: u16,
    cols: u16,
    manager: State<'_, ClaudeManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.resize_session(&session_id, rows, cols)
}

#[tauri::command]
pub async fn close_claude_session(
    session_id: String,
    manager: State<'_, ClaudeManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.close_session(&session_id)
}

#[tauri::command]
pub async fn get_claude_config(
    config: State<'_, ConfigState>,
) -> AppResult<ClaudeConfig> {
    let cfg = config.0.lock().unwrap();
    Ok(cfg.claude_code.clone())
}

#[tauri::command]
pub async fn update_claude_config(
    provider: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
    config: State<'_, ConfigState>,
) -> AppResult<()> {
    let mut cfg = config.0.lock().unwrap();
    if let Some(v) = provider { cfg.claude_code.provider = v; }
    if let Some(v) = api_key { cfg.claude_code.api_key = v; }
    if let Some(v) = base_url { cfg.claude_code.base_url = v; }
    if let Some(v) = model { cfg.claude_code.model = v; }
    config::save_config(&cfg)?;
    Ok(())
}
```

- [ ] **Step 2: 修改 commands/mod.rs**

添加一行：`pub mod claude_commands;`

- [ ] **Step 3: 修改 lib.rs 注册新 State 和 Commands**

在 `mod` 声明区添加 `mod claude;`（如 Task 2 已做则跳过）。

在 `use` 区添加：
```rust
use claude::manager::ClaudeManager;
use commands::claude_commands::{ClaudeManagerState, CliDownloaderState};
use claude::CliDownloader;
```

在 `setup` 中添加 State 注册（`app.manage(...)` 区域）：
```rust
app.manage(ClaudeManagerState(std::sync::Mutex::new(ClaudeManager::new())));
app.manage(CliDownloaderState(std::sync::Mutex::new(CliDownloader::new())));
```

在 `invoke_handler` 中添加所有新命令：
```rust
commands::claude_commands::check_claude_installed,
commands::claude_commands::install_claude_cli,
commands::claude_commands::get_claude_cli_path,
commands::claude_commands::start_claude_session,
commands::claude_commands::start_claude_output,
commands::claude_commands::write_claude_input,
commands::claude_commands::resize_claude_terminal,
commands::claude_commands::close_claude_session,
commands::claude_commands::get_claude_config,
commands::claude_commands::update_claude_config,
```

- [ ] **Step 4: 检查 uuid 依赖**

Run: `grep 'uuid' src-tauri/Cargo.toml`
若无，添加 `uuid = { version = "1", features = ["v4"] }` 到 `Cargo.toml`。

- [ ] **Step 5: 编译验证**

Run: `cd src-tauri && cargo check 2>&1 | tail -10`
Expected: 编译成功

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/ src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat(claude): 注册 Claude Tauri Commands 和 State"
```

---

### Task 6: 前端类型 + IPC 封装

**Files:**
- Modify: `src/types/persona.ts`
- Create: `src/composables/useClaude.ts`

- [ ] **Step 1: 在 persona.ts 底部添加 Claude 相关类型**

```typescript
export interface ClaudeConfig {
  provider: string
  api_key: string
  base_url: string
  model: string
}

export interface ClaudeCliStatus {
  installed: boolean
  version: string | null
}
```

- [ ] **Step 2: 创建 useClaude.ts**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { ClaudeConfig, ClaudeCliStatus } from '../types/persona'

export function useClaude() {
  const checkInstalled = async (): Promise<ClaudeCliStatus> => {
    return invoke<ClaudeCliStatus>('check_claude_installed')
  }

  const installCli = async (): Promise<{ path: string }> => {
    return invoke<{ path: string }>('install_claude_cli')
  }

  const getCliPath = async (): Promise<{ path: string; exists: boolean }> => {
    return invoke<{ path: string; exists: boolean }>('get_claude_cli_path')
  }

  const startSession = async (): Promise<string> => {
    return invoke<string>('start_claude_session')
  }

  const startOutput = async (sessionId: string): Promise<void> => {
    return invoke('start_claude_output', { sessionId })
  }

  const writeInput = async (sessionId: string, data: string): Promise<void> => {
    return invoke('write_claude_input', { sessionId, data })
  }

  const resizeTerminal = async (sessionId: string, rows: number, cols: number): Promise<void> => {
    return invoke('resize_claude_terminal', { sessionId, rows, cols })
  }

  const closeSession = async (sessionId: string): Promise<void> => {
    return invoke('close_claude_session', { sessionId })
  }

  const getConfig = async (): Promise<ClaudeConfig> => {
    return invoke<ClaudeConfig>('get_claude_config')
  }

  const updateConfig = async (params: {
    provider?: string
    apiKey?: string
    baseUrl?: string
    model?: string
  }): Promise<void> => {
    return invoke('update_claude_config', {
      provider: params.provider ?? null,
      apiKey: params.apiKey ?? null,
      baseUrl: params.baseUrl ?? null,
      model: params.model ?? null,
    })
  }

  return {
    checkInstalled,
    installCli,
    getCliPath,
    startSession,
    startOutput,
    writeInput,
    resizeTerminal,
    closeSession,
    getConfig,
    updateConfig,
  }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/types/persona.ts src/composables/useClaude.ts
git commit -m "feat(frontend): 添加 Claude TypeScript 类型 + IPC 封装"
```

---

### Task 7: BottomPanel 底部面板组件

**Files:**
- Create: `src/components/layout/BottomPanel.vue`

- [ ] **Step 1: 创建 BottomPanel.vue**

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
import InteractiveTerminal from '../claude/InteractiveTerminal.vue'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const isOpen = computed({
  get: () => props.modelValue,
  set: (v) => emit('update:modelValue', v),
})

const panelHeight = ref(280)
const MIN_HEIGHT = 120
const MAX_HEIGHT = 600
const DEFAULT_HEIGHT = 280

const isDragging = ref(false)
let startY = 0
let startHeight = 0

function onDragStart(e: MouseEvent) {
  isDragging.value = true
  startY = e.clientY
  startHeight = panelHeight.value
  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragEnd)
  e.preventDefault()
}

function onDragMove(e: MouseEvent) {
  const delta = startY - e.clientY
  panelHeight.value = Math.max(MIN_HEIGHT, Math.min(MAX_HEIGHT, startHeight + delta))
}

function onDragEnd() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDragMove)
  document.removeEventListener('mouseup', onDragEnd)
}

function onDoubleClick() {
  if (isOpen.value && panelHeight.value === DEFAULT_HEIGHT) {
    isOpen.value = false
  } else {
    isOpen.value = true
    panelHeight.value = DEFAULT_HEIGHT
  }
}
</script>

<template>
  <div class="flex flex-col" :class="{ 'select-none': isDragging }">
    <!-- Drag handle / divider -->
    <div
      class="group flex cursor-row-resize items-center justify-center border-t border-border hover:bg-accent/50 transition-colors"
      style="height: 6px;"
      @mousedown="onDragStart"
      @dblclick="onDoubleClick"
    >
      <div class="h-[2px] w-8 rounded-full bg-border group-hover:bg-foreground/30 transition-colors" />
    </div>

    <!-- Status bar (collapsed) -->
    <div
      v-if="!isOpen"
      class="flex items-center justify-between border-t border-border px-4 py-1.5 text-xs text-muted-foreground bg-background cursor-pointer hover:bg-accent/30"
      @click="isOpen = true"
    >
      <span>● Claude Code 就绪</span>
      <span>点击展开</span>
    </div>

    <!-- Panel (expanded) -->
    <div
      v-if="isOpen"
      :style="{ height: `${panelHeight}px` }"
      class="flex flex-col overflow-hidden bg-background"
    >
      <!-- Panel header -->
      <div class="flex items-center justify-between border-b border-border px-3 py-1">
        <span class="text-xs font-medium">Claude Code</span>
        <button
          class="text-xs text-muted-foreground hover:text-foreground px-1"
          @click="isOpen = false"
        >
          ✕
        </button>
      </div>

      <!-- Terminal content -->
      <div class="flex-1 overflow-hidden">
        <InteractiveTerminal />
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/layout/BottomPanel.vue
git commit -m "feat(layout): 添加 BottomPanel 底部面板（拖拽分割条）"
```

---

### Task 8: InteractiveTerminal 交互式终端组件

**Files:**
- Create: `src/components/claude/InteractiveTerminal.vue`

- [ ] **Step 1: 创建 InteractiveTerminal.vue**

```vue
<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { Terminal } from 'xterm'
import { FitAddon } from 'xterm-addon-fit'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useClaude } from '@/composables/useClaude'
import 'xterm/css/xterm.css'

const {
  checkInstalled,
  installCli,
  startSession,
  startOutput,
  writeInput,
  resizeTerminal,
  closeSession,
} = useClaude()

const terminalRef = ref<HTMLElement | null>(null)
const status = ref<'idle' | 'installing' | 'running' | 'exited'>('idle')
const sessionId = ref<string | null>(null)
const error = ref<string | null>(null)

let terminal: Terminal | null = null
let fitAddon: FitAddon | null = null
let unlistenOutput: UnlistenFn | null = null
let unlistenStatus: UnlistenFn | null = null

function initTerminal() {
  if (!terminalRef.value) return

  terminal = new Terminal({
    cursorBlink: true,
    fontSize: 13,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    theme: {
      background: '#1a1a1a',
      foreground: '#d4d4d4',
      cursor: '#d4d4d4',
      selectionBackground: '#264f78',
    },
  })

  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  terminal.open(terminalRef.value)
  nextTick(() => fitAddon?.fit())

  terminal.onData((data) => {
    if (sessionId.value) {
      writeInput(sessionId.value, data).catch(console.error)
    }
  })
}

function handleResize() {
  if (fitAddon && terminal) {
    fitAddon.fit()
    if (sessionId.value) {
      const { rows, cols } = terminal
      resizeTerminal(sessionId.value, rows, cols).catch(console.error)
    }
  }
}

async function setupEventListeners() {
  unlistenOutput = await listen<string>('claude-pty-output', (event) => {
    if (terminal && event.payload) {
      terminal.write(event.payload)
    }
  })

  unlistenStatus = await listen<{ sessionId: string; status: string }>(
    'claude-session-status',
    (event) => {
      if (event.payload.status === 'exited') {
        status.value = 'exited'
        terminal?.write('\r\n\x1b[33m--- 会话已结束 ---\x1b[0m\r\n')
      }
    },
  )
}

async function startClaude() {
  error.value = null
  const cliStatus = await checkInstalled()

  if (!cliStatus.installed) {
    status.value = 'installing'
    terminal?.write('\x1b[36m正在下载 Claude CLI...\x1b[0m\r\n')
    try {
      await installCli()
      terminal?.write('\x1b[32m下载完成！\x1b[0m\r\n')
    } catch (e: any) {
      error.value = e.toString()
      terminal?.write(`\x1b[31m下载失败: ${e}\x1b[0m\r\n`)
      status.value = 'idle'
      return
    }
  }

  status.value = 'running'
  try {
    const sid = await startSession()
    sessionId.value = sid
    await startOutput(sid)
    terminal?.write('\x1b[32mClaude Code 已启动\x1b[0m\r\n')
  } catch (e: any) {
    error.value = e.toString()
    terminal?.write(`\x1b[31m启动失败: ${e}\x1b[0m\r\n`)
    status.value = 'idle'
  }
}

async function restart() {
  if (sessionId.value) {
    await closeSession(sessionId.value).catch(console.error)
    sessionId.value = null
  }
  terminal?.clear()
  await startClaude()
}

function cleanup() {
  unlistenOutput?.()
  unlistenStatus?.()
  unlistenOutput = null
  unlistenStatus = null
  if (sessionId.value) {
    closeSession(sessionId.value).catch(console.error)
    sessionId.value = null
  }
  if (terminal) {
    terminal.dispose()
    terminal = null
  }
  window.removeEventListener('resize', handleResize)
}

onMounted(() => {
  initTerminal()
  setupEventListeners()
  window.addEventListener('resize', handleResize)
  startClaude()
})

onUnmounted(() => {
  cleanup()
})
</script>

<template>
  <div class="flex h-full flex-col bg-[#1a1a1a]">
    <!-- Terminal body -->
    <div ref="terminalRef" class="flex-1 overflow-hidden p-2" />

    <!-- Restart button (shown when exited) -->
    <div
      v-if="status === 'exited'"
      class="flex items-center justify-center gap-3 border-t border-[#3e3e3e] px-3 py-2"
    >
      <span class="text-xs text-[#888]">会话已结束</span>
      <button
        class="rounded bg-blue-600 px-3 py-1 text-xs text-white hover:bg-blue-500 transition-colors"
        @click="restart"
      >
        重新启动
      </button>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/claude/InteractiveTerminal.vue
git commit -m "feat(claude): 添加 InteractiveTerminal 交互式终端组件"
```

---

### Task 9: ClaudeSettings 设置组件

**Files:**
- Create: `src/components/settings/ClaudeSettings.vue`
- Modify: `src/views/SettingsView.vue`

- [ ] **Step 1: 创建 ClaudeSettings.vue**

```vue
<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useClaude } from '@/composables/useClaude'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

const claude = useClaude()
const saving = ref(false)
const saved = ref(false)
const cliStatus = ref<{ installed: boolean; version: string | null }>({ installed: false, version: null })
const installing = ref(false)
const activeMirror = ref<string | null>(null)

const config = reactive({
  provider: 'anthropic',
  apiKey: '',
  baseUrl: 'https://api.anthropic.com',
  model: 'claude-sonnet-4-20250514',
})

onMounted(async () => {
  try {
    const status = await claude.checkInstalled()
    cliStatus.value = status
  } catch (e) {
    console.error('Failed to check CLI status:', e)
  }

  try {
    const cfg = await claude.getConfig()
    config.provider = cfg.provider
    config.apiKey = cfg.api_key
    config.baseUrl = cfg.base_url
    config.model = cfg.model
  } catch (e) {
    console.error('Failed to load Claude config:', e)
  }
})

function onProviderChange() {
  if (config.provider === 'anthropic') {
    config.baseUrl = 'https://api.anthropic.com'
    config.model = 'claude-sonnet-4-20250514'
  }
}

async function installCli() {
  installing.value = true
  try {
    await claude.installCli()
    const status = await claude.checkInstalled()
    cliStatus.value = status
  } catch (e: any) {
    console.error('Install failed:', e)
  } finally {
    installing.value = false
  }
}

async function saveConfig() {
  saving.value = true
  saved.value = false
  try {
    await claude.updateConfig({
      provider: config.provider,
      apiKey: config.apiKey || undefined,
      baseUrl: config.baseUrl || undefined,
      model: config.model || undefined,
    })
    saved.value = true
    setTimeout(() => { saved.value = false }, 2000)
  } catch (e) {
    console.error('Failed to save Claude config:', e)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>Claude Code</CardTitle>
      <CardDescription>内置 Claude Code 终端配置</CardDescription>
    </CardHeader>
    <CardContent class="flex flex-col gap-4">
      <!-- CLI Status -->
      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground">CLI 状态</label>
        <div class="flex items-center gap-3">
          <span v-if="cliStatus.installed" class="text-sm text-emerald-500">
            已安装 {{ cliStatus.version }}
          </span>
          <span v-else class="text-sm text-muted-foreground">未安装</span>
          <Button
            v-if="!cliStatus.installed"
            size="sm"
            :disabled="installing"
            @click="installCli"
          >
            {{ installing ? '安装中...' : '安装 Claude CLI' }}
          </Button>
        </div>
      </div>

      <!-- Provider -->
      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground">API Provider</label>
        <select
          v-model="config.provider"
          @change="onProviderChange"
          class="flex h-8 w-full rounded-lg border border-input bg-background px-3 py-1 text-sm shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
        >
          <option value="anthropic">Anthropic</option>
          <option value="openai-compatible">OpenAI Compatible</option>
          <option value="custom">自定义</option>
        </select>
      </div>

      <!-- API Key -->
      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground">API Key</label>
        <Input v-model="config.apiKey" type="password" placeholder="sk-ant-..." />
      </div>

      <!-- Base URL -->
      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground">请求地址</label>
        <Input v-model="config.baseUrl" placeholder="https://api.anthropic.com" />
      </div>

      <!-- Model -->
      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground">模型</label>
        <Input v-model="config.model" placeholder="claude-sonnet-4-20250514" />
      </div>

      <!-- Save -->
      <div class="flex items-center gap-3 pt-2">
        <Button @click="saveConfig" :disabled="saving">
          {{ saving ? '保存中...' : '保存 Claude 配置' }}
        </Button>
        <span v-if="saved" class="text-sm text-emerald-500">已保存</span>
      </div>
    </CardContent>
  </Card>
</template>
```

- [ ] **Step 2: 在 SettingsView.vue 中集成 ClaudeSettings**

在 `SettingsView.vue` 的 `<script setup>` 中添加导入：
```typescript
import ClaudeSettings from '@/components/settings/ClaudeSettings.vue';
```

在模板中"数据信息"Card 之前添加：
```html
<!-- Claude Code 配置 -->
<ClaudeSettings />
```

- [ ] **Step 3: Commit**

```bash
git add src/components/settings/ClaudeSettings.vue src/views/SettingsView.vue
git commit -m "feat(settings): 添加 Claude Code 配置组件并集成到设置页"
```

---

### Task 10: 集成 BottomPanel 到 AppLayout

**Files:**
- Modify: `src/components/AppLayout.vue`

- [ ] **Step 1: 修改 AppLayout.vue 集成 BottomPanel**

在 `<script setup>` 中添加导入：
```typescript
import BottomPanel from './layout/BottomPanel.vue';
```

添加状态：
```typescript
const claudePanelOpen = ref(false);
```

在模板中，将 `<main>` 区域和 BottomPanel 一起包裹。修改模板结构：

将原来第 50-65 行的 `<main>` 区块和其后的 `</div>`（根元素闭合标签，原第 72 行）之间插入 BottomPanel。具体改动：

```html
    <!-- 原有 main 区块不变 -->
    <main class="flex flex-1 flex-col overflow-hidden">
      <!-- ... 原有内容不变 ... -->
    </main>

    <!-- BottomPanel 放在 main 之后、CreateNoteDialog 之前 -->
    <BottomPanel v-model="claudePanelOpen" />
```

同时在顶部标签栏（topbar）的按钮区域（第 55-59 行）添加 Claude Code 切换按钮：

```html
<Button variant="ghost" size="icon" @click="claudePanelOpen = !claudePanelOpen" title="Claude Code">
  ⌨
</Button>
```

放在主题切换按钮之前。

- [ ] **Step 2: 验证开发模式**

Run: `npm run dev:all`

验证：
1. 应用正常启动
2. 顶部标签栏出现 ⌨ 按钮
3. 点击 ⌨ 按钮展开底部面板
4. 底部面板显示 CLI 安装状态和终端

- [ ] **Step 3: Commit**

```bash
git add src/components/AppLayout.vue
git commit -m "feat(layout): 集成 BottomPanel 到 AppLayout，添加 Claude Code 切换按钮"
```

---

### Task 11: 端到端验证 + 收尾

**Files:**
- 无新增文件

- [ ] **Step 1: 完整流程验证**

1. 打开设置页 → Claude Code 区块可见
2. 填写 API Key → 保存 → 确认 config.json 已更新
3. 点击 ⌨ 按钮 → 底部面板展开
4. CLI 自动下载（检查进度推送）
5. 终端启动 → 可输入 → 可看到 Claude 输出
6. 拖拽分割条 → 面板高度变化
7. 双击分割条 → 面板收起
8. 关闭应用 → PTY 会话自动终止

- [ ] **Step 2: 修复发现的问题**

根据验证结果修复任何 bug，每个修复单独 commit。

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat: 内置 Claude Code 完成（CLI 下载 + 交互终端 + Skills 注入 + 设置）"
```
