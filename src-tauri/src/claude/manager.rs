use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

use crate::claude::cli_downloader::CliDownloader;
use crate::claude::skills::SkillsManager;
use crate::config::{load_config, ClaudeConfig};
use crate::error::{AppError, AppResult};

/// 单个 Claude PTY 会话
pub struct ClaudeSession {
    pub id: String,
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    pub child: Box<dyn portable_pty::Child + Send + Sync>,
    pub reader: Option<Box<dyn std::io::Read + Send>>,
    pub reader_handle: Option<std::thread::JoinHandle<()>>,
}

/// Claude Code 进程管理器
pub struct ClaudeManager {
    sessions: Arc<Mutex<HashMap<String, ClaudeSession>>>,
    downloader: Arc<CliDownloader>,
    skills: Arc<SkillsManager>,
}

impl ClaudeManager {
    pub fn new(downloader: CliDownloader, skills: SkillsManager) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            downloader: Arc::new(downloader),
            skills: Arc::new(skills),
        }
    }

    /// 获取当前 CLI 路径（如果已安装）
    pub fn cli_path(&self) -> AppResult<std::path::PathBuf> {
        let status = self.downloader.check_installed();
        if status.installed {
            Ok(self.downloader.current_cli_path())
        } else {
            Err(AppError::Internal("Claude CLI 未安装".to_string()))
        }
    }

    /// 创建新的 Claude PTY 会话
    pub fn create_session(
        &self,
        app_handle: tauri::AppHandle,
        session_id: String,
    ) -> AppResult<()> {
        let cli_path = self.cli_path()?;

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::Internal(format!("PTY 创建失败: {}", e)))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| AppError::Internal(format!("PTY reader 克隆失败: {}", e)))?;

        let config = load_config()?;
        let skills_dir = self.skills.skills_deploy_dir();

        // 构建 Claude CLI 命令
        let mut cmd = CommandBuilder::new(cli_path.as_os_str());

        // 注入 Skills 目录
        if skills_dir.exists() {
            cmd.arg("--add-dir");
            cmd.arg(skills_dir.as_os_str());
        }

        // 注入配置环境变量
        Self::apply_config_env(&mut cmd, &config.claude_code);

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| AppError::Internal(format!("Claude CLI 启动失败: {}", e)))?;

        let session = ClaudeSession {
            id: session_id.clone(),
            master: pair.master,
            child,
            reader: Some(reader),
            reader_handle: None,
        };

        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id.clone(), session);

        tracing::info!("Claude PTY session created: {}", session_id);
        Ok(())
    }

    /// 启动 PTY 输出循环（将 PTY 输出推送到前端）
    pub fn start_output_loop(
        &self,
        app_handle: tauri::AppHandle,
        session_id: &str,
    ) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = match sessions.get_mut(session_id) {
            Some(s) => s,
            None => return Err(AppError::Internal(format!("会话 {} 不存在", session_id))),
        };

        let reader = match session.reader.take() {
            Some(r) => r,
            None => return Ok(()), // reader 已被消费（可能是前端重连）
        };

        let sid = session_id.to_string();
        let mut child_killer = session.child.clone_killer();

        let handle = std::thread::spawn(move || {
            tracing::info!("[claude-pty-reader] started for session {}", sid);
            let mut reader = reader;
            let mut buf = [0u8; 4096];

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        tracing::info!("[claude-pty-reader] EOF for session {}", sid);
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
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => {
                        tracing::error!("[claude-pty-reader] read error for session {}: {}", sid, e);
                        break;
                    }
                }
            }

            let _ = child_killer.kill();
            tracing::info!("[claude-pty-reader] exiting for session {}", sid);
        });

        session.reader_handle = Some(handle);
        Ok(())
    }

    /// 向 PTY 写入用户输入
    pub fn write_input(&self, session_id: &str, data: &str) -> AppResult<()> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AppError::Internal(format!("会话 {} 不存在", session_id)))?;

        let mut writer = session
            .master
            .take_writer()
            .map_err(|e| AppError::Internal(format!("PTY writer 获取失败: {}", e)))?;
        write!(writer, "{}", data)?;
        writer.flush()?;
        Ok(())
    }

    /// 调整 PTY 终端大小
    pub fn resize_session(&self, session_id: &str, rows: u16, cols: u16) -> AppResult<()> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AppError::Internal(format!("会话 {} 不存在", session_id)))?;

        session
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::Internal(format!("PTY resize 失败: {}", e)))?;
        Ok(())
    }

    /// 关闭指定会话
    pub fn close_session(&self, session_id: &str) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(mut session) = sessions.remove(session_id) {
            let _ = session.child.kill();
            if let Some(handle) = session.reader_handle.take() {
                let _ = handle.join();
            }
            tracing::info!("Claude PTY session closed: {}", session_id);
        }
        Ok(())
    }

    /// 关闭所有会话（应用退出时调用）
    pub fn close_all(&self) {
        let mut sessions = self.sessions.lock().unwrap();
        for (id, mut session) in sessions.drain() {
            let _ = session.child.kill();
            if let Some(handle) = session.reader_handle.take() {
                let _ = handle.join();
            }
            tracing::info!("Claude PTY session closed (cleanup): {}", id);
        }
    }

    /// 获取 downloader 引用
    pub fn downloader(&self) -> &CliDownloader {
        &self.downloader
    }

    /// 将 ClaudeConfig 注入为 CommandBuilder 环境变量
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

impl Drop for ClaudeManager {
    fn drop(&mut self) {
        self.close_all();
    }
}
