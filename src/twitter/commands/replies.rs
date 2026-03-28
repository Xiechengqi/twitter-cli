use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::twitter::extract::required_string;
use crate::twitter::features::TWEET_DETAIL_FEATURES;

const TWEET_DETAIL_FALLBACK_QUERY_ID: &str = "xd_EMdYvB9hfZsZ6Idri0w";

#[derive(Debug, Serialize, Deserialize)]
struct ReplyResult {
    id: String,
    author: String,
    name: String,
    text: String,
    likes: u64,
    retweets: u64,
    created_at: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct RepliesPayload {
    error: Option<String>,
    replies: Option<Vec<ReplyResult>>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let url = required_string(params, "url")?;
    let limit = params
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(20) as usize;

    client.open(&url).await?;
    client.wait_ms(3_000).await?;

    let script = format!(
        r#"(async () => {{
          const tweetUrl = {url:?};
          const limit = {limit};
          const idMatch = tweetUrl.match(/status\/(\d+)/);
          if (!idMatch) {{
            return JSON.stringify({{ error: 'Could not extract tweet ID from URL: ' + tweetUrl }});
          }}
          const focalTweetId = idMatch[1];

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

          const queryId = await resolveQueryId('TweetDetail', {fallback:?});
          const fieldToggles = {{
            withArticleRichContentState: true,
            withArticlePlainText: false,
            withGrokAnalyze: false,
            withDisallowedReplyControls: false,
          }};

          function extractTweet(tweetResult) {{
            const tw = tweetResult?.tweet || tweetResult;
            if (!tw || !tw.legacy) return null;
            const user = tw.core?.user_results?.result;
            const screenName = user?.legacy?.screen_name || 'unknown';
            const displayName = user?.legacy?.name || screenName;
            const legacy = tw.legacy;
            const noteText = tw.note_tweet?.note_tweet_results?.result?.text;
            const text = noteText || legacy.full_text || '';
            return {{
              id: legacy.id_str || '',
              author: screenName,
              name: displayName,
              text: text,
              likes: legacy.favorite_count || 0,
              retweets: legacy.retweet_count || 0,
              created_at: legacy.created_at || '',
              url: 'https://x.com/' + screenName + '/status/' + (legacy.id_str || ''),
            }};
          }}

          const allReplies = [];
          let cursor = null;
          const maxPages = 5;

          for (let page = 0; page < maxPages && allReplies.length < limit; page++) {{
            const variables = {{
              focalTweetId: focalTweetId,
              referrer: 'tweet',
              count: 20,
              with_rux_injections: false,
              rankingMode: 'Relevance',
              includePromotedContent: false,
              withCommunity: true,
              withQuickPromoteEligibilityTweetFields: true,
              withBirdwatchNotes: true,
              withVoice: true,
            }};
            if (cursor) {{
              variables.cursor = cursor;
            }}

            const apiUrl = '/i/api/graphql/' + queryId + '/TweetDetail?variables='
              + encodeURIComponent(JSON.stringify(variables))
              + '&features=' + encodeURIComponent(JSON.stringify({features}))
              + '&fieldToggles=' + encodeURIComponent(JSON.stringify(fieldToggles));

            const resp = await fetch(apiUrl, {{ headers, credentials: 'include' }});
            if (!resp.ok) {{
              if (allReplies.length > 0) break;
              const body = await resp.text().catch(() => '');
              return JSON.stringify({{ error: 'HTTP ' + resp.status + ': ' + body.slice(0, 200) }});
            }}

            const payload = await resp.json();
            const instructions = payload?.data?.threaded_conversation_with_injections_v2?.instructions || [];

            let foundEntries = false;
            for (const instruction of instructions) {{
              const entries = instruction.entries || [];
              for (const entry of entries) {{
                // Bottom cursor for pagination
                if (entry.entryId?.startsWith('cursor-bottom')) {{
                  const cursorValue = entry.content?.itemContent?.value;
                  if (cursorValue) cursor = cursorValue;
                  continue;
                }}

                // Single tweet entry
                if (entry.entryId?.startsWith('tweet-')) {{
                  const tweetResult = entry.content?.itemContent?.tweet_results?.result;
                  if (tweetResult) {{
                    const tweet = extractTweet(tweetResult);
                    if (tweet && tweet.id !== focalTweetId) {{
                      allReplies.push(tweet);
                      foundEntries = true;
                    }}
                  }}
                }}

                // Conversation thread entries (nested replies)
                if (entry.entryId?.startsWith('conversationthread-')) {{
                  const items = entry.content?.items || [];
                  for (const item of items) {{
                    const tweetResult = item.item?.itemContent?.tweet_results?.result;
                    if (tweetResult) {{
                      const tweet = extractTweet(tweetResult);
                      if (tweet && tweet.id !== focalTweetId) {{
                        allReplies.push(tweet);
                        foundEntries = true;
                      }}
                    }}
                  }}
                }}
              }}
            }}

            if (!foundEntries || !cursor) break;
          }}

          return JSON.stringify({{ replies: allReplies.slice(0, limit) }});
        }})()"#,
        url = url,
        limit = limit,
        fallback = TWEET_DETAIL_FALLBACK_QUERY_ID,
        features = TWEET_DETAIL_FEATURES,
    );

    let payload: RepliesPayload = client.eval_json(&script).await?;
    if let Some(error) = payload.error {
        if error.contains("ct0 cookie") {
            return Err(AppError::TwitterLoginRequired);
        }
        return Err(AppError::TwitterRequestFailed(error));
    }

    Ok(json!(payload.replies.unwrap_or_default()))
}
