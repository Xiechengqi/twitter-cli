use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::twitter::extract::{detect_username, normalize_username};

#[derive(Debug, Serialize, Deserialize)]
struct PersonItem {
    screen_name: String,
    name: String,
    bio: String,
    followers: u64,
}

#[derive(Debug, Deserialize)]
struct PeoplePayload {
    error: Option<String>,
    people: Option<Vec<PersonItem>>,
}

pub async fn execute_followings(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    execute_people(client, params, "following").await
}

pub async fn execute_followers(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    execute_people(client, params, "followers").await
}

async fn execute_people(
    client: &AgentBrowserClient,
    params: &Value,
    relationship: &str,
) -> AppResult<Value> {
    let mut username = params
        .get("username")
        .or_else(|| params.get("user"))
        .and_then(Value::as_str)
        .map(normalize_username)
        .unwrap_or_default();
    let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(50);

    if limit == 0 {
        return Err(AppError::InvalidParams(
            "`limit` must be greater than 0".to_string(),
        ));
    }

    if username.is_empty() {
        client.open("https://x.com/home").await?;
        client.wait_ms(3_000).await?;
        username = detect_username(client).await?;
    }

    client
        .open(&format!("https://x.com/{username}/{relationship}"))
        .await?;
    client.wait_ms(3_000).await?;

    let scrolls = ((limit / 20).max(1)).min(10);
    for _ in 0..scrolls {
        client
            .eval("window.scrollBy(0, window.innerHeight * 2); 'ok'")
            .await?;
        client.wait_ms(1_500).await?;
    }

    let script = format!(
        r#"JSON.stringify((() => {{
          const ct0 = document.cookie.split(';').map(c => c.trim()).find(c => c.startsWith('ct0='))?.split('=')[1] || null;
          if (!ct0) {{
            return {{ error: 'Not logged into x.com (no ct0 cookie)' }};
          }}

          const items = Array.from(document.querySelectorAll('div[data-testid="cellInnerDiv"]'))
            .map((cell) => {{
              const hrefs = Array.from(cell.querySelectorAll('a[href]'))
                .map((node) => node.getAttribute('href'))
                .filter(Boolean);
              const profileHref = hrefs.find((href) => /^\/[^/]+$/.test(href));
              if (!profileHref) return null;

              const lines = (cell.innerText || '')
                .split('\n')
                .map((line) => line.trim())
                .filter(Boolean);
              if (lines.length < 2) return null;

              const name = lines[0] || '';
              const handleLine = lines.find((line) => line.startsWith('@')) || '';
              const bioLines = lines.filter((line, index) => index > 1 && line !== 'Follow' && line !== 'Following');

              return {{
                screen_name: profileHref.replace(/^\//, ''),
                name,
                bio: bioLines.join('\n'),
                followers: 0
              }};
            }})
            .filter(Boolean);

          const seen = new Set();
          const people = [];
          for (const item of items) {{
            if (!item.screen_name || seen.has(item.screen_name)) continue;
            seen.add(item.screen_name);
            people.push(item);
            if (people.length >= {limit}) break;
          }}
          return {{ people }};
        }})())"#,
        limit = limit
    );

    let payload: PeoplePayload = client.eval_json(&script).await?;
    if let Some(error) = payload.error {
        if error.contains("ct0 cookie") {
            return Err(AppError::TwitterLoginRequired);
        }
        return Err(AppError::TwitterRequestFailed(error));
    }

    Ok(json!(payload.people.unwrap_or_default()))
}
