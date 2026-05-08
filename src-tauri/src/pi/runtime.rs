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
#[derive(Clone)]
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
