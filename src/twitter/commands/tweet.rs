use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::twitter::extract::required_string;
use crate::twitter::features::TWEET_FEATURES;

const TWEET_RESULT_FALLBACK_QUERY_ID: &str = "7xflPyRiUxGVbJd4uWmbfg";

#[derive(Debug, Serialize, Deserialize)]
struct TweetResult {
    id: String,
    author: String,
    name: String,
    text: String,
    likes: u64,
    retweets: u64,
    replies: u64,
    views: String,
    created_at: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct TweetPayload {
    error: Option<String>,
    tweets: Option<Vec<TweetResult>>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let url = required_string(params, "url")?;

    client.open(&url).await?;
    client.wait_ms(3_000).await?;

    let script = format!(
        r#"(async () => {{
          const tweetUrl = {url:?};
          const idMatch = tweetUrl.match(/status\/(\d+)/);
          if (!idMatch) {{
            return JSON.stringify({{ error: 'Could not extract tweet ID from URL: ' + tweetUrl }});
          }}
          const tweetId = idMatch[1];

          const ct0 = document.cookie.split(';').map(c => c.trim()).find(c => c.startsWith('ct0='))?.split('=')[1] || null;
          if (!ct0) {{
            return JSON.stringify({{ error: 'Not logged into x.com (no ct0 cookie)' }});
          }}

          async function resolveQueryId(operationName, fallbackId) {{
            try {{
              const ghResp = await fetch('https://raw.githubusercontent.com/fa0311/twitter-openapi/refs/heads/main/src/config/placeholder.json');
              if (ghResp.ok) {{
                const data = await ghResp.json();
                const entry = data[operationName];
                if (entry && entry.queryId) return entry.queryId;
              }}
            }} catch {{}}
            try {{
              const scripts = performance.getEntriesByType('resource')
                .filter(r => r.name.includes('client-web') && r.name.endsWith('.js'))
                .map(r => r.name);
              for (const scriptUrl of scripts.slice(0, 15)) {{
                try {{
                  const text = await (await fetch(scriptUrl)).text();
                  const re = new RegExp('queryId:"([A-Za-z0-9_-]+)"[^}}]{{0,200}}operationName:"' + operationName + '"');
                  const match = text.match(re);
                  if (match) return match[1];
                }} catch {{}}
              }}
            }} catch {{}}
            return fallbackId;
          }}

          const headers = {{
            Authorization: 'Bearer ' + decodeURIComponent('AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA'),
            'X-Csrf-Token': ct0,
            'X-Twitter-Auth-Type': 'OAuth2Session',
            'X-Twitter-Active-User': 'yes',
          }};

          const queryId = await resolveQueryId('TweetResultByRestId', {fallback:?});
          const variables = {{
            tweetId: tweetId,
            withCommunity: false,
            includePromotedContent: false,
            withVoice: false,
          }};
          const fieldToggles = {{
            withArticleRichContentState: true,
            withArticlePlainText: false,
          }};

          const apiUrl = '/i/api/graphql/' + queryId + '/TweetResultByRestId?variables='
            + encodeURIComponent(JSON.stringify(variables))
            + '&features=' + encodeURIComponent(JSON.stringify({features}))
            + '&fieldToggles=' + encodeURIComponent(JSON.stringify(fieldToggles));

          const resp = await fetch(apiUrl, {{ headers, credentials: 'include' }});
          if (!resp.ok) {{
            const body = await resp.text().catch(() => '');
            return JSON.stringify({{ error: 'HTTP ' + resp.status + ': ' + body.slice(0, 200) }});
          }}

          const payload = await resp.json();
          const result = payload?.data?.tweetResult?.result;
          const tw = result?.tweet || result;
          if (!tw) {{
            return JSON.stringify({{ error: 'Tweet not found or unavailable.' }});
          }}

          const user = tw.core?.user_results?.result;
          const screenName = user?.legacy?.screen_name || 'unknown';
          const displayName = user?.legacy?.name || screenName;
          const legacy = tw.legacy || {{}};

          const noteText = tw.note_tweet?.note_tweet_results?.result?.text;
          const text = noteText || legacy.full_text || '';

          const viewCount = tw.views?.count || '0';

          return JSON.stringify({{
            tweets: [{{
              id: legacy.id_str || tweetId,
              author: screenName,
              name: displayName,
              text: text,
              likes: legacy.favorite_count || 0,
              retweets: legacy.retweet_count || 0,
              replies: legacy.reply_count || 0,
              views: viewCount,
              created_at: legacy.created_at || '',
              url: tweetUrl,
            }}]
          }});
        }})()"#,
        url = url,
        fallback = TWEET_RESULT_FALLBACK_QUERY_ID,
        features = TWEET_FEATURES,
    );

    let payload: TweetPayload = client.eval_json(&script).await?;
    if let Some(error) = payload.error {
        if error.contains("ct0 cookie") {
            return Err(AppError::TwitterLoginRequired);
        }
        return Err(AppError::TwitterRequestFailed(error));
    }

    Ok(json!(payload.tweets.unwrap_or_default()))
}
