use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Debug
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogConfig {
    pub level: LogLevel,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            level: LogLevel::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub ai: AiConfig,
    #[serde(default)]
    pub log: LogConfig,
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
}
