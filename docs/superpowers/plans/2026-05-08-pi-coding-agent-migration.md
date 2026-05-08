# Pi Coding Agent Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Claude Code CLI integration with pi-coding-agent, keeping PTY + xterm.js architecture.

**Architecture:** Drop-in CLI replacement. PiManager replaces ClaudeManager, PiRuntime handles Node.js + pi CLI installation. Skills deploy to `~/.pi/agent/skills/` for pi auto-discovery. Persona rewrite uses `pi -p` instead of `claude --dangerously-skip-permissions`.

**Tech Stack:** Tauri v2, Rust, Vue 3, TypeScript, portable_pty, xterm.js, Node.js (runtime dependency)

---

## File Structure

### New files to create

| File | Responsibility |
|------|------|
| `src-tauri/src/pi/mod.rs` | Module entry, re-exports |
| `src-tauri/src/pi/runtime.rs` | Node.js detection/download, pi CLI installation |
| `src-tauri/src/pi/manager.rs` | PTY session management (interactive + persona rewrite) |
| `src-tauri/src/pi/skills.rs` | Deploy built-in skills to `~/.pi/agent/skills/` |
| `src-tauri/src/commands/pi_commands.rs` | 6 Tauri commands |
| `src/composables/usePi.ts` | Frontend IPC wrapper |

### Files to modify

| File | Change |
|------|------|
| `src-tauri/src/lib.rs` | Replace claude/terminal module init with pi module |
| `src-tauri/src/config.rs` | Rename `ClaudeConfig` → `AgentConfig`, field `claude_code` → `agent` |
| `src-tauri/src/persona/builtin.rs` | Remove old `~/.claude/plugins/cache/` deployment |
| `src/components/claude/InteractiveTerminal.vue` | Switch to usePi, rename events |
| `src/components/layout/BottomPanel.vue` | Rename labels |
| `src/components/settings/ClaudeSettings.vue` | Rename to AgentSettings, switch to usePi |
| `src/components/persona/TerminalPanel.vue` | Switch to usePi, rename events |
| `src/types/persona.ts` | Rename Claude types to Agent types |
| `src/views/SettingsView.vue` | Import rename |
| `src/views/NotesView.vue` | Inject key rename |
| `src/components/AppLayout.vue` | Provide key rename, label updates |

### Files to delete

| File | Reason |
|------|------|
| `src-tauri/src/claude/mod.rs` | Replaced by `pi/mod.rs` |
| `src-tauri/src/claude/cli_downloader.rs` | Replaced by `pi/runtime.rs` |
| `src-tauri/src/claude/manager.rs` | Replaced by `pi/manager.rs` |
| `src-tauri/src/claude/skills.rs` | Replaced by `pi/skills.rs` |
| `src-tauri/src/terminal/mod.rs` | Merged into `pi/manager.rs` |
| `src-tauri/src/terminal/pty_manager.rs` | Merged into `pi/manager.rs` |
| `src-tauri/src/terminal/session.rs` | Merged into `pi/manager.rs` |
| `src-tauri/src/commands/claude_commands.rs` | Replaced by `pi_commands.rs` |
| `src-tauri/src/commands/terminal_commands.rs` | Merged into `pi_commands.rs` |
| `src/composables/useClaude.ts` | Replaced by `usePi.ts` |

---

## Task 1: Create `pi/mod.rs` — Module Entry

**Files:**
- Create: `src-tauri/src/pi/mod.rs`

- [ ] **Step 1: Create pi module directory and entry file**

```rust
// src-tauri/src/pi/mod.rs
pub mod manager;
pub mod runtime;
pub mod skills;

pub use manager::PiManager;
pub use runtime::PiRuntime;
pub use skills::PiSkillsManager;
```

