use serde_json::Value;

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};

pub fn normalize_username(value: &str) -> String {
    value.trim().trim_start_matches('@').to_string()
}

pub fn required_string(params: &Value, key: &str) -> AppResult<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| AppError::InvalidParams(format!("`{key}` is required")))
}

/// Detect the currently logged-in username from the Twitter UI.
pub async fn detect_username(client: &AgentBrowserClient) -> AppResult<String> {
    let detected: String = client
        .eval_json(
            r#"JSON.stringify((() => {
                const link = document.querySelector('a[data-testid="AppTabBar_Profile_Link"]');
                return link ? (link.getAttribute('href') || '').replace(/^\//, '') : '';
            })())"#,
        )
        .await?;
    let username = normalize_username(&detected);
    if username.is_empty() {
        return Err(AppError::TwitterLoginRequired);
    }
    Ok(username)
}
