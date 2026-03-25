use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::errors::{AppError, AppResult};

const CONFIG_DIR_NAME: &str = "twitter-cli";
const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub password: String,
    pub password_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBrowserConfig {
    pub binary: String,
    pub cdp_url: String,
    pub session_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VncConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub embed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub agent_browser: AgentBrowserConfig,
    pub vnc: VncConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 12233,
            },
            auth: AuthConfig {
                password: String::new(),
                password_changed: false,
            },
            agent_browser: AgentBrowserConfig {
                binary: "agent-browser".to_string(),
                cdp_url: String::new(),
                session_name: "twitter-cli".to_string(),
            },
            vnc: VncConfig {
                url: String::new(),
                username: String::new(),
                password: String::new(),
                embed: true,
            },
        }
    }
}

impl AppConfig {
    pub fn is_password_initialized(&self) -> bool {
        !self.auth.password.is_empty() && self.auth.password_changed
    }

    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.server.host, self.server.port)
    }
}

pub fn config_dir() -> AppResult<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::ConfigReadFailed("home directory not found".to_string()))?;
    Ok(home.join(".config").join(CONFIG_DIR_NAME))
}

pub fn config_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join(CONFIG_FILE_NAME))
}

pub async fn detect_agent_browser_binary() -> String {
    match tokio::process::Command::new("which")
        .arg("agent-browser")
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if value.is_empty() {
                "agent-browser".to_string()
            } else {
                value
            }
        }
        _ => "agent-browser".to_string(),
    }
}

pub async fn load_or_init() -> AppResult<(AppConfig, PathBuf, bool)> {
    let path = config_path()?;
    if fs::try_exists(&path)
        .await
        .map_err(|err| AppError::ConfigReadFailed(err.to_string()))?
    {
        let raw = fs::read_to_string(&path)
            .await
            .map_err(|err| AppError::ConfigReadFailed(err.to_string()))?;
        let mut config = toml::from_str::<AppConfig>(&raw)
            .map_err(|err| AppError::ConfigReadFailed(err.to_string()))?;
        if config.server.host == "127.0.0.1" {
            config.server.host = "0.0.0.0".to_string();
            save(&path, &config).await?;
        }
        return Ok((config, path, false));
    }

    let mut config = AppConfig::default();
    config.agent_browser.binary = detect_agent_browser_binary().await;
    save(&path, &config).await?;
    Ok((config, path, true))
}

pub async fn save(path: &Path, config: &AppConfig) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|err| AppError::ConfigWriteFailed(err.to_string()))?;
    }

    let temp_path = path.with_extension("toml.tmp");
    let content = toml::to_string_pretty(config)
        .map_err(|err| AppError::ConfigWriteFailed(err.to_string()))?;

    fs::write(&temp_path, content)
        .await
        .map_err(|err| AppError::ConfigWriteFailed(err.to_string()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let permissions = std::fs::Permissions::from_mode(0o600);
        fs::set_permissions(&temp_path, permissions)
            .await
            .map_err(|err| AppError::ConfigWriteFailed(err.to_string()))?;
    }

    fs::rename(&temp_path, path)
        .await
        .map_err(|err| AppError::ConfigWriteFailed(err.to_string()))?;

    Ok(())
}
