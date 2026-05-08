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
        _app_handle: tauri::AppHandle,
        session_id: String,
    ) -> AppResult<()> {
        let process_cmd = self.runtime.build_pi_command()
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
        let process_cmd = self.runtime.build_pi_command()
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
        cmd.arg("-p");
        cmd.arg(prompt);
        Self::apply_config_env(&mut cmd, &config.claude_code);

        let child = pair.slave.spawn_command(cmd)
            .map_err(|e| AppError::Internal(format!("pi print mode 启动失败: {}", e)))?;

        let session = PiSession {
            id: session_id.clone(),
            master: pair.master,
            child,
            reader: Some(reader),
            reader_handle: None,
            writer: None,
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