- [ ] **Step 2: Verify it compiles** (will fail until runtime.rs, manager.rs, skills.rs exist — that's expected, just ensure mod syntax is correct)

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/pi/mod.rs
git commit -m "feat(pi): add pi module entry point"
```

---

## Task 2: Create `pi/runtime.rs` — Node.js + Pi CLI Runtime

**Files:**
- Create: `src-tauri/src/pi/runtime.rs`

This module detects/downloads Node.js and installs pi CLI via npm. Mirrors the role of `claude/cli_downloader.rs` but for Node.js + pi.

- [ ] **Step 1: Write `pi/runtime.rs`**

```rust
// src-tauri/src/pi/runtime.rs
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiStatus {
    pub node_installed: bool,
    pub pi_installed: bool,
    pub node_version: Option<String>,
    pub pi_version: Option<String>,
}

/// Pi runtime: manages Node.js and pi CLI installation
pub struct PiRuntime {
    /// Path to local Node.js directory (~/.omnilink/node/)
    node_dir: PathBuf,
    /// Path to local pi CLI directory (~/.omnilink/pi-cli/)
    pi_dir: PathBuf,
}

impl PiRuntime {
    pub fn new() -> Self {
        let base = dirs::home_dir()
            .expect("Cannot determine home directory")
            .join(".omnilink");
        Self {
            node_dir: base.join("node"),
            pi_dir: base.join("pi-cli"),
        }
    }

    /// Get the Node.js binary path (local install or system)
    fn node_binary(&self) -> Option<PathBuf> {
        // 1. Check local install
        let local = self.node_dir.join("bin").join("node");
        if local.exists() {
            return Some(local);
        }
        // 2. Check system PATH
        if let Ok(output) = Command::new("which").arg("node").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
        }
        None
    }

    /// Get the npm binary path
    fn npm_binary(&self) -> Option<PathBuf> {
        let node = self.node_binary()?;
        let bin_dir = node.parent()?;
        Some(bin_dir.join("npm"))
    }

    /// Get the pi CLI binary path
    pub fn pi_binary(&self) -> PathBuf {
        self.pi_dir.join("bin").join("pi")
    }

    /// Check if everything is installed and ready
    pub fn check_installed(&self) -> PiStatus {
        let node = self.node_binary();
        let node_installed = node.is_some();
        let node_version = node.as_ref().and_then(|n| {
            Command::new(n).arg("--version").output().ok().and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
        });

        let pi_path = self.pi_binary();
        let pi_installed = pi_path.exists();
        let pi_version = if pi_installed {
            Command::new(&pi_path).arg("--version").output().ok().and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
        } else {
            None
        };

        PiStatus {
            node_installed,
            pi_installed,
            node_version,
            pi_version,
        }
    }

    /// Download and install Node.js LTS to ~/.omnilink/node/
    pub fn install_node(&self) -> AppResult<()> {
        if self.node_binary().is_some() {
            return Ok(());
        }

        let (platform, arch) = Self::detect_platform()?;
        let ext = if cfg!(target_os = "windows") { "zip" } else { "tar.gz" };
        let version = "v22.14.0"; // Node.js 22 LTS
        let filename = format!("node-{}-{}-{}.{}", version, platform, arch, ext);
        let url = format!("https://nodejs.org/dist/{}/{}", version, filename);

        let temp_dir = std::env::temp_dir().join("omnilink-node-install");
        std::fs::create_dir_all(&temp_dir)?;
        let archive_path = temp_dir.join(&filename);

        tracing::info!("Downloading Node.js from: {}", url);
        let response = reqwest::blocking::Client::new()
            .get(&url)
            .send()
            .map_err(|e| AppError::Internal(format!("Node.js 下载失败: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Internal(format!(
                "Node.js 下载失败: HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .map_err(|e| AppError::Internal(format!("读取响应失败: {}", e)))?;
        std::fs::write(&archive_path, &bytes)?;

        // Clean target dir
        if self.node_dir.exists() {
            std::fs::remove_dir_all(&self.node_dir)?;
        }

        // Extract
        if ext == "tar.gz" {
            Self::extract_tar_gz(&archive_path, &self.node_dir)?;
        } else {
            Self::extract_zip(&archive_path, &self.node_dir)?;
        }

        // Archives contain a single top-level dir like node-v22.14.0-darwin-arm64/
        // Move its contents up
        Self::flatten_single_child(&self.node_dir)?;

        tracing::info!("Node.js installed to: {}", self.node_dir.display());
        Ok(())
    }

    /// Install pi CLI via npm
    pub fn install_pi(&self) -> AppResult<()> {
        let npm = self.npm_binary().ok_or_else(|| {
            AppError::Internal("Node.js 未安装，无法安装 pi CLI".to_string())
        })?;

        // Ensure target dir exists
        std::fs::create_dir_all(&self.pi_dir)?;

        let output = Command::new(npm)
            .args([
                "install",
                "-g",
                "--prefix",
                self.pi_dir.to_str().unwrap(),
                "@mariozechner/pi-coding-agent",
            ])
            .output()
            .map_err(|e| AppError::Internal(format!("npm install 失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Internal(format!(
                "pi CLI 安装失败: {}",
                stderr
            )));
        }

        tracing::info!("pi CLI installed to: {}", self.pi_dir.display());
        Ok(())
    }

    /// Ensure both Node.js and pi CLI are installed
    pub fn ensure_installed(&self) -> AppResult<()> {
        if self.node_binary().is_none() {
            self.install_node()?;
        }
        if !self.pi_binary().exists() {
            self.install_pi()?;
        }
        Ok(())
    }

    /// Build a Command pre-configured to run pi
    pub fn build_pi_command(&self) -> AppResult<std::process::Command> {
        self.ensure_installed()?;
        let pi = self.pi_binary();
        let mut cmd = std::process::Command::new(&pi);
        // Inherit PATH so pi can find node if needed
        if let Some(node_bin) = self.node_binary().and_then(|n| n.parent().map(|p| p.to_path_buf())) {
            let path = std::env::var_os("PATH").unwrap_or_default();
            let mut new_path = node_bin.into_os_string();
            new_path.push(":");
            new_path.push(path);
            cmd.env("PATH", new_path);
        }
        Ok(cmd)
    }

    fn detect_platform() -> AppResult<(&'static str, &'static str)> {
        let platform = if cfg!(target_os = "macos") {
            "darwin"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "windows") {
            "win"
        } else {
            return Err(AppError::Internal("不支持的平台".to_string()));
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            return Err(AppError::Internal("不支持的架构".to_string()));
        };

        Ok((platform, arch))
    }

    fn extract_tar_gz(archive: &PathBuf, target: &PathBuf) -> AppResult<()> {
        let file = std::fs::File::open(archive)?;
        let gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);
        archive.unpack(target)?;
        Ok(())
    }

    fn extract_zip(archive: &PathBuf, target: &PathBuf) -> AppResult<()> {
        let file = std::fs::File::open(archive)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| AppError::Internal(format!("zip 解压失败: {}", e)))?;
        archive.extract(target)
            .map_err(|e| AppError::Internal(format!("zip 解压失败: {}", e)))?;
        Ok(())
    }

    /// Move contents of single child directory up one level
    fn flatten_single_child(dir: &PathBuf) -> AppResult<()> {
        let entries: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .collect();
        if entries.len() == 1 && entries[0].path().is_dir() {
            let child = entries[0].path();
            for entry in std::fs::read_dir(&child)?.filter_map(|e| e.ok()) {
                let dest = dir.join(entry.file_name());
                std::fs::rename(entry.path(), dest)?;
            }
            std::fs::remove_dir(&child)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pi_runtime_creation() {
        let runtime = PiRuntime::new();
        assert!(runtime.node_dir.to_string_lossy().contains(".omnilink/node"));
        assert!(runtime.pi_dir.to_string_lossy().contains(".omnilink/pi-cli"));
    }

    #[test]
    fn test_pi_binary_path() {
        let runtime = PiRuntime::new();
        let bin = runtime.pi_binary();
        assert!(bin.to_string_lossy().contains("pi-cli/bin/pi"));
    }

    #[test]
    fn test_detect_platform() {
        let (platform, arch) = PiRuntime::detect_platform().unwrap();
        assert!(!platform.is_empty());
        assert!(!arch.is_empty());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd src-tauri && cargo test pi::runtime --lib -- --nocapture`
Expected: 3 tests PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/pi/runtime.rs
git commit -m "feat(pi): add PiRuntime for Node.js + pi CLI management"
```

---

## Task 3: Create `pi/skills.rs` — Skills Deployment

**Files:**
- Create: `src-tauri/src/pi/skills.rs`

Deploy built-in skills to `~/.pi/agent/skills/` so pi auto-discovers them. Same `include_str!` pattern as the old `claude/skills.rs` but targeting pi's standard path.

- [ ] **Step 1: Write `pi/skills.rs`**

```rust
// src-tauri/src/pi/skills.rs
use std::path::PathBuf;

use crate::error::AppResult;

/// Built-in skill entry: compiled into binary
struct BuiltinSkill {
    name: &'static str,
    content: &'static str,
}

/// Compiled-in built-in skills (SKILL.md files)
const BUILTIN_SKILLS: &[BuiltinSkill] = &[
    BuiltinSkill {
        name: "fangqikiki-perspective",
        content: include_str!("../../skills/fangqikiki-perspective/SKILL.md"),
    },
    BuiltinSkill {
        name: "leitanzhang-perspective",
        content: include_str!("../../skills/leitanzhang-perspective/SKILL.md"),
    },
    BuiltinSkill {
        name: "huashu-perspective",
        content: include_str!("../../skills/huashu-perspective/SKILL.md"),
    },
    BuiltinSkill {
        name: "nuwa-skill",
        content: include_str!("../../skills/nuwa-skill/SKILL.md"),
    },
    BuiltinSkill {
        name: "luo-yonghao-perspective",
        content: include_str!("../../skills/luo-yonghao-perspective/SKILL.md"),
    },
    BuiltinSkill {
        name: "zhangxiaolong-perspective",
        content: include_str!("../../skills/zhangxiaolong-perspective/SKILL.md"),
    },
    BuiltinSkill {
        name: "unified-search",
        content: include_str!("../../skills/unified-search/SKILL.md"),
    },
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub builtin: bool,
}

/// Manages deployment of built-in skills to pi's auto-discovery path
pub struct PiSkillsManager {
    /// Target: ~/.pi/agent/skills/
    deploy_dir: PathBuf,
}

impl PiSkillsManager {
    pub fn new() -> Self {
        let deploy_dir = dirs::home_dir()
            .expect("Cannot determine home directory")
            .join(".pi/agent/skills");
        Self { deploy_dir }
    }

    /// Deploy built-in skills to ~/.pi/agent/skills/ (skip if already exists)
    pub fn deploy_builtin_skills(&self) -> AppResult<()> {
        std::fs::create_dir_all(&self.deploy_dir)?;

        for builtin in BUILTIN_SKILLS {
            let skill_dir = self.deploy_dir.join(builtin.name);
            std::fs::create_dir_all(&skill_dir)?;
            let skill_md = skill_dir.join("SKILL.md");
            if !skill_md.exists() {
                std::fs::write(&skill_md, builtin.content)?;
                tracing::info!("Deployed builtin skill: {}", builtin.name);
            }
        }

        tracing::info!("Skills deployed to {}", self.deploy_dir.display());
        Ok(())
    }

    /// List all deployed skills (built-in + user-added)
    pub fn list_skills(&self) -> Vec<SkillInfo> {
        let mut skills: Vec<SkillInfo> = BUILTIN_SKILLS
            .iter()
            .map(|s| SkillInfo {
                name: s.name.to_string(),
                builtin: true,
            })
            .collect();

        if let Ok(entries) = std::fs::read_dir(&self.deploy_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("SKILL.md").exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !skills.iter().any(|s| s.name == name) {
                            skills.push(SkillInfo {
                                name: name.to_string(),
                                builtin: false,
                            });
                        }
                    }
                }
            }
        }

        skills
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_builtin_skills() {
        let manager = PiSkillsManager::new();
        let skills = manager.list_skills();
        let builtin: Vec<_> = skills.iter().filter(|s| s.builtin).collect();
        assert!(builtin.len() >= 4);
        assert!(builtin.iter().any(|s| s.name == "fangqikiki-perspective"));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd src-tauri && cargo test pi::skills --lib -- --nocapture`
Expected: 1 test PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/pi/skills.rs
git commit -m "feat(pi): add PiSkillsManager for skill deployment to ~/.pi/agent/skills/"
```

---

## Task 4: Create `pi/manager.rs` — PTY Session Manager

**Files:**
- Create: `src-tauri/src/pi/manager.rs`

Unified PTY manager handling both interactive sessions and persona rewrite. Replaces both `claude/manager.rs` and `terminal/pty_manager.rs` + `terminal/session.rs`.

- [ ] **Step 1: Write `pi/manager.rs`**

```rust
// src-tauri/src/pi/manager.rs
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

use super::runtime::PiRuntime;
use crate::config::{load_config, ClaudeConfig};
use crate::error::{AppError, AppResult};

/// A single PTY session
pub struct PiSession {
    pub id: String,
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    pub child: Box<dyn portable_pty::Child + Send + Sync>,
    pub reader: Option<Box<dyn std::io::Read + Send>>,
    pub reader_handle: Option<std::thread::JoinHandle<()>>,
    pub writer: Option<Box<dyn std::io::Write + Send>>,
}

/// Unified PTY session manager for pi CLI
pub struct PiManager {
    sessions: Arc<Mutex<HashMap<String, PiSession>>>,
    runtime: Arc<PiRuntime>,
}

impl PiManager {
    pub fn new(runtime: PiRuntime) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            runtime: Arc::new(runtime),
        }
    }

    /// Get a reference to the runtime for status checks
    pub fn runtime(&self) -> &PiRuntime {
        &self.runtime
    }

    /// Create an interactive pi PTY session
    pub fn create_session(
        &self,
        app_handle: tauri::AppHandle,
        session_id: String,
    ) -> AppResult<()> {
        let mut process_cmd = self.runtime.build_pi_command()
            .map_err(|e| AppError::Internal(format!("pi CLI 不可用: {}", e)))?;

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::Internal(format!("PTY 创建失败: {}", e)))?;

        let reader = pair.master.try_clone_reader()
            .map_err(|e| AppError::Internal(format!("PTY reader 克隆失败: {}", e)))?;
        let writer = pair.master.take_writer()
            .map_err(|e| AppError::Internal(format!("PTY writer 获取失败: {}", e)))?;

        let config = load_config()?;
        // Build portable_pty CommandBuilder from the process::Command
        let mut cmd = CommandBuilder::new(process_cmd.get_program());
        if let Some(dir) = process_cmd.get_current_dir() {
            cmd.cwd(dir);
        }
        // Copy environment variables
        for (k, v) in process_cmd.get_envs() {
            if let Some(val) = v {
                cmd.env(k, val);
            }
        }
        // Copy args
        for arg in process_cmd.get_args() {
            cmd.arg(arg);
        }
        // Inject API config
        Self::apply_config_env(&mut cmd, &config.claude_code);

        let child = pair.slave.spawn_command(cmd)
            .map_err(|e| AppError::Internal(format!("pi CLI 启动失败: {}", e)))?;

        let session = PiSession {
            id: session_id.clone(),
            master: pair.master,
            child,
            reader: Some(reader),
            reader_handle: None,
            writer: Some(writer),
        };

        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id.clone(), session);
        tracing::info!("pi PTY session created: {}", session_id);
        Ok(())
    }

    /// Create a persona rewrite session (pi print mode, non-interactive)
    pub fn create_persona_session(
        &self,
        app_handle: tauri::AppHandle,
        session_id: String,
        prompt: &str,
    ) -> AppResult<()> {
        let mut process_cmd = self.runtime.build_pi_command()
            .map_err(|e| AppError::Internal(format!("pi CLI 不可用: {}", e)))?;

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::Internal(format!("PTY 创建失败: {}", e)))?;

        let reader = pair.master.try_clone_reader()
            .map_err(|e| AppError::Internal(format!("PTY reader 克隆失败: {}", e)))?;
        // Take writer but no auto-accept needed for print mode
        let _writer = pair.master.take_writer()
            .map_err(|e| AppError::Internal(format!("PTY writer 获取失败: {}", e)))?;

        let config = load_config()?;
        let mut cmd = CommandBuilder::new(process_cmd.get_program());
        if let Some(dir) = process_cmd.get_current_dir() {
            cmd.cwd(dir);
        }
        for (k, v) in process_cmd.get_envs() {
            if let Some(val) = v {
                cmd.env(k, val);
            }
        }
        for arg in process_cmd.get_args() {
            cmd.arg(arg);
        }
        // pi print mode args
        cmd.arg("-p").arg(prompt);
        // Inject API config
        Self::apply_config_env(&mut cmd, &config.claude_code);

        let child = pair.slave.spawn_command(cmd)
            .map_err(|e| AppError::Internal(format!("pi print mode 启动失败: {}", e)))?;

        let session = PiSession {
            id: session_id.clone(),
            master: pair.master,
            child,
            reader: Some(reader),
            reader_handle: None,
            writer: None, // No writer needed for print mode
        };

        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id.clone(), session);
        tracing::info!("pi persona session created: {}", session_id);
        Ok(())
    }

    /// Start the PTY output loop, emitting events to frontend
    pub fn start_output_loop(
        &self,
        app_handle: tauri::AppHandle,
        session_id: &str,
    ) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| AppError::Internal(format!("会话 {} 不存在", session_id)))?;

        let reader = match session.reader.take() {
            Some(r) => r,
            None => return Ok(()),
        };

        let sid = session_id.to_string();
        let mut child_killer = session.child.clone_killer();

        let handle = std::thread::spawn(move || {
            tracing::info!("[pi-pty-reader] started for session {}", sid);
            let mut reader = reader;
            let mut buf = [0u8; 8192];
            let mut total_bytes: usize = 0;

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        tracing::info!("[pi-pty-reader] EOF for session {} ({} bytes)", sid, total_bytes);
                        let _ = app_handle.emit(
                            "pi-session-status",
                            serde_json::json!({"sessionId": sid, "status": "exited"}),
                        );
                        break;
                    }
                    Ok(n) => {
                        total_bytes += n;
                        let output = String::from_utf8_lossy(&buf[..n]).to_string();
                        let event_name = format!("pi-pty-output-{}", sid);
                        if let Err(e) = app_handle.emit(&event_name, &output) {
                            tracing::error!("[pi-pty-reader] emit failed for {}: {}", sid, e);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => {
                        tracing::error!("[pi-pty-reader] read error for {}: {}", sid, e);
                        break;
                    }
                }
            }

            let _ = child_killer.kill();
            tracing::info!("[pi-pty-reader] exiting for session {}", sid);
        });

        session.reader_handle = Some(handle);
        Ok(())
    }

    /// Write user input to PTY
    pub fn write_input(&self, session_id: &str, data: &str) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| AppError::Internal(format!("会话 {} 不存在", session_id)))?;

        let writer = session.writer.as_mut()
            .ok_or_else(|| AppError::Internal(format!("会话 {} 的 writer 不可用（可能是 print mode）", session_id)))?;
        write!(writer, "{}", data)?;
        writer.flush()?;
        Ok(())
    }

    /// Resize PTY
    pub fn resize_session(&self, session_id: &str, rows: u16, cols: u16) -> AppResult<()> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_id)
            .ok_or_else(|| AppError::Internal(format!("会话 {} 不存在", session_id)))?;

        session.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        }).map_err(|e| AppError::Internal(format!("PTY resize 失败: {}", e)))?;
        Ok(())
    }

    /// Close a session
    pub fn close_session(&self, session_id: &str) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(mut session) = sessions.remove(session_id) {
            let _ = session.child.kill();
            if let Some(handle) = session.reader_handle.take() {
                let _ = handle.join();
            }
            tracing::info!("pi PTY session closed: {}", session_id);
        }
        Ok(())
    }

    /// Close all sessions on shutdown
    pub fn close_all(&self) {
        let mut sessions = self.sessions.lock().unwrap();
        for (id, mut session) in sessions.drain() {
            let _ = session.child.kill();
            if let Some(handle) = session.reader_handle.take() {
                let _ = handle.join();
            }
            tracing::info!("pi PTY session closed (cleanup): {}", id);
        }
    }

    fn apply_config_env(cmd: &mut CommandBuilder, config: &ClaudeConfig) {
        if !config.api_key.is_empty() {
            cmd.env("ANTHROPIC_API_KEY", &config.api_key);
        }
        if config.provider == "openai-compatible" && !config.base_url.is_empty() {
            cmd.env("OPENAI_BASE_URL", &config.base_url);
            if !config.api_key.is_empty() {
                cmd.env("OPENAI_API_KEY", &config.api_key);
            }
        } else if !config.base_url.is_empty() {
            cmd.env("ANTHROPIC_BASE_URL", &config.base_url);
        }
        if !config.model.is_empty() {
            cmd.env("ANTHROPIC_MODEL", &config.model);
        }
    }
}

