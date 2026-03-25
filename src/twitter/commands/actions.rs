use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::twitter::extract::normalize_username;

#[derive(Debug, Serialize, Deserialize)]
struct UiActionPayload {
    ok: bool,
    message: String,
}

pub async fn execute_like(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let url = required_string(params, "url")?;
    client.open(&url).await?;
    client.wait_ms(5_000).await?;

    run_ui_action(
        client,
        r#"(async () => {
          try {
            let attempts = 0;
            let likeBtn = null;
            let unlikeBtn = null;
            while (attempts < 20) {
              unlikeBtn = document.querySelector('[data-testid="unlike"]');
              likeBtn = document.querySelector('[data-testid="like"]');
              if (unlikeBtn || likeBtn) break;
              await new Promise(resolve => setTimeout(resolve, 500));
              attempts++;
            }
            if (unlikeBtn) {
              return JSON.stringify({ ok: true, message: 'Tweet is already liked.' });
            }
            if (!likeBtn) {
              return JSON.stringify({ ok: false, message: 'Could not find the Like button on this tweet. Are you logged in?' });
            }
            likeBtn.click();
            await new Promise(resolve => setTimeout(resolve, 1000));
            const verifyBtn = document.querySelector('[data-testid="unlike"]');
            if (verifyBtn) {
              return JSON.stringify({ ok: true, message: 'Tweet successfully liked.' });
            }
            return JSON.stringify({ ok: false, message: 'Like action was initiated but UI did not update as expected.' });
          } catch (error) {
            return JSON.stringify({ ok: false, message: String(error) });
          }
        })()"#,
    )
    .await
}

pub async fn execute_unlike(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let url = required_string(params, "url")?;
    client.open(&url).await?;
    client.wait_ms(5_000).await?;

    run_ui_action(
        client,
        r#"(async () => {
          try {
            let attempts = 0;
            let unlikeBtn = null;
            while (attempts < 20) {
              const likeBtn = document.querySelector('[data-testid="like"]');
              if (likeBtn) {
                return JSON.stringify({ ok: true, message: 'Tweet is not liked (already unliked).' });
              }
              unlikeBtn = document.querySelector('[data-testid="unlike"]');
              if (unlikeBtn) break;
              await new Promise(resolve => setTimeout(resolve, 500));
              attempts++;
            }
            if (!unlikeBtn) {
              return JSON.stringify({ ok: false, message: 'Could not find the Unlike button on this tweet. Are you logged in?' });
            }
            unlikeBtn.click();
            await new Promise(resolve => setTimeout(resolve, 1000));
            const verifyBtn = document.querySelector('[data-testid="like"]');
            if (verifyBtn) {
              return JSON.stringify({ ok: true, message: 'Tweet successfully unliked.' });
            }
            return JSON.stringify({ ok: false, message: 'Unlike action was initiated but UI did not update as expected.' });
          } catch (error) {
            return JSON.stringify({ ok: false, message: String(error) });
          }
        })()"#,
    )
    .await
}

pub async fn execute_bookmark(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let url = required_string(params, "url")?;
    client.open(&url).await?;
    client.wait_ms(5_000).await?;

    run_ui_action(
        client,
        r#"(async () => {
          try {
            let attempts = 0;
            let bookmarkBtn = null;
            while (attempts < 20) {
              const removeBtn = document.querySelector('[data-testid="removeBookmark"]');
              if (removeBtn) {
                return JSON.stringify({ ok: true, message: 'Tweet is already bookmarked.' });
              }
              bookmarkBtn = document.querySelector('[data-testid="bookmark"]');
              if (bookmarkBtn) break;
              await new Promise(resolve => setTimeout(resolve, 500));
              attempts++;
            }
            if (!bookmarkBtn) {
              return JSON.stringify({ ok: false, message: 'Could not find Bookmark button. Are you logged in?' });
            }
            bookmarkBtn.click();
            await new Promise(resolve => setTimeout(resolve, 1000));
            const verify = document.querySelector('[data-testid="removeBookmark"]');
            if (verify) {
              return JSON.stringify({ ok: true, message: 'Tweet successfully bookmarked.' });
            }
            return JSON.stringify({ ok: false, message: 'Bookmark action initiated but UI did not update.' });
          } catch (error) {
            return JSON.stringify({ ok: false, message: String(error) });
          }
        })()"#,
    )
    .await
}

