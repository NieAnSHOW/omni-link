use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

use crate::error::{AppError, AppResult};
use super::session::PtySession;

pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_session(
        &self,
        _app_handle: AppHandle,
        session_id: String,
        note_id: i64,
        note_path: &str,
        persona_skill: &str,
    ) -> AppResult<()> {
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

        let command = format!(
            "使用 {} 重构 {}，直接覆盖内容",
            persona_skill, note_path
        );

        let mut cmd = CommandBuilder::new("claude");
        cmd.arg("--dangerously-skip-permissions");
        cmd.arg(&command);

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| AppError::Parse(e.to_string()))?;

        // Take writer for auto-accepting bypass permissions warning
        let mut accept_writer = pair
            .master
            .take_writer()
            .map_err(|e| AppError::Parse(e.to_string()))?;

        let sid_for_accept = session_id.clone();
        std::thread::spawn(move || {
            tracing::info!("[auto-accept] waiting for bypass confirmation for session {}", sid_for_accept);
            std::thread::sleep(std::time::Duration::from_millis(2000));
            // Down arrow to select "Yes, I accept"
            if let Err(e) = accept_writer.write_all(b"\x1b[B") {
                tracing::warn!("[auto-accept] failed to send down arrow: {}", e);
                return;
            }
            let _ = accept_writer.flush();
            std::thread::sleep(std::time::Duration::from_millis(300));
            // Enter to confirm
            if let Err(e) = accept_writer.write_all(b"\r") {
                tracing::warn!("[auto-accept] failed to send enter: {}", e);
                return;
            }
            let _ = accept_writer.flush();
            tracing::info!("[auto-accept] sent acceptance keystrokes for session {}", sid_for_accept);
        });

        let mut session = PtySession {
            id: session_id.clone(),
            master: pair.master,
            child,
            note_id,
            persona_skill: persona_skill.to_string(),
            reader: Some(reader),
            reader_handle: None,
        };

        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id, session);

        Ok(())
    }

    pub fn get_session(&self, session_id: &str) -> Option<String> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).map(|s| s.id.clone())
    }

    pub fn start_reading(
        &self,
        app_handle: tauri::AppHandle,
        session_id: &str,
    ) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.start_output_loop(app_handle)?;
        }
        Ok(())
    }

    pub fn close_session(&self, session_id: &str) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(mut session) = sessions.remove(session_id) {
            let _ = session.child.kill();
            session.stop_reader();
        }
        Ok(())
    }

    pub fn resize_session(&self, session_id: &str, rows: u16, cols: u16) -> AppResult<()> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            session.resize(rows, cols)?;
        }
        Ok(())
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}