impl Drop for PiManager {
    fn drop(&mut self) {
        self.close_all();
    }
}
```

- [ ] **Step 2: Verify it compiles** (depends on Task 1-3 being complete)

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/pi/manager.rs
git commit -m "feat(pi): add PiManager for PTY session management"
```

---

## Task 5: Create `commands/pi_commands.rs` — Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/pi_commands.rs`

6 Tauri commands replacing the 13 claude_commands + 6 terminal_commands.

- [ ] **Step 1: Write `pi_commands.rs`**

```rust
// src-tauri/src/commands/pi_commands.rs
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, State};

use crate::config::{self, ClaudeConfig, ConfigState};
use crate::db::DbState;
use crate::error::AppResult;
use crate::models::terminal_session::CreateTerminalSession;
use crate::pi::manager::PiManager;
use crate::pi::runtime::PiRuntime;
use crate::pi::skills::PiSkillsManager;
use crate::repositories::terminal_session_repo;

pub struct PiManagerState(pub Mutex<PiManager>);
pub struct PiRuntimeState(pub Arc<PiRuntime>);
pub struct PiSkillsState(pub Arc<PiSkillsManager>);

#[tauri::command]
pub async fn check_pi_installed(
    runtime: State<'_, PiRuntimeState>,
) -> AppResult<serde_json::Value> {
    let status = runtime.0.check_installed();
    Ok(serde_json::json!({
        "nodeInstalled": status.node_installed,
        "piInstalled": status.pi_installed,
        "nodeVersion": status.node_version,
        "piVersion": status.pi_version,
    }))
}

#[tauri::command]
pub async fn install_pi(
    runtime: State<'_, PiRuntimeState>,
) -> AppResult<serde_json::Value> {
    runtime.0.install_node()?;
    runtime.0.install_pi()?;
    let status = runtime.0.check_installed();
    Ok(serde_json::json!({
        "nodeInstalled": status.node_installed,
        "piInstalled": status.pi_installed,
        "piVersion": status.pi_version,
    }))
}

#[tauri::command]
pub async fn start_pi_session(
    app_handle: AppHandle,
    manager: State<'_, PiManagerState>,
) -> AppResult<String> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let mgr = manager.0.lock().unwrap();
    mgr.create_session(app_handle, session_id.clone())?;
    Ok(session_id)
}

#[tauri::command]
pub async fn start_pi_output(
    session_id: String,
    app_handle: AppHandle,
    manager: State<'_, PiManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.start_output_loop(app_handle, &session_id)
}

#[tauri::command]
pub async fn write_pi_input(
    session_id: String,
    data: String,
    manager: State<'_, PiManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.write_input(&session_id, &data)
}

#[tauri::command]
pub async fn resize_pi_terminal(
    session_id: String,
    rows: u16,
    cols: u16,
    manager: State<'_, PiManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.resize_session(&session_id, rows, cols)
}

// === Persona Rewrite (replaces terminal_commands) ===

#[tauri::command]
pub async fn start_persona_rewrite(
    note_id: i64,
    note_path: String,
    persona_skill: String,
    mode: String,
    app_handle: AppHandle,
    db: State<'_, DbState>,
    manager: State<'_, PiManagerState>,
) -> AppResult<String> {
    tracing::info!("start_persona_rewrite: note_id={}, persona={}", note_id, persona_skill);

    let conn = db.0.lock().unwrap();
    let create_session = CreateTerminalSession {
        note_id,
        persona_skill: persona_skill.clone(),
        mode,
    };
    let session_id = terminal_session_repo::insert_terminal_session(&conn, &create_session)?;
    drop(conn);

    let prompt = format!("使用 {} 重构 {}，直接覆盖内容", persona_skill, note_path);
    let mgr = manager.0.lock().unwrap();
    mgr.create_persona_session(app_handle, session_id.clone(), &prompt)?;
    Ok(session_id)
}

#[tauri::command]
pub async fn update_session_status(
    session_id: String,
    status: String,
    db: State<'_, DbState>,
) -> AppResult<()> {
    let conn = db.0.lock().unwrap();
    terminal_session_repo::update_session_status(&conn, &session_id, &status)
}

#[tauri::command]
pub async fn close_pi_session(
    session_id: String,
    manager: State<'_, PiManagerState>,
) -> AppResult<()> {
    let mgr = manager.0.lock().unwrap();
    mgr.close_session(&session_id)
}

#[tauri::command]
pub async fn get_agent_config(config: State<'_, ConfigState>) -> AppResult<ClaudeConfig> {
    let cfg = config.0.lock().unwrap();
    Ok(cfg.claude_code.clone())
}

#[tauri::command]
pub async fn update_agent_config(
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

#[tauri::command]
pub async fn list_pi_skills(
    skills: State<'_, PiSkillsState>,
) -> AppResult<serde_json::Value> {
    let list = skills.0.list_skills();
    Ok(serde_json::json!(list))
}

#[tauri::command]
pub async fn deploy_pi_skills(skills: State<'_, PiSkillsState>) -> AppResult<()> {
    skills.0.deploy_builtin_skills()
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/commands/pi_commands.rs
git commit -m "feat(pi): add pi_commands with 11 Tauri commands"
```

