use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::error::AppResult;

/// 镜像源条目
struct MirrorSource {
    name: &'static str,
    /// GitHub releases 下载 URL 模板，{platform} 和 {arch} 占位符
    url_template: &'static str,
}

/// 内置镜像源列表（优先级从高到低）
const MIRROR_SOURCES: &[MirrorSource] = &[
    MirrorSource {
        name: "ghfast.top",
        url_template: "https://ghfast.top/https://github.com/anthropics/claude-code/releases/latest/download/claude-{platform}-{arch}.{ext}",
    },
    MirrorSource {
        name: "ghproxy.net",
        url_template: "https://ghproxy.net/https://github.com/anthropics/claude-code/releases/latest/download/claude-{platform}-{arch}.{ext}",
    },
    MirrorSource {
        name: "gh-proxy.com",
        url_template: "https://gh-proxy.com/https://github.com/anthropics/claude-code/releases/latest/download/claude-{platform}-{arch}.{ext}",
    },
    MirrorSource {
        name: "github.com (direct)",
        url_template: "https://github.com/anthropics/claude-code/releases/latest/download/claude-{platform}-{arch}.{ext}",
    },
];

/// Claude CLI 安装状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeInstallStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

/// CLI 下载进度事件 payload
#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub phase: String,       // "testing" | "downloading" | "extracting" | "done" | "error"
    pub message: String,
    pub progress: f32,       // 0.0 - 1.0
    pub mirror: Option<String>,
}

/// CLI 下载器 — 管理 Claude Code CLI 二进制的下载和版本检测
pub struct CliDownloader {
    base_dir: PathBuf,
    client: reqwest::Client,
}

impl CliDownloader {
    pub fn new() -> Self {
        let base_dir = dirs::home_dir()
            .expect("Cannot determine home directory")
            .join(".omnilink/claude-cli");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        CliDownloader { base_dir, client }
    }

    /// 获取当前可用的 CLI 路径（~/.omnilink/claude-cli/current/claude）
    pub fn current_cli_path(&self) -> PathBuf {
        self.base_dir.join("current").join("claude")
    }

