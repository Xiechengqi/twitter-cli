use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::twitter::extract::{normalize_username, required_string};

#[derive(Debug, Serialize, Deserialize)]
struct UiActionPayload {
    ok: bool,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConvListItem {
    user: String,
    #[serde(rename = "convId", default)]
    conv_id: String,
    #[serde(default)]
    href: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConvListResult {
    ok: bool,
    conversations: Vec<ConvListItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SendDmResult {
    status: String,
    user: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AcceptConvItem {
    idx: usize,
    text: String,
    #[serde(default)]
    href: String,
    user: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AcceptConvList {
    ok: bool,
    count: u64,
    items: Vec<AcceptConvItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AcceptConvResult {
    status: String,
    user: String,
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
            const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));
            // Wait for the primary tweet article to load
            let article = null;
            let attempts = 0;
            while (attempts < 20) {
              article = document.querySelector('article[data-testid="tweet"]');
              if (article) break;
              await sleep(500);
              attempts++;
            }
            if (!article) {
              return JSON.stringify({ ok: false, message: 'Could not find the tweet on the page. Are you logged in?' });
            }

            const unlikeBtn = article.querySelector('[data-testid="unlike"]');
            if (unlikeBtn) {
              return JSON.stringify({ ok: true, message: 'Tweet is already liked.' });
            }
            const likeBtn = article.querySelector('[data-testid="like"]');
            if (!likeBtn) {
              return JSON.stringify({ ok: false, message: 'Could not find the Like button on this tweet.' });
            }
            likeBtn.click();
            await sleep(1000);
            const verifyBtn = article.querySelector('[data-testid="unlike"]');
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
            const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));
            // Wait for the primary tweet article to load
            let article = null;
            let attempts = 0;
            while (attempts < 20) {
              article = document.querySelector('article[data-testid="tweet"]');
              if (article) break;
              await sleep(500);
              attempts++;
            }
            if (!article) {
              return JSON.stringify({ ok: false, message: 'Could not find the tweet on the page. Are you logged in?' });
            }

            const likeBtn = article.querySelector('[data-testid="like"]');
            if (likeBtn) {
              return JSON.stringify({ ok: true, message: 'Tweet is not liked (already unliked).' });
            }
            const unlikeBtn = article.querySelector('[data-testid="unlike"]');
            if (!unlikeBtn) {
              return JSON.stringify({ ok: false, message: 'Could not find the Unlike button on this tweet.' });
            }
            unlikeBtn.click();
            await sleep(1000);
            const verifyBtn = article.querySelector('[data-testid="like"]');
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
            const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));
            let article = null;
            let attempts = 0;
            while (attempts < 20) {
              article = document.querySelector('article[data-testid="tweet"]');
              if (article) break;
              await sleep(500);
              attempts++;
            }
            if (!article) {
              return JSON.stringify({ ok: false, message: 'Could not find the tweet on the page. Are you logged in?' });
            }

            const removeBtn = article.querySelector('[data-testid="removeBookmark"]');
            if (removeBtn) {
              return JSON.stringify({ ok: true, message: 'Tweet is already bookmarked.' });
            }
            const bookmarkBtn = article.querySelector('[data-testid="bookmark"]');
            if (!bookmarkBtn) {
              return JSON.stringify({ ok: false, message: 'Could not find Bookmark button.' });
            }
            bookmarkBtn.click();
            await sleep(1000);
            const verify = article.querySelector('[data-testid="removeBookmark"]');
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
            const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));
            let article = null;
            let attempts = 0;
            while (attempts < 20) {
              article = document.querySelector('article[data-testid="tweet"]');
              if (article) break;
              await sleep(500);
              attempts++;
            }
            if (!article) {
              return JSON.stringify({ ok: false, message: 'Could not find the tweet on the page. Are you logged in?' });
            }

            const bookmarkBtn = article.querySelector('[data-testid="bookmark"]');
            if (bookmarkBtn) {
              return JSON.stringify({ ok: true, message: 'Tweet is not bookmarked (already removed).' });
            }
            const removeBtn = article.querySelector('[data-testid="removeBookmark"]');
            if (!removeBtn) {
              return JSON.stringify({ ok: false, message: 'Could not find Remove Bookmark button.' });
            }
            removeBtn.click();
            await sleep(1000);
            const verify = article.querySelector('[data-testid="bookmark"]');
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