---

## Task 6: Rewrite `lib.rs` — Wire Up New Modules

**Files:**
- Modify: `src-tauri/src/lib.rs`

Replace all Claude/terminal module references with pi module. Remove old module declarations and command registrations. Add new ones.

- [ ] **Step 1: Rewrite `lib.rs`**

Replace the entire file content with:

```rust
mod ai;
mod commands;
mod config;
mod db;
mod error;
mod logger;
mod models;
mod parser;
mod pi;
mod persona;
mod repositories;

use commands::pi_commands::{
    PiManagerState, PiRuntimeState, PiSkillsState,
};
use config::ConfigState;
use db::DbState;
use pi::PiManager;
use pi::PiRuntime;
use pi::PiSkillsManager;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tracing::info!("OmniLink application starting");
            tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));

            let conn = db::database::init_connection()?;
            db::schema::init_schema(&conn)?;
            tracing::info!("Database initialized successfully");

            config::migrate_ai_config_from_db(&conn)?;
            let app_config = config::load_config()?;
            tracing::info!("Configuration loaded from: ~/.omnilink/config.json");

            if let Err(e) = logger::init_logger(&app_config) {
                eprintln!("Failed to initialize logger: {}", e);
            }

            app.manage(DbState(std::sync::Arc::new(std::sync::Mutex::new(conn))));
            app.manage(ConfigState(std::sync::Mutex::new(app_config)));

            // Initialize pi module
            let skills_manager = PiSkillsManager::new();
            skills_manager.deploy_builtin_skills()?;

            let runtime = PiRuntime::new();
            let manager = PiManager::new(runtime);

            app.manage(PiRuntimeState(std::sync::Arc::new(manager.runtime().clone())));
            app.manage(PiSkillsState(std::sync::Arc::new(skills_manager)));
            app.manage(PiManagerState(std::sync::Mutex::new(manager)));
            tracing::info!("Pi agent modules initialized");

            tracing::info!("OmniLink application setup completed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::note_commands::create_note,
            commands::note_commands::get_note,
            commands::note_commands::list_notes,
            commands::note_commands::update_note,
            commands::note_commands::delete_note,
            commands::note_commands::create_note_from_link,
            commands::settings_commands::get_settings,
            commands::settings_commands::get_ai_config,
            commands::settings_commands::update_ai_config,
            commands::persona_commands::scan_local_personas,
            commands::persona_commands::get_all_personas,
            commands::persona_commands::get_persona_by_skill,
            commands::persona_commands::save_persona,
            commands::persona_commands::delete_persona,
            commands::pi_commands::check_pi_installed,
            commands::pi_commands::install_pi,
            commands::pi_commands::start_pi_session,
            commands::pi_commands::start_pi_output,
            commands::pi_commands::write_pi_input,
            commands::pi_commands::resize_pi_terminal,
            commands::pi_commands::close_pi_session,
            commands::pi_commands::get_agent_config,
            commands::pi_commands::update_agent_config,
            commands::pi_commands::start_persona_rewrite,
            commands::pi_commands::update_session_status,
            commands::pi_commands::list_pi_skills,
            commands::pi_commands::deploy_pi_skills,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Note: `PiRuntime` needs to implement `Clone` for this to work. Add `#[derive(Clone)]` to `PiRuntime` struct in `pi/runtime.rs`.

