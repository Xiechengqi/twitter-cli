use serde::Deserialize;
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::models::TrendItem;

#[derive(Debug, Deserialize)]
struct TrendingPayload {
    error: Option<String>,
    trends: Option<Vec<TrendItem>>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(20);
    if limit == 0 {
        return Err(AppError::InvalidParams(
            "`limit` must be greater than 0".to_string(),
        ));
    }

    client.open("https://x.com/explore/tabs/trending").await?;
    client.wait_ms(3_000).await?;

    let script = format!(
        r#"(async () => {{
          const ct0 = document.cookie.split(';').map(c => c.trim()).find(c => c.startsWith('ct0='))?.split('=')[1] || null;
          if (!ct0) {{
            return JSON.stringify({{ error: 'Not logged into x.com (no ct0 cookie)' }});
          }}

          let trends = [];
          try {{
            const response = await fetch('/i/api/2/guide.json?include_page_configuration=true', {{
              credentials: 'include',
              headers: {{
                'x-twitter-active-user': 'yes',
                'x-csrf-token': ct0,
                authorization: 'Bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA'
              }}
            }});
            if (response.ok) {{
              const apiData = await response.json();
              const instructions = apiData?.timeline?.instructions || [];
              const entries = instructions.flatMap(inst => inst?.addEntries?.entries || inst?.entries || []);
              const apiTrends = entries
                .filter(entry => entry.content?.timelineModule)
                .flatMap(entry => entry.content.timelineModule.items || [])
                .map(item => item?.item?.content?.trend)
                .filter(Boolean);

              trends = apiTrends.map((trend, index) => ({{
                rank: index + 1,
                topic: trend.name,
                tweets: trend.tweetCount ? String(trend.tweetCount) : 'N/A',
                category: trend.trendMetadata?.domainContext || ''
              }}));
            }}
          }} catch {{}}

          if (trends.length === 0) {{
            const cells = document.querySelectorAll('[data-testid="trend"]');
            cells.forEach((cell) => {{
              const text = cell.textContent || '';
              if (text.includes('Promoted')) return;
              const container = cell.querySelector(':scope > div');
              if (!container) return;
              const divs = container.children;
              const topicEl = divs.length >= 2 ? divs[1] : null;
              const topic = topicEl ? topicEl.textContent.trim() : '';
              const catEl = divs.length >= 1 ? divs[0] : null;
              const catText = catEl ? catEl.textContent.trim() : '';
              const category = catText.replace(/^\\d+\\s*/, '').replace(/^\\xB7\\s*/, '').trim();
              const extraEl = divs.length >= 3 ? divs[2] : null;
              const extra = extraEl ? extraEl.textContent.trim() : '';
              if (topic) {{
                trends.push({{
                  rank: trends.length + 1,
                  topic,
                  tweets: extra || 'N/A',
                  category
                }});
              }}
            }});
          }}

          if (trends.length === 0) {{
            return JSON.stringify({{ error: 'No trends found' }});
          }}

          return JSON.stringify({{ trends: trends.slice(0, {limit}) }});
        }})()"#,
        limit = limit
    );

    let payload: TrendingPayload = client.eval_json(&script).await?;
    if let Some(error) = payload.error {
        if error.contains("ct0 cookie") {
            return Err(AppError::TwitterLoginRequired);
        }
        return Err(AppError::TwitterRequestFailed(error));
    }

    Ok(json!(payload.trends.unwrap_or_default()))
}
