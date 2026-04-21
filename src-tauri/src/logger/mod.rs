pub mod filter;

use crate::config::AppConfig;
use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// 获取日志目录路径
pub fn log_dir() -> PathBuf {
    crate::config::data_dir().join("logs")
}

/// 初始化日志系统
pub fn init_logger(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 创建日志目录
    let log_path = log_dir();
    std::fs::create_dir_all(&log_path)?;

    // 解析日志等级
    let log_level = config.log.level.to_level_filter();

    // 创建按等级分文件的 appender
    let error_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER) // 手动控制轮转
        .filename_prefix("error")
        .filename_suffix("log")
        .max_log_files(5)
        .build(&log_path)?;

    let warn_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("warn")
        .filename_suffix("log")
        .max_log_files(5)
        .build(&log_path)?;

    let info_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("info")
        .filename_suffix("log")
        .max_log_files(5)
        .build(&log_path)?;

    let debug_appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("debug")
        .filename_suffix("log")
        .max_log_files(5)
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