- [ ] **Step 2: Also need to remove old module declarations from `commands/mod.rs`**

In `src-tauri/src/commands/mod.rs`, remove `pub mod claude_commands;` and `pub mod terminal_commands;`, add `pub mod pi_commands;`.

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles (old files still exist but are no longer referenced)

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/commands/mod.rs
git commit -m "feat(pi): wire up pi module in lib.rs, remove claude/terminal references"
```

---

## Task 7: Create Frontend `usePi.ts` — IPC Wrapper

**Files:**
- Create: `src/composables/usePi.ts`

Replace `useClaude.ts` with pi-specific IPC calls.

- [ ] **Step 1: Write `usePi.ts`**

```typescript
// src/composables/usePi.ts
import { invoke } from '@tauri-apps/api/core'
import type { AgentConfig, PiCliStatus, AgentSkillInfo } from '../types/persona'

export function usePi() {
  const checkInstalled = async (): Promise<PiCliStatus> => {
    return invoke<PiCliStatus>('check_pi_installed')
  }

  const installPi = async (): Promise<PiCliStatus> => {
    return invoke<PiCliStatus>('install_pi')
  }

  const startSession = async (): Promise<string> => {
    return invoke<string>('start_pi_session')
  }

  const startPrintSession = async (prompt: string): Promise<string> => {
    // pi print mode: start a session, then use persona session
    return invoke<string>('start_persona_rewrite', {
      noteId: 0,
      notePath: '',
      personaSkill: prompt,
      mode: 'manual',
    })
  }

  const startOutput = async (sessionId: string): Promise<void> => {
    return invoke('start_pi_output', { sessionId })
  }

  const writeInput = async (sessionId: string, data: string): Promise<void> => {
    return invoke('write_pi_input', { sessionId, data })
  }

  const resizeTerminal = async (sessionId: string, rows: number, cols: number): Promise<void> => {
    return invoke('resize_pi_terminal', { sessionId, rows, cols })
  }

  const closeSession = async (sessionId: string): Promise<void> => {
    return invoke('close_pi_session', { sessionId })
  }

  const getConfig = async (): Promise<AgentConfig> => {
    return invoke<AgentConfig>('get_agent_config')
  }

  const updateConfig = async (params: {
    provider?: string
    apiKey?: string
    baseUrl?: string
    model?: string
  }): Promise<void> => {
    return invoke('update_agent_config', {
      provider: params.provider ?? null,
      apiKey: params.apiKey ?? null,
      baseUrl: params.baseUrl ?? null,
      model: params.model ?? null,
    })
  }

  const listSkills = async (): Promise<AgentSkillInfo[]> => {
    return invoke<AgentSkillInfo[]>('list_pi_skills')
  }

  const deploySkills = async (): Promise<void> => {
    return invoke('deploy_pi_skills')
  }

  return {
    checkInstalled,
    installPi,
    startSession,
    startPrintSession,
    startOutput,
    writeInput,
    resizeTerminal,
    closeSession,
    getConfig,
    updateConfig,
    listSkills,
    deploySkills,
  }
}
```

- [ ] **Step 2: Update `src/types/persona.ts` — Rename Claude types**

Replace the Claude-related types at the bottom of the file:

```typescript
// Replace:
// export interface ClaudeConfig { ... }
// export interface ClaudeCliStatus { ... }
// export interface ClaudeSkillInfo { ... }

