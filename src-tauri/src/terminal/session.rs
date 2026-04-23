use portable_pty::{Child, MasterPty, PtySize};
use std::io::Write;
use crate::error::{AppError, AppResult};

pub struct PtySession {
    pub id: String,
    pub master: Box<dyn MasterPty + Send>,
    pub child: Box<dyn Child + Send + Sync>,
    pub note_id: i64,
    pub persona_skill: String,
}

impl PtySession {
    pub fn write_command(&mut self, command: &str) -> AppResult<()> {
        let mut writer = self
            .master
            .take_writer()
            .map_err(|e| AppError::Parse(e.to_string()))?;
        write!(writer, "{}\n", command)?;
        writer.flush()?;
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
}
