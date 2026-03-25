use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};

const BOOKMARKS_QUERY_ID: &str = "Fy0QMy4q_aZCpkO0PnyLYw";
const BOOKMARKS_FEATURES: &str = r#"{
  "rweb_video_screen_enabled": false,
  "profile_label_improvements_pcf_label_in_post_enabled": true,
  "responsive_web_profile_redirect_enabled": false,
  "rweb_tipjar_consumption_enabled": false,
  "verified_phone_label_enabled": false,
  "creator_subscriptions_tweet_preview_api_enabled": true,
  "responsive_web_graphql_timeline_navigation_enabled": true,
  "responsive_web_graphql_skip_user_profile_image_extensions_enabled": false,
  "premium_content_api_read_enabled": false,
  "communities_web_enable_tweet_community_results_fetch": true,
  "c9s_tweet_anatomy_moderator_badge_enabled": true,
  "articles_preview_enabled": true,
  "responsive_web_edit_tweet_api_enabled": true,
  "graphql_is_translatable_rweb_tweet_is_translatable_enabled": true,
  "view_counts_everywhere_api_enabled": true,
  "longform_notetweets_consumption_enabled": true,
  "responsive_web_twitter_article_tweet_consumption_enabled": true,
  "tweet_awards_web_tipping_enabled": false,
  "content_disclosure_indicator_enabled": true,
  "content_disclosure_ai_generated_indicator_enabled": true,
  "freedom_of_speech_not_reach_fetch_enabled": true,
  "standardized_nudges_misinfo": true,
  "tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled": true,
  "longform_notetweets_rich_text_read_enabled": true,
  "longform_notetweets_inline_media_enabled": false,
  "responsive_web_enhance_cards_enabled": false
}"#;

#[derive(Debug, Serialize, Deserialize)]
struct BookmarkTweet {
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
struct BookmarksPayload {
    error: Option<String>,
    tweets: Option<Vec<BookmarkTweet>>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(20);
    if limit == 0 {
        return Err(AppError::InvalidParams(
            "`limit` must be greater than 0".to_string(),
        ));
    }

    client.open("https://x.com").await?;
    client.wait_ms(3_000).await?;

    let script = format!(
        r#"(async () => {{
          const ct0 = document.cookie.split(';').map(c => c.trim()).find(c => c.startsWith('ct0='))?.split('=')[1] || null;
          if (!ct0) {{
            return JSON.stringify({{ error: 'Not logged into x.com (no ct0 cookie)' }});
          }}

          async function resolveQueryId(fallbackId) {{
            try {{
              const ghResp = await fetch('https://raw.githubusercontent.com/fa0311/twitter-openapi/refs/heads/main/src/config/placeholder.json');
              if (ghResp.ok) {{
                const data = await ghResp.json();
                const entry = data['Bookmarks'];
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
                  const match = text.match(/queryId:"([A-Za-z0-9_-]+)"[^}}]{{0,200}}operationName:"Bookmarks"/);
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

          function parseBookmarks(data, seen) {{
            const tweets = [];
            let nextCursor = null;
            const instructions =
              data?.data?.bookmark_timeline_v2?.timeline?.instructions ||
              data?.data?.bookmark_timeline?.timeline?.instructions ||
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

          const queryId = await resolveQueryId({fallback_query_id:?});
          const headers = {{
            Authorization: 'Bearer ' + decodeURIComponent('AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA'),
            'X-Csrf-Token': ct0,
            'X-Twitter-Auth-Type': 'OAuth2Session',
            'X-Twitter-Active-User': 'yes',
          }};

          const allTweets = [];
          const seen = new Set();
          let cursor = null;

          for (let i = 0; i < 5 && allTweets.length < {limit}; i++) {{
            const fetchCount = Math.min(100, {limit} - allTweets.length + 10);
            const variables = {{
              count: fetchCount,
              includePromotedContent: false,
            }};
            if (cursor) variables.cursor = cursor;

            const apiUrl = '/i/api/graphql/' + queryId + '/Bookmarks?variables='
              + encodeURIComponent(JSON.stringify(variables))
              + '&features=' + encodeURIComponent(JSON.stringify({features}));

            const response = await fetch(apiUrl, {{
              headers,
              credentials: 'include'
            }});

            if (!response.ok) {{
              return JSON.stringify({{
                error: 'HTTP ' + response.status + ': Failed to fetch bookmarks. queryId may have expired.'
              }});
            }}

            const data = await response.json();
            const parsed = parseBookmarks(data, seen);
            allTweets.push(...parsed.tweets);
            if (!parsed.nextCursor || parsed.nextCursor === cursor) break;
            cursor = parsed.nextCursor;
          }}

          return JSON.stringify({{ tweets: allTweets.slice(0, {limit}) }});
        }})()"#,
        fallback_query_id = BOOKMARKS_QUERY_ID,
        limit = limit,
        features = BOOKMARKS_FEATURES,
    );

    let payload: BookmarksPayload = client.eval_json(&script).await?;
    if let Some(error) = payload.error {
        if error.contains("ct0 cookie") {
            return Err(AppError::TwitterLoginRequired);
        }
        return Err(AppError::TwitterRequestFailed(error));
    }

    Ok(json!(payload.tweets.unwrap_or_default()))
}