pub async fn execute_retweet(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let url = required_string(params, "url")?;
    client.open(&url).await?;
    client.wait_ms(5_000).await?;

    run_ui_action(
        client,
        r#"(async () => {
          try {
            const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));
            let article = null;
            let attempts = 0;
            while (attempts < 20) {
              article = document.querySelector('article[data-testid="tweet"]');
              if (article) break;
              await sleep(500);
              attempts++;
            }
            if (!article) {
              return JSON.stringify({ ok: false, message: 'Could not find the tweet on the page. Are you logged in?' });
            }

            const unretweetBtn = article.querySelector('[data-testid="unretweet"]');
            if (unretweetBtn) {
              return JSON.stringify({ ok: true, message: 'Tweet is already retweeted.' });
            }
            const retweetBtn = article.querySelector('[data-testid="retweet"]');
            if (!retweetBtn) {
              return JSON.stringify({ ok: false, message: 'Could not find the Retweet button on this tweet.' });
            }

            retweetBtn.click();
            await sleep(1000);

            let confirmBtn = document.querySelector('[data-testid="retweetConfirm"]');
            if (!confirmBtn) {
              const candidates = Array.from(document.querySelectorAll('[role="menuitem"], button'));
              confirmBtn = candidates.find(node => /repost|retweet/i.test((node.textContent || '').trim())) || null;
            }
            if (!confirmBtn) {
              return JSON.stringify({ ok: false, message: 'Could not find the Retweet confirmation option.' });
            }

            confirmBtn.click();
            await sleep(1500);

            const verifyBtn = article.querySelector('[data-testid="unretweet"]');
            if (verifyBtn) {
              return JSON.stringify({ ok: true, message: 'Tweet successfully retweeted.' });
            }
            return JSON.stringify({ ok: false, message: 'Retweet action was initiated but UI did not update as expected.' });
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

pub async fn execute_block(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
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
            const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

            // Check if already blocked (unblock button visible)
            let attempts = 0;
            while (attempts < 20) {{
              const unblockBtn = document.querySelector('[data-testid$="-unblock"]');
              if (unblockBtn) {{
                return JSON.stringify({{ ok: true, message: '@{username} is already blocked.' }});
              }}
              const userActions = document.querySelector('[data-testid="userActions"]');
              if (userActions) break;
              const followBtn = document.querySelector('[data-testid$="-follow"]');
              if (followBtn) break;
              await sleep(500);
              attempts++;
            }}

            // Click the more actions button
            const userActions = document.querySelector('[data-testid="userActions"]');
            if (!userActions) {{
              return JSON.stringify({{ ok: false, message: 'Could not find user actions menu. Are you logged in?' }});
            }}
            userActions.click();
            await sleep(1000);

            // Find and click Block menu item
            const blockItem = Array.from(document.querySelectorAll('[role="menuitem"]'))
              .find(node => {{
                const text = (node.textContent || '').trim().toLowerCase();
                return text.includes('block') && !text.includes('unblock');
              }});
            if (!blockItem) {{
              return JSON.stringify({{ ok: false, message: 'Could not find Block option in the menu.' }});
            }}
            blockItem.click();
            await sleep(1000);

            // Confirm the block dialog
            const confirmBtn = document.querySelector('[data-testid="confirmationSheetConfirm"]') ||
              Array.from(document.querySelectorAll('button')).find(btn =>
                /block/i.test((btn.textContent || '').trim())
              );
            if (confirmBtn) {{
              confirmBtn.click();
              await sleep(1500);
            }}

            // Verify block succeeded
            const verify = document.querySelector('[data-testid$="-unblock"]');
            if (verify) {{
              return JSON.stringify({{ ok: true, message: 'Successfully blocked @{username}.' }});
            }}
            return JSON.stringify({{ ok: true, message: 'Block action submitted for @{username}.' }});
          }} catch (error) {{
            return JSON.stringify({{ ok: false, message: String(error) }});
          }}
        }})()"#,
        username = username
    );

    run_ui_action(client, &script).await
}

