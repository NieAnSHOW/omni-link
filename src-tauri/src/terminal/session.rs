use portable_pty::{Child, MasterPty, PtySize};
use std::io::{Read, Write};
use tauri::Emitter;

use crate::error::{AppError, AppResult};

pub struct PtySession {
    pub id: String,
    pub master: Box<dyn MasterPty + Send>,
    pub child: Box<dyn Child + Send + Sync>,
    pub note_id: i64,
    pub persona_skill: String,
    pub reader: Option<Box<dyn Read + Send>>,
    pub reader_handle: Option<std::thread::JoinHandle<()>>,
}

impl PtySession {
    pub fn write_command(&mut self, command: &str) -> AppResult<()> {
        tracing::info!("[pty-session] write_command: {}", command);
        let mut writer = self
            .master
            .take_writer()
            .map_err(|e| AppError::Parse(e.to_string()))?;
        write!(writer, "{}\n", command)?;
        writer.flush()?;
        tracing::info!("[pty-session] write_command flushed successfully");
        Ok(())
    }

    pub fn resize(&self, rows: u16, cols: u16) -> AppResult<()> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::Parse(e.to_string()))?;
        Ok(())
    }

    pub fn start_output_loop(&mut self, app_handle: tauri::AppHandle) -> AppResult<()> {
        let reader = match self.reader.take() {
            Some(r) => r,
            None => return Ok(()),
        };
        let session_id = self.id.clone();
        let mut child_killer = self.child.clone_killer();
        let handle = std::thread::spawn(move || {
            tracing::info!("[pty-reader] thread started for session {}", session_id);
            let mut reader = reader;
            let mut buf = [0u8; 4096];
            let mut total_bytes = 0u64;
            let mut read_count = 0u64;

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        tracing::info!(
                            "[pty-reader] EOF for session {} (total_bytes={}, read_count={})",
                            session_id, total_bytes, read_count
                        );
                        let _ = app_handle.emit(
                            "session-status",
                            serde_json::json!({
                                "sessionId": session_id,
                                "status": "completed",
                            }),
                        );
                        break;
                    }
                    Ok(n) => {
                        total_bytes += n as u64;
                        read_count += 1;
                        if read_count <= 5 {
                            tracing::debug!(
                                "[pty-reader] read {} bytes (total={}) for session {}",
                                n, total_bytes, session_id
                            );
                        }
                        let output = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = app_handle.emit("pty-output", output);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => {
                        tracing::error!("PTY read error for session {}: {}", session_id, e);
                        let _ = app_handle.emit(
                            "session-status",
                            serde_json::json!({
                                "sessionId": session_id,
                                "status": "failed",
                                "error": e.to_string(),
                            }),
                        );
                        break;
                    }
                }
            }

            tracing::info!("[pty-reader] thread exiting for session {}", session_id);
            let _ = child_killer.kill();
        });

        self.reader_handle = Some(handle);
        Ok(())
    }

    pub fn stop_reader(&mut self) {
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }
    }
}