    /// 检测 Claude CLI 是否已安装，返回版本信息
    pub fn check_installed(&self) -> ClaudeInstallStatus {
        let cli_path = self.current_cli_path();

        if !cli_path.exists() {
            return ClaudeInstallStatus {
                installed: false,
                version: None,
                path: None,
            };
        }

        let version = Command::new(&cli_path)
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|v| v.trim().to_string())
                } else {
                    None
                }
            });

        if version.is_some() {
            ClaudeInstallStatus {
                installed: true,
                version,
                path: Some(cli_path.to_string_lossy().to_string()),
            }
        } else {
            ClaudeInstallStatus {
                installed: false,
                version: None,
                path: None,
            }
        }
    }

    /// 检测首个可用的镜像源
    async fn find_available_mirror(&self) -> AppResult<(String, String)> {
        let (platform, arch, ext) = detect_platform_arch();

        for mirror in MIRROR_SOURCES {
            let url = mirror
                .url_template
                .replace("{platform}", &platform)
                .replace("{arch}", &arch)
                .replace("{ext}", &ext);

            // 只做 HEAD 请求快速测试连通性
            match self
                .client
                .head(&url)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => {
                    tracing::info!("Mirror available: {} (status {})", mirror.name, resp.status());
                    return Ok((mirror.name.to_string(), url));
                }
                Ok(resp) => {
                    tracing::debug!(
                        "Mirror {} returned status {}",
                        mirror.name,
                        resp.status()
                    );
                }
                Err(e) => {
                    tracing::debug!("Mirror {} failed: {}", mirror.name, e);
                }
            }
        }

        Err(crate::error::AppError::Internal(
            "所有镜像源均不可用".to_string(),
        ))
    }

    /// 下载并安装 Claude CLI
    pub async fn install(
        &self,
        app_handle: Option<tauri::AppHandle>,
    ) -> AppResult<ClaudeInstallStatus> {
        let _lock_guard = self.acquire_lock()?;

        // 已安装则直接返回
        let status = self.check_installed();
        if status.installed {
            tracing::info!("Claude CLI already installed: {:?}", status.version);
            return Ok(status);
        }

        // 确保 base_dir 存在
        std::fs::create_dir_all(&self.base_dir)?;

        // 找到可用镜像源并下载
        self.emit_progress(
            &app_handle,
            DownloadProgress {
                phase: "testing".to_string(),
                message: "正在检测可用镜像源...".to_string(),
                progress: 0.0,
                mirror: None,
            },
        );

        let (mirror_name, download_url) = match self.find_available_mirror().await {
            Ok(result) => result,
            Err(e) => {
                self.emit_progress(
                    &app_handle,
                    DownloadProgress {
                        phase: "error".to_string(),
                        message: format!("所有镜像源均不可用: {}", e),
                        progress: 0.0,
                        mirror: None,
                    },
                );
                return Err(e);
            }
        };
        tracing::info!("Downloading Claude CLI from mirror: {}", mirror_name);

        self.emit_progress(
            &app_handle,
            DownloadProgress {
                phase: "downloading".to_string(),
                message: format!("正在从 {} 下载...", mirror_name),
                progress: 0.3,
                mirror: Some(mirror_name.clone()),
            },
        );

        let response = self.client.get(&download_url).send().await?;
        if !response.status().is_success() {
            let msg = format!("Download failed: HTTP {}", response.status());
            self.emit_progress(
                &app_handle,
                DownloadProgress {
                    phase: "error".to_string(),
                    message: msg.clone(),
                    progress: 0.0,
                    mirror: Some(mirror_name),
                },
            );
            return Err(crate::error::AppError::Internal(msg));
        }

        let bytes = response.bytes().await?;

        self.emit_progress(
            &app_handle,
            DownloadProgress {
                phase: "extracting".to_string(),
                message: "正在解压安装...".to_string(),
                progress: 0.8,
                mirror: Some(mirror_name.clone()),
            },
        );

        // 解压归档并提取 claude 二进制
        let current_dir = self.base_dir.join("current");
        std::fs::create_dir_all(&current_dir)?;
        let cli_path = current_dir.join("claude");

        extract_binary(&bytes, &cli_path)?;

        // 设置可执行权限（Unix）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&cli_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&cli_path, perms)?;
        }

        tracing::info!("Claude CLI installed to: {}", cli_path.display());

        // 验证安装
        let status = self.check_installed();
        if status.installed {
            self.emit_progress(
                &app_handle,
                DownloadProgress {
                    phase: "done".to_string(),
                    message: "安装完成".to_string(),
                    progress: 1.0,
                    mirror: Some(mirror_name),
                },
            );
            Ok(status)
        } else {
            self.emit_progress(
                &app_handle,
                DownloadProgress {
                    phase: "error".to_string(),
                    message: "Claude CLI 安装后验证失败".to_string(),
                    progress: 0.0,
                    mirror: Some(mirror_name),
                },
            );
            Err(crate::error::AppError::Internal(
                "Claude CLI 安装后验证失败".to_string(),
            ))
        }
    }

    /// 发送下载进度事件（如果 app_handle 存在）
    fn emit_progress(
        &self,
        app_handle: &Option<tauri::AppHandle>,
        progress: DownloadProgress,
    ) {
        if let Some(handle) = app_handle {
            let _ = handle.emit("cli-download-progress", &progress);
        }
    }

    /// 获取当前使用的镜像源名称（用于设置页面展示）
    pub async fn get_current_mirror(&self) -> String {
        match self.find_available_mirror().await {
            Ok((name, _)) => name,
            Err(_) => "未检测到可用源".to_string(),
        }
    }

    /// 获取基础目录
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// 下载锁（简易文件锁，防止并发下载）
    fn acquire_lock(&self) -> AppResult<LockGuard> {
        let lock_path = self.base_dir.join("download.lock");
        if lock_path.exists() {
            // 检查锁文件是否过期（超过 5 分钟视为过期）
            if let Ok(metadata) = std::fs::metadata(&lock_path) {
                if let Ok(modified) = metadata.modified() {
                    if modified.elapsed().unwrap_or_default() < std::time::Duration::from_secs(300) {
                        return Err(crate::error::AppError::Internal(
                            "另一个下载任务正在进行中".to_string(),
                        ));
                    }
                }
            }
        }
        std::fs::create_dir_all(&self.base_dir)?;
        std::fs::write(&lock_path, "")?;
        Ok(LockGuard { lock_path })
    }
}