pub async fn execute_unbookmark(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let url = required_string(params, "url")?;
    client.open(&url).await?;
    client.wait_ms(5_000).await?;

    run_ui_action(
        client,
        r#"(async () => {
          try {
            let attempts = 0;
            let removeBtn = null;
            while (attempts < 20) {
              const bookmarkBtn = document.querySelector('[data-testid="bookmark"]');
              if (bookmarkBtn) {
                return JSON.stringify({ ok: true, message: 'Tweet is not bookmarked (already removed).' });
              }
              removeBtn = document.querySelector('[data-testid="removeBookmark"]');
              if (removeBtn) break;
              await new Promise(resolve => setTimeout(resolve, 500));
              attempts++;
            }
            if (!removeBtn) {
              return JSON.stringify({ ok: false, message: 'Could not find Remove Bookmark button. Are you logged in?' });
            }
            removeBtn.click();
            await new Promise(resolve => setTimeout(resolve, 1000));
            const verify = document.querySelector('[data-testid="bookmark"]');
            if (verify) {
              return JSON.stringify({ ok: true, message: 'Tweet successfully removed from bookmarks.' });
            }
            return JSON.stringify({ ok: false, message: 'Unbookmark action initiated but UI did not update.' });
          } catch (error) {
            return JSON.stringify({ ok: false, message: String(error) });
          }
        })()"#,
    )
    .await
}

pub async fn execute_follow(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let username = normalize_username(&required_string(params, "username")?);
    if username.is_empty() {
        return Err(AppError::InvalidParams(
            "`username` is required".to_string(),
        ));
    }

    client.open(&format!("https://x.com/{username}")).await?;
    client.wait_ms(5_000).await?;

    let script = format!(
        r#"(async () => {{
          try {{
            let attempts = 0;
            let followBtn = null;
            while (attempts < 20) {{
              const unfollowBtn = document.querySelector('[data-testid$="-unfollow"]');
              if (unfollowBtn) {{
                return JSON.stringify({{ ok: true, message: 'Already following @{username}.' }});
              }}
              followBtn = document.querySelector('[data-testid$="-follow"]');
              if (followBtn) break;
              await new Promise(resolve => setTimeout(resolve, 500));
              attempts++;
            }}
            if (!followBtn) {{
              return JSON.stringify({{ ok: false, message: 'Could not find Follow button. Are you logged in?' }});
            }}
            followBtn.click();
            await new Promise(resolve => setTimeout(resolve, 1500));
            const verify = document.querySelector('[data-testid$="-unfollow"]');
            if (verify) {{
              return JSON.stringify({{ ok: true, message: 'Successfully followed @{username}.' }});
            }}
            return JSON.stringify({{ ok: false, message: 'Follow action initiated but UI did not update.' }});
          }} catch (error) {{
            return JSON.stringify({{ ok: false, message: String(error) }});
          }}
        }})()"#,
        username = username
    );

    run_ui_action(client, &script).await
}

pub async fn execute_unfollow(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let username = normalize_username(&required_string(params, "username")?);
    if username.is_empty() {
        return Err(AppError::InvalidParams(
            "`username` is required".to_string(),
        ));
    }

    client.open(&format!("https://x.com/{username}")).await?;
    client.wait_ms(5_000).await?;

    let script = format!(
        r#"(async () => {{
          try {{
            let attempts = 0;
            let unfollowBtn = null;
            while (attempts < 20) {{
              const followBtn = document.querySelector('[data-testid$="-follow"]');
              if (followBtn) {{
                return JSON.stringify({{ ok: true, message: 'Not following @{username} (already unfollowed).' }});
              }}
              unfollowBtn = document.querySelector('[data-testid$="-unfollow"]');
              if (unfollowBtn) break;
              await new Promise(resolve => setTimeout(resolve, 500));
              attempts++;
            }}
            if (!unfollowBtn) {{
              return JSON.stringify({{ ok: false, message: 'Could not find Unfollow button. Are you logged in?' }});
            }}
            unfollowBtn.click();
            await new Promise(resolve => setTimeout(resolve, 1000));
            const confirmBtn = document.querySelector('[data-testid="confirmationSheetConfirm"]');
            if (confirmBtn) {{
              confirmBtn.click();
              await new Promise(resolve => setTimeout(resolve, 1000));
            }}
            const verify = document.querySelector('[data-testid$="-follow"]');
            if (verify) {{
              return JSON.stringify({{ ok: true, message: 'Successfully unfollowed @{username}.' }});
            }}
            return JSON.stringify({{ ok: false, message: 'Unfollow action initiated but UI did not update.' }});
          }} catch (error) {{
            return JSON.stringify({{ ok: false, message: String(error) }});
          }}
        }})()"#,
        username = username
    );

    run_ui_action(client, &script).await
}

