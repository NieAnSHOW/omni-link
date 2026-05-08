use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing_subscriber::filter::LevelFilter;

use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    #[default]
    Debug,
    Trace,
}

impl LogLevel {
    /// 转换为 tracing_subscriber 的 LevelFilter
    pub fn to_level_filter(self) -> LevelFilter {
        match self {
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LogConfig {
    pub level: LogLevel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub ai: AiConfig,
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub claude_code: ClaudeConfig,
    #[serde(default)]
    pub workspace_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConfig {
    pub provider: String,
    pub openai: OpenAiConfig,
    pub ollama: OllamaConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAiConfig {
    #[serde(rename = "api_key")]
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaudeConfig {
    #[serde(default = "default_claude_provider")]
    pub provider: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_claude_base_url")]
    pub base_url: String,
    #[serde(default = "default_claude_model")]
    pub model: String,
}

fn default_claude_provider() -> String {
    "anthropic".into()
}

fn default_claude_base_url() -> String {
    "https://api.anthropic.com".into()
}

fn default_claude_model() -> String {
    "claude-sonnet-4-20250514".into()
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        ClaudeConfig {
            provider: default_claude_provider(),
            api_key: String::new(),
            base_url: default_claude_base_url(),
            model: default_claude_model(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            ai: AiConfig {
                provider: "fallback".into(),
                openai: OpenAiConfig {
                    api_key: String::new(),
                    base_url: "https://api.openai.com/v1".into(),
                    model: "gpt-4o-mini".into(),
                },
                ollama: OllamaConfig {
                    base_url: "http://localhost:11434".into(),
                    model: "llama3.2".into(),
                },
            },
            log: LogConfig::default(),
            claude_code: ClaudeConfig::default(),
            workspace_path: None,
        }
    }
}

pub struct ConfigState(pub Mutex<AppConfig>);

pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot determine home directory")
        .join(".omnilink")
}

pub fn config_path() -> PathBuf {
    data_dir().join("config.json")
}

pub fn load_config() -> AppResult<AppConfig> {
    let path = config_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        let config = AppConfig::default();
        save_config(&config)?;
        Ok(config)
    }
}

pub fn save_config(config: &AppConfig) -> AppResult<()> {
    let path = config_path();
    std::fs::create_dir_all(path.parent().unwrap())?;
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn migrate_ai_config_from_db(conn: &rusqlite::Connection) -> AppResult<()> {
    let path = config_path();
    if path.exists() {
        return Ok(());
    }

    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM user_settings WHERE key = 'ai_provider'",
        [],
        |row| row.get(0),
    );

    if let Ok(config_str) = result {
        if let Ok(old) = serde_json::from_str::<serde_json::Value>(&config_str) {
            let provider = old
                .get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("fallback")
                .to_string();

            let get_str = |key: &str, default: &str| {
                old.get(key)
                    .and_then(|v| v.as_str())
                    .unwrap_or(default)
                    .to_string()
            };

            let new_config = AppConfig {
                ai: AiConfig {
                    provider: provider.clone(),
                    openai: OpenAiConfig {
                        api_key: get_str("apiKey", ""),
                        base_url: get_str("baseUrl", "https://api.openai.com/v1"),
                        model: get_str("model", "gpt-4o-mini"),
                    },
                    ollama: OllamaConfig {
                        base_url: get_str("baseUrl", "http://localhost:11434"),
                        model: get_str("model", "llama3.2"),
                    },
                },
                log: LogConfig::default(),
                claude_code: ClaudeConfig::default(),
                workspace_path: None,
            };
            save_config(&new_config)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loglevel_serialization() {
        let config = LogConfig { level: LogLevel::Debug };
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{"level":"DEBUG"}"#);
    }

    #[test]
    fn test_loglevel_deserialization() {
        let json = r#"{"level":"DEBUG"}"#;
        let config: LogConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.level, LogLevel::Debug);
    }

    #[test]
    fn test_all_loglevels() {
        let levels = vec![
            ("ERROR", LogLevel::Error),
            ("WARN", LogLevel::Warn),
            ("INFO", LogLevel::Info),
            ("DEBUG", LogLevel::Debug),
            ("TRACE", LogLevel::Trace),
        ];

        for (json_str, expected) in levels {
            let json = format!(r#"{{"level":"{}"}}"#, json_str);
            let config: LogConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(config.level, expected);
        }
    }

    #[test]
    fn test_appconfig_with_log() {
        let config = AppConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.log.level, LogLevel::Debug);
    }

    #[test]
    fn test_print_default_config() {
        let config = AppConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        eprintln!("\nDefault config JSON:\n{}\n", json);
        assert!(json.contains(r#""level": "DEBUG""#));
    }

    #[test]
    fn test_load_config_with_different_levels() {
        let json = r#"{
            "ai": {
                "provider": "openai",
                "openai": {
                    "api_key": "test-key",
                    "base_url": "https://api.openai.com/v1",
                    "model": "gpt-4"
                },
                "ollama": {
                    "base_url": "http://localhost:11434",
                    "model": "llama3.2"
                }
            },
            "log": {
                "level": "INFO"
            }
        }"#;

        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.log.level, LogLevel::Info);
    }

    #[test]
    fn test_backward_compatibility_missing_log() {
        let json = r#"{
            "ai": {
                "provider": "openai",
                "openai": {
                    "api_key": "test-key",
                    "base_url": "https://api.openai.com/v1",
                    "model": "gpt-4"
                },
                "ollama": {
                    "base_url": "http://localhost:11434",
                    "model": "llama3.2"
                }
            }
        }"#;

        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.log.level, LogLevel::Debug);
    }

    #[test]
    fn test_loglevel_to_level_filter() {
        use tracing_subscriber::filter::LevelFilter;

        assert_eq!(LogLevel::Error.to_level_filter(), LevelFilter::ERROR);
        assert_eq!(LogLevel::Warn.to_level_filter(), LevelFilter::WARN);
        assert_eq!(LogLevel::Info.to_level_filter(), LevelFilter::INFO);
        assert_eq!(LogLevel::Debug.to_level_filter(), LevelFilter::DEBUG);
        assert_eq!(LogLevel::Trace.to_level_filter(), LevelFilter::TRACE);
    }
}
