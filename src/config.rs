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
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_timeout_secs() -> u64 {
    60
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
                timeout_secs: 60,
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

/// Run `agent-browser get cdp-url` and extract the base URL (scheme + host + port).
/// e.g. `ws://127.0.0.1:12345/devtools/browser/abc` → `ws://127.0.0.1:12345`
pub async fn detect_cdp_url(binary: &str) -> Option<String> {
    let output = tokio::process::Command::new(binary)
        .args(["get", "cdp-url"])
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw.is_empty() {
        return None;
    }
    // Extract base: ws://host:port
    extract_cdp_base_url(&raw)
}

fn extract_cdp_base_url(url: &str) -> Option<String> {
    // ws://127.0.0.1:12345/devtools/browser/... → ws://127.0.0.1:12345
    let scheme_end = url.find("://")?;
    let scheme = &url[..scheme_end];
    let after_scheme = &url[scheme_end + 3..];
    let host_port = match after_scheme.find('/') {
        Some(i) => &after_scheme[..i],
        None => after_scheme,
    };
    Some(format!("{scheme}://{host_port}"))
}

/// Run `agent-browser connect <cdp_url>` to reconnect to a new CDP endpoint.
pub async fn reconnect_agent_browser(binary: &str, cdp_url: &str) {
    let _ = tokio::process::Command::new(binary)
        .args(["connect", cdp_url])
        .output()
        .await;
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
        // Auto-detect cdp_url if not configured
        if config.agent_browser.cdp_url.is_empty() {
            if let Some(url) = detect_cdp_url(&config.agent_browser.binary).await {
                config.agent_browser.cdp_url = url;
                let _ = save(&path, &config).await;
            }
        }
        return Ok((config, path, false));
    }

    let mut config = AppConfig::default();
    config.agent_browser.binary = detect_agent_browser_binary().await;
    if let Some(url) = detect_cdp_url(&config.agent_browser.binary).await {
        config.agent_browser.cdp_url = url;
    }
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

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{AppConfig, extract_cdp_base_url, load_or_init};

    #[tokio::test]
    async fn load_existing_config_does_not_mutate_localhost_host() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temp_home = std::env::temp_dir().join(format!("twitter-cli-config-test-{unique}"));
        std::fs::create_dir_all(temp_home.join(".config/twitter-cli"))
            .expect("create temp config dir");
        std::fs::write(
            temp_home.join(".config/twitter-cli/config.toml"),
            r#"[server]
host = "127.0.0.1"
port = 12233

[auth]
password = ""
password_changed = false

[agent_browser]
binary = "agent-browser"
cdp_url = ""
session_name = "twitter-cli"

[vnc]
url = ""
username = ""
password = ""
embed = true
"#,
        )
        .expect("write config");

        let original_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", &temp_home);
        }

        let result = load_or_init().await.expect("load existing config");

        match original_home {
            Some(value) => unsafe {
                std::env::set_var("HOME", value);
            },
            None => unsafe {
                std::env::remove_var("HOME");
            },
        }

        assert_eq!(result.0.server.host, "127.0.0.1");
        let saved = std::fs::read_to_string(temp_home.join(".config/twitter-cli/config.toml"))
            .expect("read config");
        assert!(saved.contains("host = \"127.0.0.1\""));

        let _ = std::fs::remove_dir_all(temp_home);
    }

    #[test]
    fn default_config_uses_public_bind_host() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 12233);
    }

    #[test]
    fn extract_cdp_base_url_strips_path() {
        assert_eq!(
            extract_cdp_base_url("ws://127.0.0.1:12345/devtools/browser/abc-123"),
            Some("ws://127.0.0.1:12345".to_string()),
        );
        assert_eq!(
            extract_cdp_base_url("wss://host:9222/devtools/page/1"),
            Some("wss://host:9222".to_string()),
        );
        assert_eq!(
            extract_cdp_base_url("ws://localhost:9222"),
            Some("ws://localhost:9222".to_string()),
        );
        assert_eq!(extract_cdp_base_url("not-a-url"), None);
    }
}
