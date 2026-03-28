use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::twitter::extract::required_string;
use crate::twitter::features::TWEET_FEATURES;

const TWEET_RESULT_FALLBACK_QUERY_ID: &str = "7xflPyRiUxGVbJd4uWmbfg";

#[derive(Debug, Serialize, Deserialize)]
struct ArticleResult {
    title: String,
    author: String,
    content: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct ArticlePayload {
    error: Option<String>,
    articles: Option<Vec<ArticleResult>>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let url = required_string(params, "url")?;

    client.open("https://x.com").await?;
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
            withGrokAnalyze: false,
            withDisallowedReplyControls: false,
          }};

          const apiUrl = '/i/api/graphql/' + queryId + '/TweetResultByRestId?variables='
            + encodeURIComponent(JSON.stringify(variables))
            + '&features=' + encodeURIComponent(JSON.stringify({features}))
            + '&fieldToggles=' + encodeURIComponent(JSON.stringify(fieldToggles));

          const resp = await fetch(apiUrl, {{ headers, credentials: 'include' }});
          if (!resp.ok) {{
            return JSON.stringify({{ error: 'HTTP ' + resp.status + ': Failed to fetch tweet.' }});
          }}

          const payload = await resp.json();
          const result = payload?.data?.tweetResult?.result;
          const tw = result?.tweet || result;
          if (!tw) {{
            return JSON.stringify({{ error: 'Tweet not found or unavailable.' }});
          }}

          const user = tw.core?.user_results?.result;
          const screenName = user?.legacy?.screen_name || 'unknown';

          // Try to extract article content from Draft.js rich content
          const articleState = tw.article?.rich_content_state;
          let title = '';
          let content = '';

          if (articleState) {{
            try {{
              const state = typeof articleState === 'string' ? JSON.parse(articleState) : articleState;
              const blocks = state.blocks || [];
              const mdLines = [];
              for (const block of blocks) {{
                const text = block.text || '';
                switch (block.type) {{
                  case 'header-one':
                    if (!title) title = text;
                    mdLines.push('# ' + text);
                    break;
                  case 'header-two':
                    mdLines.push('## ' + text);
                    break;
                  case 'header-three':
                    mdLines.push('### ' + text);
                    break;
                  case 'blockquote':
                    mdLines.push('> ' + text);
                    break;
                  case 'unordered-list-item':
                    mdLines.push('- ' + text);
                    break;
                  case 'ordered-list-item':
                    mdLines.push('1. ' + text);
                    break;
                  case 'code-block':
                    mdLines.push('```\n' + text + '\n```');
                    break;
                  default:
                    mdLines.push(text);
                }}
              }}
              content = mdLines.join('\n\n');
            }} catch {{}}
          }}

          // Fallback to note_tweet or full_text
          if (!content) {{
            const noteText = tw.note_tweet?.note_tweet_results?.result?.text;
            const legacy = tw.legacy || {{}};
            content = noteText || legacy.full_text || '';
          }}

          if (!title) {{
            // Use first line of content as title
            const firstLine = content.split('\n').find(l => l.trim());
            title = firstLine ? firstLine.replace(/^#+\s*/, '').slice(0, 120) : 'Untitled';
          }}

          return JSON.stringify({{
            articles: [{{
              title: title,
              author: screenName,
              content: content,
              url: tweetUrl,
            }}]
          }});
        }})()"#,
        url = url,
        fallback = TWEET_RESULT_FALLBACK_QUERY_ID,
        features = TWEET_FEATURES,
    );

    let payload: ArticlePayload = client.eval_json(&script).await?;
    if let Some(error) = payload.error {
        if error.contains("ct0 cookie") {
            return Err(AppError::TwitterLoginRequired);
        }
        return Err(AppError::TwitterRequestFailed(error));
    }

    Ok(json!(payload.articles.unwrap_or_default()))
}