// With:
export interface AgentConfig {
  provider: string;
  api_key: string;
  base_url: string;
  model: string;
}

export interface PiCliStatus {
  nodeInstalled: boolean;
  piInstalled: boolean;
  nodeVersion: string | null;
  piVersion: string | null;
}

export interface AgentSkillInfo {
  name: string;
  builtin: boolean;
}
```

- [ ] **Step 3: Commit**

```bash
git add src/composables/usePi.ts src/types/persona.ts
git commit -m "feat(frontend): add usePi composable and rename Claude types to Agent types"
```

---

## Task 8: Update `InteractiveTerminal.vue` — Switch to Pi

**Files:**
- Modify: `src/components/claude/InteractiveTerminal.vue`

Replace all `useClaude` imports/calls with `usePi`. Rename event names from `claude-*` to `pi-*`.

- [ ] **Step 1: Update imports and composable usage**

Replace the import and destructuring block at the top of `<script setup>`:

```typescript
// Replace:
import { useClaude } from '@/composables/useClaude'
// ... and the destructuring:
const {
  checkInstalled,
  installCli,
  startSession,
  startPrintSession: cliStartPrintSession,
  startOutput,
  writeInput,
  resizeTerminal,
  closeSession,
} = useClaude()

// With:
import { usePi } from '@/composables/usePi'
const {
  checkInstalled,
  installPi,
  startSession,
  startOutput,
  writeInput,
  resizeTerminal,
  closeSession,
} = usePi()
```

- [ ] **Step 2: Update `setupStatusListener` — event name**

Replace `'claude-session-status'` with `'pi-session-status'`.

- [ ] **Step 3: Update `setupOutputListener` — event name**

Replace `` `claude-pty-output-${sid}` `` with `` `pi-pty-output-${sid}` ``.

- [ ] **Step 4: Update `startClaude` method**

Rename to `startPi`. Replace `installCli()` call with `installPi()`. Update log messages and terminal write messages from "Claude" to "Pi".

- [ ] **Step 5: Update `startPrintSession` method**

Replace `installCli()` with `installPi()`. Update terminal messages.

- [ ] **Step 6: Update `restart` to call `startPi` instead of `startClaude`**

- [ ] **Step 7: Update `onMounted` to call `startPi` instead of `startClaude`**

- [ ] **Step 8: Verify dev server renders without errors**

Run: `npm run dev`
Expected: Vite compiles, page loads

- [ ] **Step 9: Commit**

```bash
git add src/components/claude/InteractiveTerminal.vue
git commit -m "feat(frontend): migrate InteractiveTerminal from useClaude to usePi"
```

---

## Task 9: Update Remaining Frontend Components

**Files:**
- Modify: `src/components/layout/BottomPanel.vue`
- Modify: `src/components/settings/ClaudeSettings.vue`
- Modify: `src/components/persona/TerminalPanel.vue`
- Modify: `src/components/AppLayout.vue`
- Modify: `src/views/SettingsView.vue`
- Modify: `src/views/NotesView.vue`

- [ ] **Step 1: Update `BottomPanel.vue`**

Changes:
- Replace import of `InteractiveTerminal` (path unchanged, keep `../claude/InteractiveTerminal.vue`)
- Change status bar text from `"Claude Code 就绪"` to `"Pi Agent 就绪"`
- Change panel header from `"Claude Code"` to `"Pi Agent"`

- [ ] **Step 2: Update `ClaudeSettings.vue`**

Changes:
- Replace `import { useClaude } from '@/composables/useClaude'` with `import { usePi } from '@/composables/usePi'`
- Replace `const claude = useClaude()` with `const pi = usePi()`
- Replace all `claude.checkInstalled()` with `pi.checkInstalled()`
- Replace all `claude.installCli()` with `pi.installPi()`
- Replace all `claude.getConfig()` with `pi.getConfig()`
- Replace all `claude.updateConfig(...)` with `pi.updateConfig(...)`
- Change `<CardTitle>` from `"Claude Code"` to `"Pi Agent"`
- Change `<CardDescription>` from `"内置 Claude Code 终端配置"` to `"内置 Pi Agent 终端配置"`
- Change install button text from `"安装 Claude CLI"` to `"安装 Pi CLI"`
- Change save button text from `"保存 Claude 配置"` to `"保存配置"`
- Update status display to show Node.js + pi status instead of just "version"
- Update config reactive default model from `"claude-sonnet-4-20250514"` to `"claude-sonnet-4-20250514"` (keep same)

- [ ] **Step 3: Update `TerminalPanel.vue`**

Changes:
- Replace `import { useClaude } from '@/composables/useClaude'` with `import { usePi } from '@/composables/usePi'`
- Replace `const { resizeTerminal, closeSession } = useClaude()` with `const { resizeTerminal, closeSession } = usePi()`
- Replace event name `` `claude-pty-output-${props.sessionId}` `` with `` `pi-pty-output-${props.sessionId}` ``
- Replace `'claude-session-status'` with `'pi-session-status'`

- [ ] **Step 4: Update `AppLayout.vue`**

Changes:
- Change `provide('claudeTerminal', ...)` to `provide('agentTerminal', ...)`
- Change button title from `"Claude Code"` to `"Pi Agent"`
- Update comments

- [ ] **Step 5: Update `NotesView.vue`**

Changes:
- Replace `inject('claudeTerminal')` with `inject('agentTerminal')`

- [ ] **Step 6: Update `SettingsView.vue`**

Changes:
- Verify import of ClaudeSettings works (path unchanged, component still at `../settings/ClaudeSettings.vue`)
- No code changes needed if file path stays same

- [ ] **Step 7: Verify frontend compiles**

Run: `npm run dev`
Expected: Vite compiles without errors

- [ ] **Step 8: Commit**

```bash
git add src/components/layout/BottomPanel.vue \
        src/components/settings/ClaudeSettings.vue \
        src/components/persona/TerminalPanel.vue \
        src/components/AppLayout.vue \
        src/views/NotesView.vue
