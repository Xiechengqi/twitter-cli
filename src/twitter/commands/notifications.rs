use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
struct NotificationItem {
    action: String,
    author: String,
    text: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct NotificationsPayload {
    error: Option<String>,
    notifications: Option<Vec<NotificationItem>>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(20);
    if limit == 0 {
        return Err(AppError::InvalidParams(
            "`limit` must be greater than 0".to_string(),
        ));
    }

    client.open("https://x.com/notifications").await?;
    client.wait_ms(5_000).await?;

    let scrolls = ((limit / 10).max(1)).min(10);
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

          const cells = Array.from(document.querySelectorAll('div[data-testid="cellInnerDiv"]'));
          const notifications = [];
          const seen = new Set();

          for (const cell of cells) {{
            if (notifications.length >= {limit}) break;

            const links = Array.from(cell.querySelectorAll('a[href]'))
              .map(a => a.getAttribute('href'))
              .filter(Boolean);

            const tweetLink = links.find(h => /^\/[^/]+\/status\/\d+/.test(h));
            const profileLink = links.find(h => /^\/[^/]+$/.test(h) && h !== '/notifications');

            const lines = (cell.innerText || '')
              .split('\n')
              .map(l => l.trim())
              .filter(Boolean);

            if (lines.length === 0) continue;

            // Detect action type from text content
            const fullText = lines.join(' ').toLowerCase();
            let action = 'notification';
            if (fullText.includes('liked')) action = 'like';
            else if (fullText.includes('retweeted') || fullText.includes('reposted')) action = 'retweet';
            else if (fullText.includes('followed')) action = 'follow';
            else if (fullText.includes('replied')) action = 'reply';
            else if (fullText.includes('mentioned')) action = 'mention';

            const author = profileLink ? profileLink.replace(/^\//, '') : (lines[0] || '');
            const text = lines.slice(0, 3).join(' ');
            const url = tweetLink
              ? 'https://x.com' + tweetLink
              : profileLink
                ? 'https://x.com' + profileLink
                : '';

            const key = action + ':' + author + ':' + url;
            if (seen.has(key)) continue;
            seen.add(key);

            notifications.push({{ action, author, text, url }});
          }}

          return {{ notifications }};
        }})())"#,
        limit = limit
    );

    let payload: NotificationsPayload = client.eval_json(&script).await?;
    if let Some(error) = payload.error {
        if error.contains("ct0 cookie") {
            return Err(AppError::TwitterLoginRequired);
        }
        return Err(AppError::TwitterRequestFailed(error));
    }

    Ok(json!(payload.notifications.unwrap_or_default()))
}
