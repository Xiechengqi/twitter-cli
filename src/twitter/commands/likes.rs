use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::twitter::extract::{detect_username, normalize_username};
use crate::twitter::features::{PROFILE_FEATURES, TWEET_FEATURES};
use crate::twitter::query_ids::user_by_screen_name_fallback;

const LIKES_FALLBACK_QUERY_ID: &str = "eSSNbhECHHBBgt0YSLnBRA";

#[derive(Debug, Serialize, Deserialize)]
struct LikeTweet {
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
struct LikesPayload {
    error: Option<String>,
    tweets: Option<Vec<LikeTweet>>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let mut username = params
        .get("username")
        .and_then(Value::as_str)
        .map(normalize_username)
        .unwrap_or_default();
    let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(20);

    if limit == 0 {
        return Err(AppError::InvalidParams(
            "`limit` must be greater than 0".to_string(),
        ));
    }

    client.open("https://x.com").await?;
    client.wait_ms(3_000).await?;

    if username.is_empty() {
        username = detect_username(client).await?;
    }

    let script = format!(
        r#"(async () => {{
          const ct0 = document.cookie.split(';').map(c => c.trim()).find(c => c.startsWith('ct0='))?.split('=')[1] || null;
          if (!ct0) {{
            return JSON.stringify({{ error: 'Not logged into x.com (no ct0 cookie)' }});
          }}

          // Fetch GitHub lookup table once, reuse for both query ID resolutions
          let ghLookup = null;
          try {{
            const ghResp = await fetch('https://raw.githubusercontent.com/fa0311/twitter-openapi/refs/heads/main/src/config/placeholder.json');
            if (ghResp.ok) ghLookup = await ghResp.json();
          }} catch {{}}

          async function resolveQueryId(operationName, fallbackId) {{
            if (ghLookup) {{
              const entry = ghLookup[operationName];
              if (entry && entry.queryId) return entry.queryId;
            }}
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

          function extractTweet(result, seen) {{
            if (!result) return null;
            const tw = result.tweet || result;
            const legacy = tw.legacy || {{}};
            if (!tw.rest_id || seen.has(tw.rest_id)) return null;
            seen.add(tw.rest_id);

            const user = tw.core?.user_results?.result;
            const screenName = user?.legacy?.screen_name || user?.core?.screen_name || 'unknown';
            const displayName = user?.legacy?.name || user?.core?.name || '';
            const noteText = tw.note_tweet?.note_tweet_results?.result?.text;

            return {{
              id: tw.rest_id,
              author: screenName,
              name: displayName,
              text: noteText || legacy.full_text || '',
              likes: legacy.favorite_count || 0,
              retweets: legacy.retweet_count || 0,
              created_at: legacy.created_at || '',
              url: `https://x.com/${{screenName}}/status/${{tw.rest_id}}`,
            }};
          }}

          function parseLikes(data, seen) {{
            const tweets = [];
            let nextCursor = null;
            const instructions =
              data?.data?.user?.result?.timeline_v2?.timeline?.instructions ||
              data?.data?.user?.result?.timeline?.timeline?.instructions ||
              [];

            for (const inst of instructions) {{
              for (const entry of inst.entries || []) {{
                const content = entry.content;
                if (content?.entryType === 'TimelineTimelineCursor' || content?.__typename === 'TimelineTimelineCursor') {{
                  if (content.cursorType === 'Bottom' || content.cursorType === 'ShowMore') nextCursor = content.value;
                  continue;
                }}
                if (entry.entryId?.startsWith('cursor-bottom-') || entry.entryId?.startsWith('cursor-showMore-')) {{
                  nextCursor = content?.value || content?.itemContent?.value || nextCursor;
                  continue;
                }}

                const direct = extractTweet(content?.itemContent?.tweet_results?.result, seen);
                if (direct) {{
                  tweets.push(direct);
                  continue;
                }}

                for (const item of content?.items || []) {{
                  const nested = extractTweet(item.item?.itemContent?.tweet_results?.result, seen);
                  if (nested) tweets.push(nested);
                }}
              }}
            }}

            return {{ tweets, nextCursor }};
          }}

          // Resolve user ID first
          const profileQueryId = await resolveQueryId('UserByScreenName', {profile_fallback:?});
          const profileVars = JSON.stringify({{
            screen_name: {username:?},
            withSafetyModeUserFields: true,
          }});
          const profileUrl = '/i/api/graphql/' + profileQueryId + '/UserByScreenName?variables='
            + encodeURIComponent(profileVars)
            + '&features=' + encodeURIComponent(JSON.stringify({profile_features}));

          const headers = {{
            Authorization: 'Bearer ' + decodeURIComponent('AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA'),
            'X-Csrf-Token': ct0,
            'X-Twitter-Auth-Type': 'OAuth2Session',
            'X-Twitter-Active-User': 'yes',
          }};

          const profileResp = await fetch(profileUrl, {{ headers, credentials: 'include' }});
          if (!profileResp.ok) {{
            return JSON.stringify({{ error: 'HTTP ' + profileResp.status + ': Failed to resolve user.' }});
          }}
          const profileData = await profileResp.json();
          const userId = profileData?.data?.user?.result?.rest_id;
          if (!userId) {{
            return JSON.stringify({{ error: 'Could not resolve user ID for @' + {username:?} }});
          }}

          const queryId = await resolveQueryId('Likes', {fallback_query_id:?});

          const allTweets = [];
          const seen = new Set();
          let cursor = null;

          for (let i = 0; i < 5 && allTweets.length < {limit}; i++) {{
            const fetchCount = Math.min(100, {limit} - allTweets.length + 10);
            const variables = {{
              userId: userId,
              count: fetchCount,
              includePromotedContent: false,
            }};
            if (cursor) variables.cursor = cursor;

            const apiUrl = '/i/api/graphql/' + queryId + '/Likes?variables='
              + encodeURIComponent(JSON.stringify(variables))
              + '&features=' + encodeURIComponent(JSON.stringify({features}));

            const response = await fetch(apiUrl, {{
              headers,
              credentials: 'include'
            }});

            if (!response.ok) {{
              return JSON.stringify({{
                error: 'HTTP ' + response.status + ': Failed to fetch likes. queryId may have expired.'
              }});
            }}

            const data = await response.json();
            const parsed = parseLikes(data, seen);
            allTweets.push(...parsed.tweets);
            if (!parsed.nextCursor || parsed.nextCursor === cursor) break;
            cursor = parsed.nextCursor;
          }}

          return JSON.stringify({{ tweets: allTweets.slice(0, {limit}) }});
        }})()"#,
        profile_fallback = user_by_screen_name_fallback(),
        username = username,
        profile_features = PROFILE_FEATURES,
        fallback_query_id = LIKES_FALLBACK_QUERY_ID,
        limit = limit,
        features = TWEET_FEATURES,
    );

    let payload: LikesPayload = client.eval_json(&script).await?;
    if let Some(error) = payload.error {
        if error.contains("ct0 cookie") {
            return Err(AppError::TwitterLoginRequired);
        }
        return Err(AppError::TwitterRequestFailed(error));
    }

    Ok(json!(payload.tweets.unwrap_or_default()))
}