pub async fn execute_post(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let text = required_string(params, "text")?;
    client.open("https://x.com/compose/tweet").await?;
    client.wait_ms(3_000).await?;

    let script = format!(
        r#"(async () => {{
          try {{
            const textToInsert = {text:?};
            const box = document.querySelector('[data-testid="tweetTextarea_0"]');
            if (!box) {{
              return JSON.stringify({{ ok: false, message: 'Could not find the tweet composer text area.' }});
            }}

            box.focus();
            const dataTransfer = new DataTransfer();
            dataTransfer.setData('text/plain', textToInsert);
            box.dispatchEvent(new ClipboardEvent('paste', {{
              clipboardData: dataTransfer,
              bubbles: true,
              cancelable: true
            }}));

            await new Promise(resolve => setTimeout(resolve, 1000));

            const primaryBtn = document.querySelector('[data-testid="tweetButton"]');
            const inlineBtn = document.querySelector('[data-testid="tweetButtonInline"]');
            const btn = (primaryBtn && !primaryBtn.disabled) ? primaryBtn : ((inlineBtn && !inlineBtn.disabled) ? inlineBtn : null);
            if (!btn) {{
              return JSON.stringify({{ ok: false, message: 'Tweet button is disabled or not found.' }});
            }}

            btn.click();
            await new Promise(resolve => setTimeout(resolve, 2000));

            const stillVisible = document.querySelector('[data-testid="tweetTextarea_0"]');
            if (!stillVisible) {{
              return JSON.stringify({{ ok: true, message: 'Tweet posted successfully.' }});
            }}

            return JSON.stringify({{ ok: true, message: 'Tweet action submitted; composer still visible, verify on timeline if needed.' }});
          }} catch (error) {{
            return JSON.stringify({{ ok: false, message: String(error) }});
          }}
        }})()"#,
        text = text
    );

    run_ui_action(client, &script).await
}

pub async fn execute_reply(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let url = required_string(params, "url")?;
    let text = required_string(params, "text")?;
    client.open(&url).await?;
    client.wait_ms(5_000).await?;

    let script = format!(
        r#"(async () => {{
          try {{
            const textToInsert = {text:?};
            const box = document.querySelector('[data-testid="tweetTextarea_0"]');
            if (!box) {{
              return JSON.stringify({{ ok: false, message: 'Could not find the reply text area. Are you logged in?' }});
            }}

            box.focus();
            const dataTransfer = new DataTransfer();
            dataTransfer.setData('text/plain', textToInsert);
            box.dispatchEvent(new ClipboardEvent('paste', {{
              clipboardData: dataTransfer,
              bubbles: true,
              cancelable: true
            }}));

            await new Promise(resolve => setTimeout(resolve, 1000));

            const btn = document.querySelector('[data-testid="tweetButtonInline"]');
            if (!btn || btn.disabled) {{
              return JSON.stringify({{ ok: false, message: 'Reply button is disabled or not found.' }});
            }}

            btn.click();
            await new Promise(resolve => setTimeout(resolve, 2000));

            const stillVisible = document.querySelector('[data-testid="tweetTextarea_0"]');
            if (!stillVisible) {{
              return JSON.stringify({{ ok: true, message: 'Reply posted successfully.' }});
            }}

            return JSON.stringify({{ ok: true, message: 'Reply action submitted; reply box still visible, verify in thread if needed.' }});
          }} catch (error) {{
            return JSON.stringify({{ ok: false, message: String(error) }});
          }}
        }})()"#,
        text = text
    );

    run_ui_action(client, &script).await
}

