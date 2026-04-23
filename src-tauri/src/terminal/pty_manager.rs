use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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

        let cmd = CommandBuilder::new("claude");
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| AppError::Parse(e.to_string()))?;

        let mut session = PtySession {
            id: session_id.clone(),
            master: pair.master,
            child,
            note_id,
            persona_skill: persona_skill.to_string(),
        };

        let command = format!(
            "使用 {} 重构 {}，直接覆盖内容",
            persona_skill, note_path
        );
        session.write_command(&command)?;

        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id, session);

        Ok(())
    }

    pub fn get_session(&self, session_id: &str) -> Option<String> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).map(|s| s.id.clone())
    }

    pub fn close_session(&self, session_id: &str) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(mut session) = sessions.remove(session_id) {
            let _ = session.child.kill();
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