/// 锁守卫 — Drop 时自动删除锁文件
struct LockGuard {
    lock_path: PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

/// 从归档字节流中提取 claude 二进制到目标路径
fn extract_binary(archive_bytes: &[u8], dest_path: &PathBuf) -> AppResult<()> {
    // 尝试 tar.gz 解压（macOS / Linux）
    let cursor = std::io::Cursor::new(archive_bytes);
    let decoder = GzDecoder::new(cursor);
    let mut archive = tar::Archive::new(decoder);

    for entry in archive.entries().map_err(|e| {
        crate::error::AppError::Internal(format!("读取归档失败: {}", e))
    })? {
        let mut entry = entry.map_err(|e| {
            crate::error::AppError::Internal(format!("读取归档条目失败: {}", e))
        })?;

        let entry_path = entry.path().map_err(|e| {
            crate::error::AppError::Internal(format!("获取条目路径失败: {}", e))
        })?;

        // 只提取名为 "claude" 的顶层文件
        if entry_path.to_string_lossy() == "claude" {
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents).map_err(|e| {
                crate::error::AppError::Internal(format!("解压 claude 二进制失败: {}", e))
            })?;
            std::fs::write(dest_path, &contents)?;
            return Ok(());
        }
    }

    // tar.gz 中未找到 claude，尝试 zip（Windows）
    extract_binary_from_zip(archive_bytes, dest_path)
}

/// 从 zip 归档中提取 claude 二进制（Windows）
fn extract_binary_from_zip(archive_bytes: &[u8], dest_path: &PathBuf) -> AppResult<()> {
    let reader = std::io::Cursor::new(archive_bytes);
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| {
        crate::error::AppError::Internal(format!("读取 zip 归档失败: {}", e))
    })?;

    // 查找 claude.exe 或 claude
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(|e| {
            crate::error::AppError::Internal(format!("读取 zip 条目失败: {}", e))
        })?;

        let name = file.name().to_string();
        if name == "claude" || name == "claude.exe" {
            let mut contents = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut contents).map_err(|e| {
                crate::error::AppError::Internal(format!("解压 claude 二进制失败: {}", e))
            })?;
            std::fs::write(dest_path, &contents)?;
            return Ok(());
        }
    }

    Err(crate::error::AppError::Internal(
        "归档中未找到 claude 二进制文件".to_string(),
    ))
}

/// 检测当前平台、架构和归档扩展名，返回用于下载 URL 的标识
fn detect_platform_arch() -> (String, String, String) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let (platform, ext) = match os {
        "macos" => ("darwin", "tar.gz"),
        "linux" => ("linux", "tar.gz"),
        "windows" => ("win32", "zip"),
        _ => (os, "tar.gz"),
    };

    let arch = match arch {
        "x86_64" | "amd64" => "x64",
        "aarch64" | "arm64" => "arm64",
        _ => arch,
    };

    (platform.to_string(), arch.to_string(), ext.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_platform_arch() {
        let (platform, arch, ext) = detect_platform_arch();
        assert!(
            ["darwin", "linux", "win32"].contains(&platform.as_str()),
            "Unexpected platform: {}",
            platform
        );
        assert!(
            ["x64", "arm64"].contains(&arch.as_str()),
            "Unexpected arch: {}",
            arch
        );
        assert!(
            ["tar.gz", "zip"].contains(&ext.as_str()),
            "Unexpected ext: {}",
            ext
        );
    }

    #[test]
    fn test_current_cli_path() {
        let downloader = CliDownloader::new();
        let path = downloader.current_cli_path();
        assert!(path.ends_with("claude"));
        assert!(path.to_string_lossy().contains(".omnilink/claude-cli/current"));
    }

    #[test]
    fn test_check_installed_when_missing() {
        let downloader = CliDownloader::new();
        let status = downloader.check_installed();
        // 在开发环境中通常不会安装 CLI
        assert!(!status.installed || status.version.is_some());
    }
}