pub async fn execute_unblock(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
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
            const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

            let attempts = 0;
            let unblockBtn = null;
            while (attempts < 20) {{
              const followBtn = document.querySelector('[data-testid$="-follow"]');
              if (followBtn) {{
                return JSON.stringify({{ ok: true, message: '@{username} is not blocked (already unblocked).' }});
              }}
              unblockBtn = document.querySelector('[data-testid$="-unblock"]');
              if (unblockBtn) break;
              await sleep(500);
              attempts++;
            }}
            if (!unblockBtn) {{
              return JSON.stringify({{ ok: false, message: 'Could not find Unblock button. Are you logged in?' }});
            }}
            unblockBtn.click();
            await sleep(1000);

            // Confirm the unblock dialog
            const confirmBtn = document.querySelector('[data-testid="confirmationSheetConfirm"]') ||
              Array.from(document.querySelectorAll('button')).find(btn =>
                /unblock/i.test((btn.textContent || '').trim())
              );
            if (confirmBtn) {{
              confirmBtn.click();
              await sleep(1500);
            }}

            // Verify unblock succeeded
            const verify = document.querySelector('[data-testid$="-follow"]');
            if (verify) {{
              return JSON.stringify({{ ok: true, message: 'Successfully unblocked @{username}.' }});
            }}
            return JSON.stringify({{ ok: true, message: 'Unblock action submitted for @{username}.' }});
          }} catch (error) {{
            return JSON.stringify({{ ok: false, message: String(error) }});
          }}
        }})()"#,
        username = username
    );

    run_ui_action(client, &script).await
}

pub async fn execute_hide_reply(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
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

            const hideItem = Array.from(document.querySelectorAll('[role="menuitem"]'))
              .find(node => /hide reply/i.test((node.textContent || '').trim()));
            if (!hideItem) {
              return JSON.stringify({ ok: false, message: 'Could not find "Hide reply" option in the menu. You may not have permission to hide this reply.' });
            }

            hideItem.click();
            await sleep(1500);

            return JSON.stringify({ ok: true, message: 'Reply hidden successfully.' });
          } catch (error) {
            return JSON.stringify({ ok: false, message: String(error) });
          }
        })()"#,
    )
    .await
}

pub async fn execute_reply_dm(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let text = required_string(params, "text")?;
    let max = params.get("max").and_then(Value::as_u64).unwrap_or(20) as usize;
    let skip_replied = params
        .get("skip_replied")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    client.open("https://x.com/messages").await?;
    client.wait_ms(5_000).await?;

    let needed = max + 10;
    let collect_script = format!(
        r#"(async () => {{
          try {{
            let attempts = 0;
            while (attempts < 10) {{
              const items = document.querySelectorAll('[data-testid^="dm-conversation-item-"], [data-testid="conversation"]');
              if (items.length > 0) break;
              await new Promise(r => setTimeout(r, 1000));
              attempts++;
            }}

            const needed = {needed};
            const seenIds = new Set();
            let noNewCount = 0;

            for (let scroll = 0; scroll < 30; scroll++) {{
              const items = Array.from(document.querySelectorAll('[data-testid^="dm-conversation-item-"], [data-testid="conversation"]'));
              const prevSize = seenIds.size;
              items.forEach(el => seenIds.add(el.getAttribute('data-testid')));

              if (seenIds.size >= needed) break;

              const scrollContainer = document.querySelector('[data-testid="dm-inbox-panel"]') ||
                                      (items.length > 0 ? items[items.length - 1].closest('[class*="scroll"]') : null) ||
                                      (items.length > 0 ? items[items.length - 1].parentElement : null);
              if (scrollContainer) scrollContainer.scrollTop = scrollContainer.scrollHeight;
              if (items.length > 0) items[items.length - 1].scrollIntoView({{ behavior: 'instant', block: 'end' }});

              await new Promise(r => setTimeout(r, 1500));

              const newItems = Array.from(document.querySelectorAll('[data-testid^="dm-conversation-item-"], [data-testid="conversation"]'));
              const newSize = new Set(newItems.map(el => el.getAttribute('data-testid'))).size;
              if (newSize <= seenIds.size) {{
                noNewCount++;
                if (noNewCount >= 3) break;
              }} else {{
                noNewCount = 0;
              }}
            }}

            const finalItems = Array.from(document.querySelectorAll('[data-testid^="dm-conversation-item-"], [data-testid="conversation"]'));
            const conversations = finalItems.map((item) => {{
              const testId = item.getAttribute('data-testid') || '';
              const text = item.innerText || '';
              const lines = text.split('\\n').filter(l => l.trim());
              const user = lines[0] || 'Unknown';
              const match = testId.match(/dm-conversation-item-(.+)/);
              const convId = match ? match[1].replace(':', '-') : '';
              const link = item.querySelector('a[href*="/messages/"]');
              const href = link ? link.href : '';
              return {{ user, convId, href }};
            }});

            return {{ ok: true, conversations }};
          }} catch(e) {{
            return {{ ok: false, conversations: [] }};
          }}
        }})()"#,
        needed = needed
    );

    let collected: ConvListResult = client.eval_json(&collect_script).await?;
    if !collected.ok || collected.conversations.is_empty() {
        return Ok(
            json!([{"index": 0, "status": "info", "user": "System", "message": "No conversations found"}]),
        );
    }

    let mut results: Vec<Value> = Vec::new();
    let mut sent_count = 0usize;

    for conv in collected.conversations.iter().take(max + 10) {
        if sent_count >= max {
            break;
        }

        let conv_url = if !conv.conv_id.is_empty() {
            format!("https://x.com/messages/{}", conv.conv_id)
        } else if !conv.href.is_empty() {
            conv.href.clone()
        } else {
            continue;
        };

        client.open(&conv_url).await?;
        client.wait_ms(3_000).await?;

        let conv_user = conv.user.clone();
        let send_script = format!(
            r#"(async () => {{
              try {{
                const messageText = {text:?};
                const skipReplied = {skip_replied};

                const dmHeader = document.querySelector('[data-testid="DmActivityContainer"] [dir="ltr"] span') ||
                                 document.querySelector('[data-testid="conversation-header"]') ||
                                 document.querySelector('[data-testid="DmActivityContainer"] h2');
                const username = dmHeader ? dmHeader.innerText.trim().split('\\n')[0] : {conv_user:?};

                if (skipReplied) {{
                  const chatArea = document.querySelector('[data-testid="DmScrollerContainer"]') ||
                                   document.querySelector('main');
                  const chatText = chatArea ? chatArea.innerText : '';
                  if (chatText.includes(messageText)) {{
                    return {{ status: 'skipped', user: username, message: 'Already sent this message' }};
                  }}
                }}

                const input = document.querySelector('[data-testid="dmComposerTextInput"]');
                if (!input) {{
                  return {{ status: 'error', user: username, message: 'No message input found' }};
                }}

                input.focus();
                await new Promise(r => setTimeout(r, 300));
                document.execCommand('insertText', false, messageText);
                await new Promise(r => setTimeout(r, 500));

                const sendBtn = document.querySelector('[data-testid="dmComposerSendButton"]');
                if (!sendBtn) {{
                  return {{ status: 'error', user: username, message: 'No send button found' }};
                }}

                sendBtn.click();
                await new Promise(r => setTimeout(r, 1500));

                return {{ status: 'sent', user: username, message: 'Message sent' }};
              }} catch(e) {{
                return {{ status: 'error', user: 'system', message: String(e) }};
              }}
            }})()"#,
            text = text,
            skip_replied = skip_replied,
            conv_user = conv_user,
        );

        let send_result: SendDmResult = client.eval_json(&send_script).await?;
        if send_result.status == "sent" {
            sent_count += 1;
        }
        results.push(json!({
            "index": results.len() + 1,
            "status": send_result.status,
            "user": send_result.user,
            "message": send_result.message,
        }));

        client.wait_ms(1_000).await?;
    }

    if results.is_empty() {
        results.push(
            json!({"index": 0, "status": "info", "user": "System", "message": "No conversations processed"}),
        );
    }

    Ok(json!(results))
}

