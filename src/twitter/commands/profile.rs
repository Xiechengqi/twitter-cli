use serde::Deserialize;
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::models::Account;
use crate::twitter::extract::normalize_username;
use crate::twitter::query_ids::user_by_screen_name_fallback;

const PROFILE_FEATURES: &str = r#"{
  "hidden_profile_subscriptions_enabled": true,
  "rweb_tipjar_consumption_enabled": true,
  "responsive_web_graphql_exclude_directive_enabled": true,
  "verified_phone_label_enabled": false,
  "subscriptions_verification_info_is_identity_verified_enabled": true,
  "subscriptions_verification_info_verified_since_enabled": true,
  "highlights_tweets_tab_ui_enabled": true,
  "responsive_web_twitter_article_notes_tab_enabled": true,
  "subscriptions_feature_can_gift_premium": true,
  "creator_subscriptions_tweet_preview_api_enabled": true,
  "responsive_web_graphql_skip_user_profile_image_extensions_enabled": false,
  "responsive_web_graphql_timeline_navigation_enabled": true
}"#;

#[derive(Debug, Deserialize)]
struct ProfilePayload {
    error: Option<String>,
    hint: Option<String>,
    profiles: Option<Vec<Account>>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let mut username = params
        .get("username")
        .and_then(Value::as_str)
        .map(normalize_username)
        .unwrap_or_default();

    if username.is_empty() {
        client.open("https://x.com/home").await?;
        client.wait_ms(3_000).await?;
        let detected: String = client
            .eval_json(
                r#"JSON.stringify((() => {
                    const link = document.querySelector('a[data-testid="AppTabBar_Profile_Link"]');
                    return link ? (link.getAttribute('href') || '').replace(/^\//, '') : '';
                })())"#,
            )
            .await?;
        username = normalize_username(&detected);
        if username.is_empty() {
            return Err(AppError::TwitterLoginRequired);
        }
    }

    client.open(&format!("https://x.com/{username}")).await?;
    client.wait_ms(3_000).await?;

    let script = format!(
        r#"(async () => {{
            const screenName = {username:?};
            const ct0 = document.cookie.split(';').map(c => c.trim()).find(c => c.startsWith('ct0='))?.split('=')[1];
            if (!ct0) {{
              return JSON.stringify({{ error: 'No ct0 cookie — not logged into x.com' }});
            }}

            const headers = {{
              Authorization: 'Bearer ' + decodeURIComponent('AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA'),
              'X-Csrf-Token': ct0,
              'X-Twitter-Auth-Type': 'OAuth2Session',
              'X-Twitter-Active-User': 'yes'
            }};

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

            const variables = JSON.stringify({{
              screen_name: screenName,
              withSafetyModeUserFields: true
            }});
            const features = JSON.stringify({features});
            const queryId = await resolveQueryId('UserByScreenName', {fallback:?});
            const url = '/i/api/graphql/' + queryId + '/UserByScreenName?variables='
              + encodeURIComponent(variables)
              + '&features=' + encodeURIComponent(features);

            const resp = await fetch(url, {{ headers, credentials: 'include' }});
            if (!resp.ok) {{
              return JSON.stringify({{
                error: 'HTTP ' + resp.status,
                hint: 'User may not exist or queryId expired'
              }});
            }}

            const payload = await resp.json();
            const result = payload.data?.user?.result;
            if (!result) {{
              return JSON.stringify({{
                error: 'User @' + screenName + ' not found'
              }});
            }}

            const legacy = result.legacy || {{}};
            const expandedUrl = legacy.entities?.url?.urls?.[0]?.expanded_url || '';
            return JSON.stringify({{
              profiles: [{{
                screen_name: legacy.screen_name || screenName,
                name: legacy.name || '',
                bio: legacy.description || '',
                location: legacy.location || '',
                url: expandedUrl,
                followers: legacy.followers_count || 0,
                following: legacy.friends_count || 0,
                tweets: legacy.statuses_count || 0,
                likes: legacy.favourites_count || 0,
                verified: result.is_blue_verified || legacy.verified || false,
                created_at: legacy.created_at || ''
              }}]
            }});
        }})()"#,
        username = username,
        features = PROFILE_FEATURES,
        fallback = user_by_screen_name_fallback(),
    );

    let payload: ProfilePayload = client.eval_json(&script).await?;
    if let Some(error) = payload.error {
        if error.contains("No ct0 cookie") {
            return Err(AppError::TwitterLoginRequired);
        }
        return Err(AppError::TwitterRequestFailed(match payload.hint {
            Some(hint) => format!("{error} ({hint})"),
            None => error,
        }));
    }

    Ok(json!(payload.profiles.unwrap_or_default()))
}