pub async fn execute_thread(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let texts = required_string_array(params, "texts")?;
    client.open("https://x.com/compose/tweet").await?;
    client.wait_ms(3_000).await?;

    let script = format!(
        r#"(async () => {{
          try {{
            const texts = {texts};
            if (!Array.isArray(texts) || texts.length === 0) {{
              return JSON.stringify({{ ok: false, message: 'Thread requires at least one text item.' }});
            }}

            const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));
            const pasteInto = async (box, text) => {{
              box.focus();
              const dataTransfer = new DataTransfer();
              dataTransfer.setData('text/plain', text);
              box.dispatchEvent(new ClipboardEvent('paste', {{
                clipboardData: dataTransfer,
                bubbles: true,
                cancelable: true
              }}));
              await sleep(700);
            }};

            for (let index = 0; index < texts.length; index++) {{
              let attempts = 0;
              let box = null;
              while (attempts < 20) {{
                const boxes = Array.from(document.querySelectorAll('[data-testid="tweetTextarea_0"]'));
                box = boxes[boxes.length - 1];
                if (box) break;
                await sleep(500);
                attempts++;
              }}
              if (!box) {{
                return JSON.stringify({{ ok: false, message: `Could not find composer for thread item ${{index + 1}}.` }});
              }}

              await pasteInto(box, texts[index]);

              if (index < texts.length - 1) {{
                const addButton = document.querySelector('[data-testid="addButton"]');
                if (!addButton) {{
                  return JSON.stringify({{ ok: false, message: 'Could not find Add to thread button.' }});
                }}
                addButton.click();
                await sleep(1200);
              }}
            }}

            const primaryBtn = document.querySelector('[data-testid="tweetButton"]');
            const inlineBtn = document.querySelector('[data-testid="tweetButtonInline"]');
            const btn = (primaryBtn && !primaryBtn.disabled) ? primaryBtn : ((inlineBtn && !inlineBtn.disabled) ? inlineBtn : null);
            if (!btn) {{
              return JSON.stringify({{ ok: false, message: 'Tweet button is disabled or not found for thread submission.' }});
            }}

            btn.click();
            await sleep(2500);

            const composerStillVisible = document.querySelector('[data-testid="tweetTextarea_0"]');
            if (!composerStillVisible) {{
              return JSON.stringify({{ ok: true, message: `Thread with ${{texts.length}} posts submitted successfully.` }});
            }}

            return JSON.stringify({{ ok: true, message: `Thread submission triggered for ${{texts.length}} posts; verify on timeline if needed.` }});
          }} catch (error) {{
            return JSON.stringify({{ ok: false, message: String(error) }});
          }}
        }})()"#,
        texts = serde_json::to_string(&texts).map_err(|err| AppError::Internal(err.to_string()))?
    );

    run_ui_action(client, &script).await
}

pub async fn execute_delete(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let url = required_string(params, "url")?;
    client.open(&url).await?;
    client.wait_ms(5_000).await?;

    run_ui_action(
        client,
        r#"(async () => {
          try {
            const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));
            let menuButton = null;
            let attempts = 0;
            while (attempts < 20) {
              menuButton = document.querySelector('[data-testid="caret"]') ||
                document.querySelector('[data-testid="Dropdown"]') ||
                Array.from(document.querySelectorAll('button')).find(button =>
                  /more|menu/i.test(button.getAttribute('aria-label') || '')
                );
              if (menuButton) break;
              await sleep(500);
              attempts++;
            }
            if (!menuButton) {
              return JSON.stringify({ ok: false, message: 'Could not find the tweet actions menu.' });
            }

            menuButton.click();
            await sleep(1000);

            const deleteItem = Array.from(document.querySelectorAll('[role="menuitem"], div, span'))
              .find(node => /delete/i.test((node.textContent || '').trim()));
            if (!deleteItem) {
              return JSON.stringify({ ok: false, message: 'Could not find Delete action in the tweet menu.' });
            }

            deleteItem.click();
            await sleep(1000);

            const confirmBtn = document.querySelector('[data-testid="confirmationSheetConfirm"]') ||
              Array.from(document.querySelectorAll('button')).find(button =>
                /delete/i.test((button.textContent || '').trim())
              );
            if (!confirmBtn) {
              return JSON.stringify({ ok: false, message: 'Could not find delete confirmation button.' });
            }

            confirmBtn.click();
            await sleep(2500);

            const stillVisibleConfirm = document.querySelector('[data-testid="confirmationSheetConfirm"]');
            if (!stillVisibleConfirm) {
              return JSON.stringify({ ok: true, message: 'Tweet delete action submitted successfully.' });
            }

            return JSON.stringify({ ok: false, message: 'Delete confirmation did not clear as expected.' });
          } catch (error) {
            return JSON.stringify({ ok: false, message: String(error) });
          }
        })()"#,
    )
    .await
}

async fn run_ui_action(client: &AgentBrowserClient, script: &str) -> AppResult<Value> {
    let payload: UiActionPayload = client.eval_json(script).await?;
    if payload.ok {
        client.wait_ms(2_000).await?;
        Ok(json!([{
            "status": "success",
            "message": payload.message
        }]))
    } else {
        Err(AppError::TwitterRequestFailed(payload.message))
    }
}

fn required_string(params: &Value, key: &str) -> AppResult<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| AppError::InvalidParams(format!("`{key}` is required")))
}

fn required_string_array(params: &Value, key: &str) -> AppResult<Vec<String>> {
    let values = params
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::InvalidParams(format!("`{key}` is required")))?;

    let cleaned = values
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    if cleaned.is_empty() {
        return Err(AppError::InvalidParams(format!(
            "`{key}` must contain at least one non-empty string"
        )));
    }

    Ok(cleaned)
}
