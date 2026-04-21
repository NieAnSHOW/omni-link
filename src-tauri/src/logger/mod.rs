pub mod filter;

use crate::config::AppConfig;
use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const MAX_LOG_FILES: usize = 5;

/// 获取日志目录路径
pub fn log_dir() -> PathBuf {
    crate::config::data_dir().join("logs")
}

/// 检查并轮转日志文件（如果超过大小限制）
fn check_and_rotate_logs(log_path: &PathBuf, prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_log = log_path.join(format!("{}.log", prefix));

    if current_log.exists() {
        let metadata = std::fs::metadata(&current_log)?;
        if metadata.len() > MAX_FILE_SIZE {
            // 找到最大的编号
            let mut max_num = 0;
            for entry in std::fs::read_dir(log_path)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        if name_str.starts_with(prefix) && name_str.ends_with(".log") {
                            // 尝试解析编号
                            if let Some(num_str) = name_str.strip_prefix(prefix).and_then(|s| s.strip_suffix(".log")) {
                                if let Ok(num) = num_str.parse::<usize>() {
                                    max_num = max_num.max(num);
                                }
                            }
                        }
                    }
                }
            }

            // 删除最旧的文件（如果达到最大数量）
            if max_num >= MAX_LOG_FILES {
                let oldest = log_path.join(format!("{}{}.log", prefix, MAX_LOG_FILES - 1));
                if oldest.exists() {
                    std::fs::remove_file(oldest)?;
                }
            }

            // 重命名现有文件
            for i in (0..max_num).rev() {
                let old_name = log_path.join(format!("{}{}.log", prefix, i));
                let new_name = log_path.join(format!("{}{}.log", prefix, i + 1));
                if old_name.exists() {
                    std::fs::rename(&old_name, &new_name)?;
                }
            }

            // 重命名当前文件为 .0
            let rotated = log_path.join(format!("{}0.log", prefix));
            std::fs::rename(&current_log, &rotated)?;
        }
    }

    Ok(())
}

/// 初始化日志系统
pub fn init_logger(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 创建日志目录
    let log_path = log_dir();
    std::fs::create_dir_all(&log_path)?;

    // 解析日志等级
    let log_level = config.log.level.to_level_filter();

    // 检查并轮转现有日志文件
    check_and_rotate_logs(&log_path, "error")?;
    check_and_rotate_logs(&log_path, "warn")?;
    check_and_rotate_logs(&log_path, "info")?;
    check_and_rotate_logs(&log_path, "debug")?;

    // 创建按等级分文件的 appender
    let error_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("error")
        .filename_suffix("log")
        .build(&log_path)?;

    let warn_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("warn")
        .filename_suffix("log")
        .build(&log_path)?;

    let info_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("info")
        .filename_suffix("log")
        .build(&log_path)?;

    let debug_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("debug")
        .filename_suffix("log")
        .build(&log_path)?;

    // 创建格式化层
    let error_layer = fmt::layer()
        .with_writer(error_appender)
        .with_ansi(false)
        .with_target(true)
        .with_filter(EnvFilter::new("error"));

    let warn_layer = fmt::layer()
        .with_writer(warn_appender)
        .with_ansi(false)
        .with_target(true)
        .with_filter(EnvFilter::new("warn"));

    let info_layer = fmt::layer()
        .with_writer(info_appender)
        .with_ansi(false)
        .with_target(true)
        .with_filter(EnvFilter::new("info"));

    let debug_layer = fmt::layer()
        .with_writer(debug_appender)
        .with_ansi(false)
        .with_target(true)
        .with_filter(log_level);

    // 组合所有层并初始化
    tracing_subscriber::registry()
        .with(error_layer)
        .with(warn_layer)
        .with(info_layer)
        .with(debug_layer)
        .init();

    tracing::info!("Logger initialized with level: {:?}", config.log.level);

    Ok(())
}
