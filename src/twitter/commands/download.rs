use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::twitter::extract::required_string;

#[derive(Debug, Serialize, Deserialize)]
struct MediaItem {
    index: u64,
    r#type: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct DownloadPayload {
    error: Option<String>,
    media: Option<Vec<MediaItem>>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let url = required_string(params, "url")?;
    let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(10);

    if limit == 0 {
        return Err(AppError::InvalidParams(
            "`limit` must be greater than 0".to_string(),
        ));
    }

    // Determine if this is a tweet URL or a profile URL
    let is_tweet = url.contains("/status/");
    let nav_url = if is_tweet {
        url.clone()
    } else {
        // Profile URL — navigate to media tab
        let trimmed = url.trim_end_matches('/');
        if trimmed.ends_with("/media") {
            trimmed.to_string()
        } else {
            format!("{trimmed}/media")
        }
    };

    client.open(&nav_url).await?;
    client.wait_ms(5_000).await?;

    if !is_tweet {
        let scrolls = ((limit / 5).max(1)).min(10);
        for _ in 0..scrolls {
            client
                .eval("window.scrollBy(0, window.innerHeight * 2); 'ok'")
                .await?;
            client.wait_ms(1_500).await?;
        }
    }

    let script = format!(
        r#"JSON.stringify((() => {{
          const ct0 = document.cookie.split(';').map(c => c.trim()).find(c => c.startsWith('ct0='))?.split('=')[1] || null;
          if (!ct0) {{
            return {{ error: 'Not logged into x.com (no ct0 cookie)' }};
          }}

          const media = [];
          const seen = new Set();

          // Collect images
          const images = document.querySelectorAll('img[src*="pbs.twimg.com/media"]');
          for (const img of images) {{
            if (media.length >= {limit}) break;
            let src = img.src || '';
            // Normalize to large size
            src = src.replace(/&name=\w+/, '&name=large');
            if (!src.includes('&name=')) src += '&name=large';
            if (seen.has(src)) continue;
            seen.add(src);
            media.push({{ index: media.length, type: 'image', url: src }});
          }}

          // Collect videos
          const videos = document.querySelectorAll('video[src]');
          for (const vid of videos) {{
            if (media.length >= {limit}) break;
            const src = vid.src || '';
            if (!src || seen.has(src)) continue;
            seen.add(src);
            media.push({{ index: media.length, type: 'video', url: src }});
          }}

          // Also check video poster images and source elements
          const sources = document.querySelectorAll('video source[src]');
          for (const src_el of sources) {{
            if (media.length >= {limit}) break;
            const src = src_el.src || '';
            if (!src || seen.has(src)) continue;
            seen.add(src);
            media.push({{ index: media.length, type: 'video', url: src }});
          }}

          return {{ media }};
        }})())"#,
        limit = limit
    );

    let payload: DownloadPayload = client.eval_json(&script).await?;
    if let Some(error) = payload.error {
        if error.contains("ct0 cookie") {
            return Err(AppError::TwitterLoginRequired);
        }
        return Err(AppError::TwitterRequestFailed(error));
    }

    Ok(json!(payload.media.unwrap_or_default()))
}
