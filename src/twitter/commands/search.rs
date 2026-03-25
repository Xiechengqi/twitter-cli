use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
struct SearchTweet {
    id: String,
    author: String,
    text: String,
    likes: String,
    views: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct SearchPayload {
    error: Option<String>,
    tweets: Option<Vec<SearchTweet>>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let query = params
        .get("query")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    if query.is_empty() {
        return Err(AppError::InvalidParams("`query` is required".to_string()));
    }

    let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(15);
    if limit == 0 {
        return Err(AppError::InvalidParams(
            "`limit` must be greater than 0".to_string(),
        ));
    }

    client
        .open(&format!("https://x.com/search?q={query}&f=top"))
        .await?;
    client.wait_ms(4_000).await?;

    let script = format!(
        r#"JSON.stringify((() => {{
          const articles = Array.from(document.querySelectorAll('article[data-testid="tweet"]'));
          if (articles.length === 0) {{
            const ct0 = document.cookie.split(';').map(c => c.trim()).find(c => c.startsWith('ct0='))?.split('=')[1] || null;
            return {{
              error: ct0 ? 'No search results found in DOM' : 'Not logged into x.com (no ct0 cookie)'
            }};
          }}

          const parseCount = (value) => {{
            const text = (value || '').trim();
            return text;
          }};

          const tweets = articles.slice(0, {limit}).map((article) => {{
            const statusLink = article.querySelector('a[href*="/status/"]');
            const url = statusLink?.href || '';
            const idMatch = url.match(/status\/(\d+)/);
            const userHref = Array.from(article.querySelectorAll('a[href]'))
              .map((node) => node.getAttribute('href'))
              .find((href) => href && /^\/[^/]+$/.test(href));
            const author = userHref ? userHref.replace(/^\//, '') : 'unknown';
            const text = article.querySelector('div[data-testid="tweetText"]')?.innerText || '';
            const likes = parseCount(
              article.querySelector('button[data-testid="like"]')?.innerText ||
              article.querySelector('button[data-testid="unlike"]')?.innerText ||
              ''
            );
            const views = parseCount(
              article.querySelector('a[href$="/analytics"]')?.innerText || ''
            );

            return {{
              id: idMatch ? idMatch[1] : '',
              author,
              text,
              likes,
              views,
              url,
            }};
          }}).filter((tweet) => tweet.id && tweet.url);

          return {{ tweets }};
        }})())"#,
        limit = limit
    );

    let payload: SearchPayload = client.eval_json(&script).await?;
    if let Some(error) = payload.error {
        if error.contains("ct0 cookie") {
            return Err(AppError::TwitterLoginRequired);
        }
        return Err(AppError::TwitterRequestFailed(error));
    }

    Ok(json!(payload.tweets.unwrap_or_default()))
}
