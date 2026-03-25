use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};

const FOR_YOU_QUERY_ID: &str = "c-CzHF1LboFilMpsx4ZCrQ";
const FOLLOWING_QUERY_ID: &str = "BKB7oi212Fi7kQtCBGE4zA";
const TIMELINE_FEATURES: &str = r#"{
  "rweb_video_screen_enabled": false,
  "profile_label_improvements_pcf_label_in_post_enabled": true,
  "rweb_tipjar_consumption_enabled": true,
  "verified_phone_label_enabled": false,
  "creator_subscriptions_tweet_preview_api_enabled": true,
  "responsive_web_graphql_timeline_navigation_enabled": true,
  "responsive_web_graphql_skip_user_profile_image_extensions_enabled": false,
  "premium_content_api_read_enabled": false,
  "communities_web_enable_tweet_community_results_fetch": true,
  "c9s_tweet_anatomy_moderator_badge_enabled": true,
  "responsive_web_grok_analyze_button_fetch_trends_enabled": false,
  "responsive_web_grok_analyze_post_followups_enabled": true,
  "responsive_web_jetfuel_frame": false,
  "responsive_web_grok_share_attachment_enabled": true,
  "articles_preview_enabled": true,
  "responsive_web_edit_tweet_api_enabled": true,
  "graphql_is_translatable_rweb_tweet_is_translatable_enabled": true,
  "view_counts_everywhere_api_enabled": true,
  "longform_notetweets_consumption_enabled": true,
  "responsive_web_twitter_article_tweet_consumption_enabled": true,
  "tweet_awards_web_tipping_enabled": false,
  "responsive_web_grok_show_grok_translated_post": false,
  "responsive_web_grok_analysis_button_from_backend": false,
  "creator_subscriptions_quote_tweet_preview_enabled": false,
  "freedom_of_speech_not_reach_fetch_enabled": true,
  "standardized_nudges_misinfo": true,
  "tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled": true,
  "longform_notetweets_rich_text_read_enabled": true,
  "longform_notetweets_inline_media_enabled": true,
  "responsive_web_grok_image_annotation_enabled": true,
  "responsive_web_enhance_cards_enabled": false
}"#;

#[derive(Debug, Serialize, Deserialize)]
struct TimelineTweet {
    id: String,
    author: String,
    text: String,
    likes: u64,
    retweets: u64,
    replies: u64,
    views: u64,
    created_at: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct TimelinePayload {
    error: Option<String>,
    tweets: Option<Vec<TimelineTweet>>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let timeline_type = params
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("for-you");
    let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(20);

    if limit == 0 {
        return Err(AppError::InvalidParams(
            "`limit` must be greater than 0".to_string(),
        ));
    }

    let (endpoint, method, fallback_query_id) = if timeline_type == "following" {
        ("HomeLatestTimeline", "POST", FOLLOWING_QUERY_ID)
    } else {
        ("HomeTimeline", "GET", FOR_YOU_QUERY_ID)
    };

    client.open("https://x.com").await?;
    client.wait_ms(3_000).await?;

    let script = format!(
        r#"(async () => {{
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
            return fallbackId;
          }}

          function buildVariables(type, count, cursor) {{
            const vars = {{
              count,
              includePromotedContent: false,
              latestControlAvailable: true,
              requestContext: 'launch',
            }};
            if (type === 'for-you') vars.withCommunity = true;
            if (type === 'following') vars.seenTweetIds = [];
            if (cursor) vars.cursor = cursor;
            return vars;
          }}

          function extractTweet(result, seen) {{
            if (!result) return null;
            const tw = result.tweet || result;
            const legacy = tw.legacy || {{}};
            if (!tw.rest_id || seen.has(tw.rest_id)) return null;
            seen.add(tw.rest_id);

            const user = tw.core?.user_results?.result;
            const screenName = user?.legacy?.screen_name || user?.core?.screen_name || 'unknown';
            const noteText = tw.note_tweet?.note_tweet_results?.result?.text;
            const views = tw.views?.count ? parseInt(tw.views.count, 10) : 0;

            return {{
              id: tw.rest_id,
              author: screenName,
              text: noteText || legacy.full_text || '',
              likes: legacy.favorite_count || 0,
              retweets: legacy.retweet_count || 0,
              replies: legacy.reply_count || 0,
              views,
              created_at: legacy.created_at || '',
              url: `https://x.com/${{screenName}}/status/${{tw.rest_id}}`,
            }};
          }}

          function parseTimeline(data, seen) {{
            const tweets = [];
            let nextCursor = null;
            const instructions = data?.data?.home?.home_timeline_urt?.instructions || [];
            for (const inst of instructions) {{
              for (const entry of inst.entries || []) {{
                const content = entry.content;
                if (content?.entryType === 'TimelineTimelineCursor' || content?.__typename === 'TimelineTimelineCursor') {{
                  if (content.cursorType === 'Bottom') nextCursor = content.value;
                  continue;
                }}
                if (entry.entryId?.startsWith('cursor-bottom-')) {{
                  nextCursor = content?.value || nextCursor;
                  continue;
                }}

                const direct = content?.itemContent?.tweet_results?.result;
                if (direct) {{
                  if (content?.itemContent?.promotedMetadata) continue;
                  const tweet = extractTweet(direct, seen);
                  if (tweet) tweets.push(tweet);
                  continue;
                }}

                for (const item of content?.items || []) {{
                  const nested = item.item?.itemContent?.tweet_results?.result;
                  if (!nested) continue;
                  if (item.item?.itemContent?.promotedMetadata) continue;
                  const tweet = extractTweet(nested, seen);
                  if (tweet) tweets.push(tweet);
                }}
              }}
            }}
            return {{ tweets, nextCursor }};
          }}

          const queryId = await resolveQueryId({endpoint:?}, {fallback_query_id:?});
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
            const fetchCount = Math.min(40, {limit} - allTweets.length + 5);
            const variables = buildVariables({timeline_type:?}, fetchCount, cursor);
            const apiUrl = '/i/api/graphql/' + queryId + '/{endpoint}?variables='
              + encodeURIComponent(JSON.stringify(variables))
              + '&features=' + encodeURIComponent(JSON.stringify({features}));

            const response = await fetch(apiUrl, {{
              method: {method:?},
              headers,
              credentials: 'include'
            }});

            if (!response.ok) {{
              return JSON.stringify({{
                error: 'HTTP ' + response.status + ': Failed to fetch timeline. queryId may have expired.'
              }});
            }}

            const data = await response.json();
            const parsed = parseTimeline(data, seen);
            allTweets.push(...parsed.tweets);
            if (!parsed.nextCursor || parsed.nextCursor === cursor) break;
            cursor = parsed.nextCursor;
          }}

          return JSON.stringify({{ tweets: allTweets.slice(0, {limit}) }});
        }})()"#,
        endpoint = endpoint,
        fallback_query_id = fallback_query_id,
        limit = limit,
        timeline_type = timeline_type,
        features = TIMELINE_FEATURES,
        method = method,
    );

    let payload: TimelinePayload = client.eval_json(&script).await?;
    if let Some(error) = payload.error {
        if error.contains("ct0 cookie") {
            return Err(AppError::TwitterLoginRequired);
        }
        return Err(AppError::TwitterRequestFailed(error));
    }

    Ok(json!(payload.tweets.unwrap_or_default()))
}