pub async fn execute_accept_dm(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let query = required_string(params, "query")?;
    let keywords: Vec<String> = query
        .split(',')
        .map(|k| k.trim().to_lowercase())
        .filter(|k| !k.is_empty())
        .collect();
    let max = params.get("max").and_then(Value::as_u64).unwrap_or(20) as usize;

    let mut results: Vec<Value> = Vec::new();
    let mut accept_count = 0usize;
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

    let max_rounds = max + 50;
    for _ in 0..max_rounds {
        if accept_count >= max {
            break;
        }

        client.open("https://x.com/messages/requests").await?;
        client.wait_ms(4_000).await?;

        let needed = max + 10;
        let collect_script = format!(
            r#"(async () => {{
              try {{
                let attempts = 0;
                while (attempts < 10) {{
                  const convs = document.querySelectorAll('[data-testid="conversation"]');
                  if (convs.length > 0) break;
                  await new Promise(r => setTimeout(r, 1000));
                  attempts++;
                }}

                const seenCount = new Set();
                let noNewCount = 0;
                for (let scroll = 0; scroll < 20; scroll++) {{
                  const convs = Array.from(document.querySelectorAll('[data-testid="conversation"]'));
                  const prevSize = seenCount.size;
                  convs.forEach((_, i) => seenCount.add(i));
                  if (convs.length >= {needed}) break;

                  if (convs.length > 0) convs[convs.length - 1].scrollIntoView({{ behavior: 'instant', block: 'end' }});
                  await new Promise(r => setTimeout(r, 1500));

                  if (seenCount.size <= prevSize) {{
                    noNewCount++;
                    if (noNewCount >= 3) break;
                  }} else {{
                    noNewCount = 0;
                  }}
                }}

                const convs = Array.from(document.querySelectorAll('[data-testid="conversation"]'));
                if (convs.length === 0) return {{ ok: false, count: 0, items: [] }};

                const items = convs.map((conv, idx) => {{
                  const text = conv.innerText || '';
                  const link = conv.querySelector('a[href]');
                  const href = link ? link.href : '';
                  const lines = text.split('\\n').filter(l => l.trim());
                  const user = lines[0] || 'Unknown';
                  return {{ idx, text, href, user }};
                }});
                return {{ ok: true, count: convs.length, items }};
              }} catch(e) {{
                return {{ ok: false, count: 0, items: [] }};
              }}
            }})()"#,
            needed = needed
        );

        let conv_list: AcceptConvList = client.eval_json(&collect_script).await?;
        if !conv_list.ok || conv_list.count == 0 {
            if results.is_empty() {
                results.push(
                    json!({"index": 1, "status": "info", "user": "System", "message": "No message requests found"}),
                );
            }
            break;
        }

        let mut found_in_round = false;
        for item in &conv_list.items {
            if accept_count >= max {
                break;
            }
            let conv_key = if !item.href.is_empty() {
                item.href.clone()
            } else {
                format!("conv-{}", item.idx)
            };
            if visited.contains(&conv_key) {
                continue;
            }
            visited.insert(conv_key);

            let text_lower = item.text.to_lowercase();
            if !keywords.iter().any(|k| text_lower.contains(k.as_str())) {
                continue;
            }

            if item.href.is_empty() {
                continue;
            }
            client.open(&item.href).await?;
            client.wait_ms(3_000).await?;

            let keywords_json = serde_json::to_string(&keywords)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let accept_script = format!(
                r#"(async () => {{
                  try {{
                    const keywords = {keywords_json};

                    const heading = document.querySelector('[data-testid="conversation-header"]') ||
                                    document.querySelector('[data-testid="DM-conversation-header"]');
                    let username = 'Unknown';
                    if (heading) username = heading.innerText.trim().split('\\n')[0];

                    const chatArea = document.querySelector('[data-testid="DmScrollerContainer"]') ||
                                     document.querySelector('[data-testid="DMConversationBody"]') ||
                                     document.querySelector('main');
                    const text = chatArea ? chatArea.innerText : '';

                    const matchedKw = keywords.filter(k => text.toLowerCase().includes(k));
                    if (matchedKw.length === 0) {{
                      return {{ status: 'skipped', user: username, message: 'No keyword match in full content' }};
                    }}

                    const allBtns = Array.from(document.querySelectorAll('[role="button"]'));
                    const acceptBtn = allBtns.find(btn => {{
                      const t = btn.innerText.trim().toLowerCase();
                      return t === 'accept' || t === '接受';
                    }});

                    if (!acceptBtn) {{
                      return {{ status: 'no_button', user: username, message: 'Keyword matched but no Accept button (already accepted?)' }};
                    }}

                    acceptBtn.click();
                    await new Promise(r => setTimeout(r, 2000));

                    const btnsAfter = Array.from(document.querySelectorAll('[role="button"]'));
                    const confirmBtn = btnsAfter.find(btn => {{
                      const t = btn.innerText.trim().toLowerCase();
                      return (t === 'accept' || t === '接受') && btn !== acceptBtn;
                    }});
                    if (confirmBtn) {{
                      confirmBtn.click();
                      await new Promise(r => setTimeout(r, 1000));
                    }}

                    return {{ status: 'accepted', user: username, message: 'Accepted! Matched: ' + matchedKw.join(', ') }};
                  }} catch(e) {{
                    return {{ status: 'error', user: 'system', message: String(e) }};
                  }}
                }})()"#,
                keywords_json = keywords_json
            );

            let res: AcceptConvResult = client.eval_json(&accept_script).await?;
            if res.status == "accepted" {
                accept_count += 1;
                found_in_round = true;
                results.push(json!({
                    "index": accept_count,
                    "status": "accepted",
                    "user": res.user,
                    "message": res.message,
                }));
                client.wait_ms(2_000).await?;
                break;
            }
            // "no_button" = already accepted, "skipped" = no match in full content — continue to next
        }

        if !found_in_round {
            break;
        }
    }

    if results.is_empty() {
        results.push(json!({
            "index": 0,
            "status": "info",
            "user": "System",
            "message": format!("No requests matched keywords \"{}\"", keywords.join(", "))
        }));
    }

    Ok(json!(results))
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
