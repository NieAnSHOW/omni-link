use portable_pty::{Child, MasterPty, PtySize};
use std::io::{Read, Write};
use tauri::Emitter;

use libc;
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
            None => {
                // Reader already taken by a prior start_output_loop call.
                // The frontend is re-connecting after re-mount — re-emit current status
                // in case the original event was emitted before the listener registered.
                let should_emit = match self.child.try_wait() {
                    Ok(Some(_)) => {
                        tracing::info!(
                            "[pty-session] re-emitting completed status for session {} (child already exited)",
                            self.id
                        );
                        true
                    }
                    Ok(None) => false,
                    Err(e) => {
                        // ECHILD: process already reaped by PID monitor thread = exited
                        tracing::info!(
                            "[pty-session] try_wait error for session {}: {} — treating as completed",
                            self.id, e
                        );
                        true
                    }
                };
                if should_emit {
                    let _ = app_handle.emit(
                        "session-status",
                        serde_json::json!({
                            "sessionId": self.id,
                            "status": "completed",
                        }),
                    );
                }
                return Ok(());
            }
        };
        let session_id = self.id.clone();
        let mut child_killer = self.child.clone_killer();
        let child_pid = self.child.process_id();

        let app_for_monitor = app_handle.clone();
        let sid_for_monitor = session_id.clone();

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
                        let output = String::from_utf8_lossy(&buf[..n]).to_string();
                        if read_count <= 5 {
                            let sample: String = output.chars().take(200).collect();
                            tracing::debug!(
                                "[pty-reader] read {} bytes (total={}) for session {}: {:?}",
                                n, total_bytes, session_id, sample
                            );
                        }
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

        // PID-based completion detection: poll child process liveness
        if let Some(pid) = child_pid {
            let app = app_for_monitor;
            let sid = sid_for_monitor;
            std::thread::spawn(move || {
                tracing::info!("[pid-monitor] watching PID {} for session {}", pid, sid);
                let start = std::time::Instant::now();
                let mut log_tick = 0u64;
                loop {
                    let wait_result = unsafe {
                        libc::waitpid(pid as i32, std::ptr::null_mut(), libc::WNOHANG)
                    };
                    match wait_result {
                        0 => {
                            log_tick += 1;
                            if log_tick % 15 == 0 {
                                tracing::debug!(
                                    "[pid-monitor] PID {} still running for session {} (elapsed {:?})",
                                    pid, sid, start.elapsed()
                                );
                            }
                        }
                        p if p == pid as i32 => {
                            tracing::info!("[pid-monitor] PID {} exited for session {}", pid, sid);
                            unsafe { libc::kill(-(pid as i32), libc::SIGKILL); }
                            let _ = app.emit(
                                "session-status",
                                serde_json::json!({
                                    "sessionId": sid,
                                    "status": "completed",
                                }),
                            );
                            break;
                        }
                        -1 => {
                            let err = std::io::Error::last_os_error();
                            if err.raw_os_error() == Some(libc::ECHILD) {
                                tracing::info!(
                                    "[pid-monitor] PID {} not found (ECHILD) for session {}",
                                    pid, sid
                                );
                                let _ = app.emit(
                                    "session-status",
                                    serde_json::json!({
                                        "sessionId": sid,
                                        "status": "completed",
                                    }),
                                );
                                break;
                            }
                            tracing::warn!(
                                "[pid-monitor] waitpid error for PID {} session {}: {}",
                                pid, sid, err
                            );
                        }
                        _ => {
                            tracing::warn!(
                                "[pid-monitor] unexpected waitpid result {} for PID {} session {}",
                                wait_result, pid, sid
                            );
                        }
                    }
                    if start.elapsed() > std::time::Duration::from_secs(120) {
                        tracing::warn!(
                            "[pid-monitor] timeout for session {} after {:?} — killing process group, reporting failure",
                            sid, start.elapsed()
                        );
                        unsafe { libc::kill(-(pid as i32), libc::SIGKILL); }
                        let _ = app.emit(
                            "session-status",
                            serde_json::json!({
                                "sessionId": sid,
                                "status": "failed",
                                "error": format!("执行超时 ({:.0}s)", start.elapsed().as_secs_f64()),
                            }),
                        );
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
                tracing::info!("[pid-monitor] exiting for session {}", sid);
            });
        }

        self.reader_handle = Some(handle);
        Ok(())
    }

    pub fn stop_reader(&mut self) {
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }
    }
}