git commit -m "feat(frontend): migrate all components from Claude to Pi naming"
```

---

## Task 10: Delete Old Files + Update `persona/builtin.rs`

**Files:**
- Delete: `src-tauri/src/claude/` directory (all 4 files)
- Delete: `src-tauri/src/terminal/` directory (all 3 files)
- Delete: `src-tauri/src/commands/claude_commands.rs`
- Delete: `src-tauri/src/commands/terminal_commands.rs`
- Delete: `src/composables/useClaude.ts`
- Modify: `src-tauri/src/persona/builtin.rs`

- [ ] **Step 1: Update `persona/builtin.rs` — Remove old deployment**

Remove the `initialize_builtin_skills()` function body that deploys to `~/.claude/plugins/cache/omnilink-builtin/skills/`. Replace with a no-op since pi now handles skills via `PiSkillsManager`:

```rust
// src-tauri/src/persona/builtin.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaInfo {
    pub name: String,
    pub skill_name: String,
    pub category: String,
    pub description: String,
}

/// No-op: skills are now managed by PiSkillsManager
pub fn initialize_builtin_skills() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn get_builtin_personas() -> Vec<PersonaInfo> {
    vec![
        PersonaInfo {
            name: "房琪kiki".to_string(),
            skill_name: "fangqikiki-perspective".to_string(),
            category: "travel".to_string(),
            description: "房琪kiki 旅游内容风格".to_string(),
        },
        PersonaInfo {
            name: "雷探长".to_string(),
            skill_name: "leitanzhang-perspective".to_string(),
            category: "adventure".to_string(),
            description: "雷探长探险内容风格".to_string(),
        },
        PersonaInfo {
            name: "花叔".to_string(),
            skill_name: "huashu-perspective".to_string(),
            category: "tech".to_string(),
            description: "花叔技术内容风格".to_string(),
        },
    ]
}
```

- [ ] **Step 2: Delete old files**

```bash
rm -rf src-tauri/src/claude/
rm -rf src-tauri/src/terminal/
rm src-tauri/src/commands/claude_commands.rs
rm src-tauri/src/commands/terminal_commands.rs
rm src/composables/useClaude.ts
```

- [ ] **Step 3: Verify full build**

Run: `cd src-tauri && cargo build`
Expected: compiles with no errors

Run: `npm run build`
Expected: Vite build succeeds

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor: remove Claude Code and old terminal modules, simplify persona builtin"
```

---

## Task 11: End-to-End Smoke Test

**Files:** None (testing only)

- [ ] **Step 1: Start dev server**

Run: `npm run dev:all`

- [ ] **Step 2: Verify Pi Agent settings page loads**

Navigate to Settings → verify "Pi Agent" card shows with Node.js/pi status

- [ ] **Step 3: Click install if needed, verify Node.js + pi CLI install**

- [ ] **Step 4: Open bottom panel, verify pi starts in terminal**

- [ ] **Step 5: Test a simple prompt**

Type "list files in current directory" in the pi terminal, verify response

- [ ] **Step 6: Test skills auto-discovery**

Type `/skill:` in the terminal to see if built-in skills appear

- [ ] **Step 7: Commit final state if any hotfixes were needed**
