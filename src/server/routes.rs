use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::auth::{AUTH_COOKIE_NAME, is_authenticated};
use crate::config;
use crate::errors::AppError;
use crate::response::ApiResponse;
use crate::server::{AppState, ExecutionRecord};

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(console_home))
        .route("/login", get(login_page))
        .route("/logout", post(logout))
        .route("/setup/password", get(setup_password_page))
        .route("/commands", get(commands_page))
        .route("/mcp", get(mcp_page).post(call_mcp))
        .route("/settings", get(settings_page))
        .route("/docs", get(docs_page))
        .route("/health", get(health))
        .route("/api/bootstrap", get(bootstrap))
        .route("/api/setup/password", post(setup_password))
        .route("/api/login", post(login))
        .route("/api/logout", post(logout_api))
        .route("/api/config", get(get_config).post(update_config))
        .route("/api/commands", get(get_commands))
        .route("/api/history", get(get_history))
        .route("/api/mcp/tools", get(get_mcp_tools))
        .route("/api/skills", get(get_skills))
        .route("/api/password/change", post(change_password))
        .route("/api/execute/{command}", post(execute_command))
        .with_state(state)
}

async fn console_home(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let runtime = state.runtime.read().await;
    if let Some(redirect) = maybe_redirect_console(&headers, &runtime.auth_state) {
        return Ok(redirect.into_response());
    }
    let runtime = state.runtime.read().await;
    let recent_executions = if runtime.recent_executions.is_empty() {
        "<p>No commands have been executed yet.</p>".to_string()
    } else {
        let items = runtime
            .recent_executions
            .iter()
            .rev()
            .take(6)
            .map(|entry| {
                format!(
                    "<tr><td><code>{}</code></td><td>{}</td><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
                    entry.timestamp,
                    escape_html(&entry.source),
                    escape_html(&entry.command),
                    if entry.ok { "ok" } else { "error" },
                    escape_html(&entry.summary)
                )
            })
            .collect::<Vec<_>>()
            .join("");
        format!(
            "<table><thead><tr><th>When</th><th>Source</th><th>Command</th><th>Status</th><th>Summary</th></tr></thead><tbody>{items}</tbody></table>"
        )
    };
    let vnc_section = if !runtime.config.vnc.url.is_empty() && runtime.config.vnc.embed {
        format!(
            "<div class=\"card\"><h2>VNC</h2><p>Embedded preview from {}</p><iframe src=\"{}\" style=\"width:100%;height:320px;border:1px solid #d1d5db;border-radius:12px;background:#fff\"></iframe></div>",
            escape_html(&runtime.config.vnc.url),
            escape_html(&runtime.config.vnc.url)
        )
    } else {
        "<div class=\"card\"><h2>VNC</h2><p>VNC is not configured or embedding is disabled.</p></div>"
            .to_string()
    };
    Ok(Html(render_page(
        "Console",
        &format!(
            r#"
            <div class="hero">
              <h1>twitter-cli</h1>
              <p>Local Twitter automation control plane backed by <code>agent-browser</code>.</p>
            </div>
            <div class="grid">
              <div class="card">
                <h2>Service Status</h2>
                <dl>
                  <dt>API</dt><dd><code>{}</code></dd>
                  <dt>Docs</dt><dd><code>{}/docs</code></dd>
                  <dt>Config</dt><dd><code>{}</code></dd>
                </dl>
              </div>
              <div class="card">
                <h2>Agent Browser</h2>
                <dl>
                  <dt>Binary</dt><dd><code>{}</code></dd>
                  <dt>CDP URL</dt><dd><code>{}</code></dd>
                  <dt>Session</dt><dd><code>{}</code></dd>
                </dl>
              </div>
              <div class="card">
                <h2>Quick Actions</h2>
                <ul>
                  <li><a href="/commands">Run profile, timeline, search, and write commands</a></li>
                  <li><a href="/mcp">Review MCP tools and auth model</a></li>
                  <li><a href="/settings">Adjust server, agent-browser, and auth settings</a></li>
                </ul>
              </div>
              <div class="card">
                <h2>Recent Executions</h2>
                {}
              </div>
              {}
            </div>
            "#,
            escape_html(&runtime.config.base_url()),
            escape_html(&runtime.config.base_url()),
            escape_html(&state.config_path),
            escape_html(&runtime.config.agent_browser.binary),
            escape_html(&runtime.config.agent_browser.cdp_url),
            escape_html(&runtime.config.agent_browser.session_name),
            recent_executions,
            vnc_section,
        ),
        NavKind::Authenticated,
    ))
    .into_response())
}

async fn login_page() -> Html<String> {
    Html(render_page(
        "Login",
        r#"
        <div class="single-card">
          <h1>Login</h1>
          <p>Use the Console password. The same credential also works as API and MCP Bearer token.</p>
          <form id="login-form">
            <label>Password</label>
            <input id="password" name="password" type="password" autocomplete="current-password" />
            <button type="submit">Login</button>
          </form>
          <pre id="result"></pre>
        </div>
        <script>
        document.getElementById('login-form').addEventListener('submit', async (event) => {
          event.preventDefault();
          const password = document.getElementById('password').value;
          const response = await fetch('/api/login', {
            method: 'POST',
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify({ password })
          });
          const data = await response.json();
          document.getElementById('result').textContent = JSON.stringify(data, null, 2);
          if (response.ok) window.location.href = '/';
        });
        </script>
        "#,
        NavKind::Anonymous,
    ))
}

async fn setup_password_page() -> Html<String> {
    Html(render_page(
        "Setup Password",
        r#"
        <div class="single-card">
          <h1>Setup Password</h1>
          <p>First run requires a password. This password will also act as the API and MCP Bearer token.</p>
          <form id="setup-form">
            <label>Password</label>
            <input id="password" name="password" type="password" autocomplete="new-password" />
            <button type="submit">Save Password</button>
          </form>
          <pre id="result"></pre>
        </div>
        <script>
        document.getElementById('setup-form').addEventListener('submit', async (event) => {
          event.preventDefault();
          const password = document.getElementById('password').value;
          const response = await fetch('/api/setup/password', {
            method: 'POST',
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify({ password })
          });
          const data = await response.json();
          document.getElementById('result').textContent = JSON.stringify(data, null, 2);
          if (response.ok) window.location.href = '/settings';
        });
        </script>
        "#,
        NavKind::Anonymous,
    ))
}

async fn commands_page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let runtime = state.runtime.read().await;
    if let Some(redirect) = maybe_redirect_console(&headers, &runtime.auth_state) {
        return Ok(redirect.into_response());
    }
    let command_cards = state
        .manifest
        .commands
        .iter()
        .map(|command| {
            format!(
                r#"<details class="command-card">
                  <summary><strong>{}</strong> <span>{}</span></summary>
                  <p>{}</p>
                  <code>{}</code>
                </details>"#,
                command.name,
                command.execution_mode,
                command.summary,
                build_example_payload(command),
            )
        })
        .collect::<Vec<_>>()
        .join("");
    Ok(Html(render_page(
        "Commands",
        &format!(
            r#"
            <div class="grid two">
              <div class="card">
                <h1>Command Runner</h1>
                <p>Run any registered command through the same API used by CLI and MCP mappings.</p>
                <label>Command</label>
                <input id="command" value="profile" />
                <label>Params (JSON)</label>
                <textarea id="params">{{"username":"OpenAI"}}</textarea>
                <button id="run">Execute</button>
                <pre id="result"></pre>
              </div>
              <div class="card">
                <h2>Registered Commands</h2>
                {}
              </div>
            </div>
            <script>
            document.getElementById('run').addEventListener('click', async () => {{
              const command = document.getElementById('command').value.trim();
              const raw = document.getElementById('params').value.trim() || '{{}}';
              let params;
              try {{
                params = JSON.parse(raw);
              }} catch (error) {{
                document.getElementById('result').textContent = 'Invalid JSON: ' + error;
                return;
              }}
              const response = await fetch('/api/execute/' + encodeURIComponent(command), {{
                method: 'POST',
                headers: {{ 'content-type': 'application/json' }},
                body: JSON.stringify({{ params, format: 'json' }})
              }});
              const data = await response.json();
              document.getElementById('result').textContent = JSON.stringify(data, null, 2);
            }});
            </script>
            "#,
            command_cards
        ),
        NavKind::Authenticated,
    ))
    .into_response())
}

async fn mcp_page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let runtime = state.runtime.read().await;
    if let Some(redirect) = maybe_redirect_console(&headers, &runtime.auth_state) {
        return Ok(redirect.into_response());
    }
    let tools = state
        .manifest
        .mcp_tools
        .iter()
        .map(|tool| {
            format!(
                "<li><strong>{}</strong> → <code>{}</code> ({})</li>",
                tool.name,
                tool.command,
                if tool.read_only { "read" } else { "write" }
            )
        })
        .collect::<Vec<_>>()
        .join("");
    Ok(Html(render_page(
        "MCP",
        &format!(
            r#"
            <div class="grid two">
              <div class="card">
                <h1>MCP</h1>
                <p>All MCP tools use the same password as Console and API.</p>
                <pre>Authorization: Bearer &lt;console-password&gt;</pre>
                <p>Endpoint: <code>/mcp</code></p>
                <p>Tool index: <code>/api/mcp/tools</code></p>
                <label>Tool</label>
                <input id="mcp-tool" value="twitter_profile" />
                <label>Arguments (JSON)</label>
                <textarea id="mcp-arguments">{{"username":"OpenAI"}}</textarea>
                <button id="call-mcp">Call Tool</button>
                <pre id="mcp-result"></pre>
              </div>
              <div class="card">
                <h2>Tools</h2>
                <ul>{}</ul>
              </div>
            </div>
            <script>
            document.getElementById('call-mcp').addEventListener('click', async () => {{
              let argumentsPayload;
              try {{
                argumentsPayload = JSON.parse(document.getElementById('mcp-arguments').value.trim() || '{{}}');
              }} catch (error) {{
                document.getElementById('mcp-result').textContent = 'Invalid JSON: ' + error;
                return;
              }}
              const response = await fetch('/mcp', {{
                method: 'POST',
                headers: {{ 'content-type': 'application/json' }},
                body: JSON.stringify({{
                  jsonrpc: '2.0',
                  id: 'console',
                  method: 'tools/call',
                  params: {{
                    name: document.getElementById('mcp-tool').value.trim(),
                    arguments: argumentsPayload
                  }}
                }})
              }});
              const data = await response.json();
              document.getElementById('mcp-result').textContent = JSON.stringify(data, null, 2);
            }});
            </script>
            "#,
            tools
        ),
        NavKind::Authenticated,
    ))
    .into_response())
}

async fn settings_page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let runtime = state.runtime.read().await;
    if let Some(redirect) = maybe_redirect_console(&headers, &runtime.auth_state) {
        return Ok(redirect.into_response());
    }
    let config_json = serde_json::to_string_pretty(&runtime.config)
        .map_err(|err| AppError::Internal(err.to_string()))?;
    Ok(Html(render_page(
        "Settings",
        &format!(
            r#"
            <div class="grid two">
              <div class="card">
                <h1>Config</h1>
                <p>Edit and save the current configuration.</p>
                <textarea id="config-json">{}</textarea>
                <button id="save-config">Save Config</button>
                <pre id="config-result"></pre>
              </div>
              <div class="card">
                <h2>Change Password</h2>
                <label>Old Password</label>
                <input id="old-password" type="password" autocomplete="current-password" />
                <label>New Password</label>
                <input id="new-password" type="password" autocomplete="new-password" />
                <button id="change-password">Change Password</button>
                <pre id="password-result"></pre>
              </div>
            </div>
            <script>
            document.getElementById('save-config').addEventListener('click', async () => {{
              let payload;
              try {{
                payload = JSON.parse(document.getElementById('config-json').value);
              }} catch (error) {{
                document.getElementById('config-result').textContent = 'Invalid JSON: ' + error;
                return;
              }}
              const response = await fetch('/api/config', {{
                method: 'POST',
                headers: {{ 'content-type': 'application/json' }},
                body: JSON.stringify(payload)
              }});
              const data = await response.json();
              document.getElementById('config-result').textContent = JSON.stringify(data, null, 2);
            }});
            document.getElementById('change-password').addEventListener('click', async () => {{
              const response = await fetch('/api/password/change', {{
                method: 'POST',
                headers: {{ 'content-type': 'application/json' }},
                body: JSON.stringify({{
                  old_password: document.getElementById('old-password').value,
                  new_password: document.getElementById('new-password').value
                }})
              }});
              const data = await response.json();
              document.getElementById('password-result').textContent = JSON.stringify(data, null, 2);
            }});
            </script>
            "#,
            escape_html(&config_json)
        ),
        NavKind::Authenticated,
    ))
    .into_response())
}

async fn docs_page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let runtime = state.runtime.read().await;
    if let Some(redirect) = maybe_redirect_console(&headers, &runtime.auth_state) {
        return Ok(redirect.into_response());
    }
    let commands = state
        .manifest
        .commands
        .iter()
        .map(|command| {
            format!(
                "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td></tr>",
                command.name, command.category, command.execution_mode, command.summary
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let skills = state
        .manifest
        .skills
        .iter()
        .map(|skill| {
            format!(
                "<li><strong>{}</strong>: {}</li>",
                skill.name, skill.summary
            )
        })
        .collect::<Vec<_>>()
        .join("");
    Ok(Html(render_page(
        "Docs",
        &format!(
            r#"
            <div class="card">
              <h1>Docs</h1>
              <p>Shared source of truth for commands, MCP tools, and skills.</p>
              <table>
                <thead><tr><th>Command</th><th>Category</th><th>Mode</th><th>Summary</th></tr></thead>
                <tbody>{}</tbody>
              </table>
            </div>
            <div class="card">
              <h2>Skills</h2>
              <ul>{}</ul>
            </div>
            "#,
            commands, skills
        ),
        NavKind::Authenticated,
    ))
    .into_response())
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}

async fn bootstrap(State(state): State<Arc<AppState>>) -> Json<Value> {
    let runtime = state.runtime.read().await;
    Json(json!({
        "first_run": state.first_run,
        "password_required": !runtime.config.is_password_initialized(),
        "server": {
            "host": runtime.config.server.host,
            "port": runtime.config.server.port,
        },
        "agent_browser": {
            "binary": runtime.config.agent_browser.binary,
            "detected": runtime.config.agent_browser.binary != "agent-browser",
            "cdp_url": runtime.config.agent_browser.cdp_url,
        },
        "vnc": {
            "configured": !runtime.config.vnc.url.is_empty(),
        }
    }))
}

#[derive(Deserialize)]
struct PasswordRequest {
    password: String,
}

async fn setup_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PasswordRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    if payload.password.is_empty() {
        return Err(AppError::InvalidParams("password is required".to_string()));
    }

    let mut runtime = state.runtime.write().await;
    let mut updated = runtime.config.clone();
    updated.auth.password = payload.password;
    updated.auth.password_changed = true;
    let path = config::config_path()?;
    config::save(&path, &updated).await?;
    runtime.auth_state = crate::auth::AuthState::from_config(&updated);
    runtime.config = updated;

    Ok(Json(ApiResponse::success(
        json!({ "configured": true }),
        None,
    )))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    let runtime = state.runtime.read().await;
    if payload.password != runtime.auth_state.password {
        return Err(AppError::InvalidPassword);
    }

    let cookie = format!(
        "{AUTH_COOKIE_NAME}={}; Path=/; HttpOnly; SameSite=Lax",
        payload.password
    );
    let mut response = Json(ApiResponse::success(json!({ "ok": true }), None)).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|err| AppError::Internal(err.to_string()))?,
    );
    Ok(response)
}

async fn logout() -> impl IntoResponse {
    let mut response = Redirect::to("/login").into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static("twitter_cli_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"),
    );
    response
}

async fn logout_api() -> impl IntoResponse {
    let mut response =
        Json(ApiResponse::success(json!({ "logged_out": true }), None)).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static("twitter_cli_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"),
    );
    response
}

async fn get_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    require_auth(&headers, &state).await?;
    let runtime = state.runtime.read().await;
    Ok(Json(ApiResponse::success(
        serde_json::to_value(&runtime.config).map_err(|err| AppError::Internal(err.to_string()))?,
        None,
    )))
}

async fn update_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<crate::config::AppConfig>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    require_auth(&headers, &state).await?;
    let path = config::config_path()?;
    config::save(&path, &payload).await?;

    let mut runtime = state.runtime.write().await;
    runtime.auth_state = crate::auth::AuthState::from_config(&payload);
    runtime.config = payload;

    Ok(Json(ApiResponse::success(json!({ "saved": true }), None)))
}

async fn get_commands(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    require_auth(&headers, &state).await?;
    Ok(Json(ApiResponse::success(
        serde_json::to_value(&state.manifest.commands)
            .map_err(|err| AppError::Internal(err.to_string()))?,
        None,
    )))
}

async fn get_history(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    require_auth(&headers, &state).await?;
    let runtime = state.runtime.read().await;
    Ok(Json(ApiResponse::success(
        serde_json::to_value(&runtime.recent_executions)
            .map_err(|err| AppError::Internal(err.to_string()))?,
        None,
    )))
}

async fn get_mcp_tools(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    require_auth(&headers, &state).await?;
    Ok(Json(ApiResponse::success(
        serde_json::to_value(&state.manifest.mcp_tools)
            .map_err(|err| AppError::Internal(err.to_string()))?,
        None,
    )))
}

async fn get_skills(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    require_auth(&headers, &state).await?;
    Ok(Json(ApiResponse::success(
        serde_json::to_value(&state.manifest.skills)
            .map_err(|err| AppError::Internal(err.to_string()))?,
        None,
    )))
}

#[derive(Deserialize)]
struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

async fn change_password(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    require_auth(&headers, &state).await?;
    if payload.new_password.is_empty() {
        return Err(AppError::InvalidParams(
            "new_password is required".to_string(),
        ));
    }

    let mut runtime = state.runtime.write().await;
    if payload.old_password != runtime.auth_state.password {
        return Err(AppError::InvalidPassword);
    }

    let mut updated = runtime.config.clone();
    updated.auth.password = payload.new_password;
    updated.auth.password_changed = true;
    let path = config::config_path()?;
    config::save(&path, &updated).await?;
    runtime.auth_state = crate::auth::AuthState::from_config(&updated);
    runtime.config = updated;

    Ok(Json(ApiResponse::success(
        json!({ "password_changed": true }),
        None,
    )))
}

#[derive(Deserialize)]
struct ExecuteRequest {
    #[serde(default)]
    params: Value,
    #[allow(dead_code)]
    format: Option<String>,
}

#[derive(Deserialize)]
struct McpToolCall {
    name: String,
    #[serde(default)]
    arguments: Value,
}

#[derive(Deserialize)]
struct McpRequest {
    #[allow(dead_code)]
    jsonrpc: Option<String>,
    #[allow(dead_code)]
    id: Option<Value>,
    method: Option<String>,
    #[serde(default)]
    params: Option<McpToolCall>,
    tool: Option<String>,
    #[serde(default)]
    arguments: Option<Value>,
}

async fn execute_command(
    State(state): State<Arc<AppState>>,
    Path(command): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<ExecuteRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    require_auth(&headers, &state).await?;
    let result = execute_and_record(&state, &command, payload.params, "api").await?;
    Ok(Json(ApiResponse::success(result, Some(command))))
}

async fn call_mcp(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<McpRequest>,
) -> impl IntoResponse {
    match payload.method.as_deref() {
        Some("initialize") => {
            return Json(json!({
                "jsonrpc": "2.0",
                "id": payload.id,
                "result": {
                    "protocolVersion": "2025-03-26",
                    "serverInfo": {
                        "name": "twitter-cli",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    }
                }
            }));
        }
        Some("ping") => {
            return Json(json!({
                "jsonrpc": "2.0",
                "id": payload.id,
                "result": {}
            }));
        }
        Some("notifications/initialized") => {
            return Json(json!({
                "jsonrpc": "2.0",
                "id": payload.id,
                "result": {}
            }));
        }
        _ => {}
    }

    if let Err(error) = require_auth(&headers, &state).await {
        return Json(mcp_error_response(
            payload.id,
            -32001,
            error.to_string(),
            Some(error.code().to_string()),
        ));
    }

    let (tool_name, arguments) = match payload.method.as_deref() {
        Some("tools/list") => {
            return Json(json!({
                "jsonrpc": "2.0",
                "id": payload.id,
                "result": {
                    "tools": state.manifest.mcp_tools.iter().map(|tool| json!({
                        "name": tool.name,
                        "description": format!("Maps to twitter-cli command `{}`", tool.command),
                        "inputSchema": build_mcp_input_schema(&state, tool.command),
                        "annotations": {
                            "readOnlyHint": tool.read_only
                        }
                    })).collect::<Vec<_>>()
                }
            }));
        }
        Some("tools/call") => {
            let params = payload
                .params
                .ok_or_else(|| AppError::InvalidParams("params is required".to_string()));
            let params = match params {
                Ok(params) => params,
                Err(error) => {
                    return Json(mcp_error_response(
                        payload.id,
                        -32602,
                        error.to_string(),
                        Some(error.code().to_string()),
                    ));
                }
            };
            (params.name, params.arguments)
        }
        Some(method) => {
            return Json(mcp_error_response(
                payload.id,
                -32601,
                format!("unsupported MCP method: {method}"),
                Some("INVALID_PARAMS".to_string()),
            ));
        }
        None => {
            let tool = match payload.tool {
                Some(tool) => tool,
                None => {
                    return Json(mcp_error_response(
                        payload.id,
                        -32602,
                        "tool is required",
                        Some("INVALID_PARAMS".to_string()),
                    ));
                }
            };
            (tool, payload.arguments.unwrap_or_else(|| json!({})))
        }
    };

    let spec = match state
        .manifest
        .mcp_tools
        .iter()
        .find(|tool| tool.name == tool_name)
    {
        Some(spec) => spec,
        None => {
            return Json(mcp_error_response(
                payload.id,
                -32602,
                format!("unknown MCP tool: {tool_name}"),
                Some("INVALID_PARAMS".to_string()),
            ));
        }
    };

    let result = match execute_and_record(&state, spec.command, arguments, "mcp").await {
        Ok(result) => result,
        Err(error) => {
            return Json(mcp_error_response(
                payload.id,
                -32000,
                error.to_string(),
                Some(error.code().to_string()),
            ));
        }
    };

    let formatted_result =
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "null".to_string());
    Json(json!({
        "jsonrpc": "2.0",
        "id": payload.id,
        "result": {
            "tool": spec.name,
            "command": spec.command,
            "ok": true,
            "data": result.clone(),
            "structuredContent": result,
            "content": [{
                "type": "text",
                "text": formatted_result
            }]
        }
    }))
}

async fn execute_and_record(
    state: &Arc<AppState>,
    command: &str,
    params: Value,
    source: &str,
) -> Result<Value, AppError> {
    let config = {
        let runtime = state.runtime.read().await;
        runtime.config.clone()
    };

    match state.executor.execute(command, params, &config).await {
        Ok(result) => {
            push_history(
                state,
                ExecutionRecord::new(source, command, true, summarize_success(&result)),
            )
            .await;
            Ok(result)
        }
        Err(error) => {
            push_history(
                state,
                ExecutionRecord::new(source, command, false, error.to_string()),
            )
            .await;
            Err(error)
        }
    }
}

async fn push_history(state: &Arc<AppState>, entry: ExecutionRecord) {
    let mut runtime = state.runtime.write().await;
    runtime.recent_executions.push(entry);
    if runtime.recent_executions.len() > 24 {
        let drain = runtime.recent_executions.len() - 24;
        runtime.recent_executions.drain(0..drain);
    }
}

fn summarize_success(result: &Value) -> String {
    match result {
        Value::Array(items) => format!("{} item(s)", items.len()),
        Value::Object(map) => {
            if let Some(message) = map.get("message").and_then(Value::as_str) {
                message.to_string()
            } else if let Some(status) = map.get("status").and_then(Value::as_str) {
                status.to_string()
            } else {
                format!("{} field(s)", map.len())
            }
        }
        Value::String(value) => value.clone(),
        Value::Null => "empty result".to_string(),
        _ => "completed".to_string(),
    }
}

fn build_mcp_input_schema(state: &AppState, command_name: &str) -> Value {
    let Some(command) = state
        .manifest
        .commands
        .iter()
        .find(|command| command.name == command_name)
    else {
        return json!({ "type": "object", "properties": {} });
    };

    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for param in &command.params {
        properties.insert(
            param.name.to_string(),
            json!({
                "type": param.kind,
                "description": param.description,
            }),
        );
        if param.required {
            required.push(param.name);
        }
    }

    json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })
}

fn mcp_error_response(
    id: Option<Value>,
    code: i32,
    message: impl Into<String>,
    data: Option<String>,
) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message.into(),
            "data": data,
        }
    })
}

async fn require_auth(headers: &HeaderMap, state: &AppState) -> Result<(), AppError> {
    let runtime = state.runtime.read().await;
    is_authenticated(headers, &runtime.auth_state)
        .then_some(())
        .ok_or(AppError::AuthRequired)
}

fn maybe_redirect_console(
    headers: &HeaderMap,
    auth_state: &crate::auth::AuthState,
) -> Option<Redirect> {
    if !auth_state.password_initialized {
        return Some(Redirect::to("/setup/password"));
    }
    if is_authenticated(headers, auth_state) {
        None
    } else {
        Some(Redirect::to("/login"))
    }
}

enum NavKind {
    Anonymous,
    Authenticated,
}

fn render_page(title: &str, body: &str, nav_kind: NavKind) -> String {
    let nav = match nav_kind {
        NavKind::Anonymous => {
            r#"<nav><a href="/login">Login</a><a href="/setup/password">Setup Password</a></nav>"#
        }
        NavKind::Authenticated => {
            r#"<nav><a href="/">Console</a><a href="/commands">Commands</a><a href="/mcp">MCP</a><a href="/docs">Docs</a><a href="/settings">Settings</a><form method="post" action="/logout" style="margin:0 0 0 auto"><button type="submit">Logout</button></form></nav>"#
        }
    };
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>{}</title>
  <style>
    :root {{
      --bg: #f5f7fb;
      --fg: #132238;
      --muted: #5f7188;
      --card: #ffffff;
      --border: #d9e0ea;
      --accent: #1166cc;
      --accent-soft: #eaf3ff;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: Georgia, "Iowan Old Style", "Palatino Linotype", serif;
      color: var(--fg);
      background:
        radial-gradient(circle at top left, #ffffff 0, #eef3fb 38%, #f5f7fb 70%),
        linear-gradient(180deg, #f8fafc 0%, #edf2f8 100%);
    }}
    nav {{
      display: flex;
      gap: 18px;
      padding: 18px 24px;
      border-bottom: 1px solid var(--border);
      background: rgba(255,255,255,0.88);
      backdrop-filter: blur(8px);
      position: sticky;
      top: 0;
    }}
    nav a {{
      color: var(--fg);
      text-decoration: none;
      font-weight: 600;
    }}
    main {{
      max-width: 1120px;
      margin: 0 auto;
      padding: 28px 24px 48px;
    }}
    .hero, .card, .single-card {{
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: 18px;
      box-shadow: 0 8px 30px rgba(19,34,56,0.05);
    }}
    .hero {{
      padding: 28px;
      margin-bottom: 22px;
    }}
    .card, .single-card {{
      padding: 22px;
    }}
    .single-card {{
      max-width: 560px;
      margin: 48px auto;
    }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 18px;
    }}
    .grid.two {{
      grid-template-columns: repeat(auto-fit, minmax(360px, 1fr));
    }}
    h1, h2 {{
      margin-top: 0;
    }}
    p, li, dd, dt, label {{
      line-height: 1.5;
    }}
    dl {{
      display: grid;
      grid-template-columns: 110px 1fr;
      gap: 8px 12px;
      margin: 0;
    }}
    dt {{ color: var(--muted); }}
    code, pre, textarea, input {{
      font-family: "SFMono-Regular", Consolas, monospace;
    }}
    pre, textarea, input {{
      width: 100%;
      border: 1px solid var(--border);
      border-radius: 12px;
      background: #fbfcfe;
      padding: 12px;
    }}
    pre {{
      overflow: auto;
      min-height: 60px;
      white-space: pre-wrap;
    }}
    textarea {{
      min-height: 240px;
      resize: vertical;
    }}
    input {{
      min-height: 44px;
      margin-bottom: 12px;
    }}
    button {{
      border: 0;
      border-radius: 999px;
      background: var(--accent);
      color: white;
      padding: 10px 16px;
      font-weight: 700;
      cursor: pointer;
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
    }}
    th, td {{
      text-align: left;
      padding: 10px 12px;
      border-bottom: 1px solid var(--border);
      vertical-align: top;
    }}
    .command-card {{
      border-top: 1px solid var(--border);
      padding: 12px 0;
    }}
    .command-card summary {{
      cursor: pointer;
      display: flex;
      justify-content: space-between;
      gap: 12px;
    }}
    .command-card code {{
      display: block;
      margin-top: 10px;
      padding: 10px;
      background: var(--accent-soft);
      border-radius: 12px;
      white-space: pre-wrap;
    }}
  </style>
</head>
<body>
  {}
  <main>{}</main>
</body>
</html>"#,
        title, nav, body
    )
}

fn build_example_payload(command: &crate::manifest::CommandSpec) -> String {
    let mut map = serde_json::Map::new();
    for param in &command.params {
        let value = match param.name {
            "username" => json!("OpenAI"),
            "query" => json!("openai"),
            "url" => json!("https://x.com/OpenAI/status/2033953592424731072"),
            "text" => json!("hello from twitter-cli"),
            "texts" => json!(["hello from twitter-cli", "follow-up post"]),
            "type" => json!("for-you"),
            "limit" => json!(5),
            _ => json!(""),
        };
        map.insert(param.name.to_string(), value);
    }
    serde_json::to_string_pretty(&Value::Object(map)).unwrap_or_else(|_| "{}".to_string())
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::RwLock;

    use super::{
        build_example_payload, build_mcp_input_schema, mcp_error_response, summarize_success,
    };
    use crate::auth::AuthState;
    use crate::commands::executor::CommandExecutor;
    use crate::commands::registry::CommandRegistry;
    use crate::config::AppConfig;
    use crate::manifest::{build_manifest, command_specs};
    use crate::server::{AppState, RuntimeState};

    fn test_state() -> AppState {
        let config = AppConfig::default();
        AppState {
            config_path: "/tmp/config.toml".to_string(),
            first_run: false,
            manifest: build_manifest(
                "/tmp/config.toml".to_string(),
                "0.0.0.0".to_string(),
                12233,
            ),
            runtime: Arc::new(RwLock::new(RuntimeState {
                auth_state: AuthState::from_config(&config),
                config,
                recent_executions: Vec::new(),
            })),
            executor: CommandExecutor::new(CommandRegistry::new()),
        }
    }

    #[test]
    fn example_payload_supports_thread_texts() {
        let command = command_specs()
            .into_iter()
            .find(|command| command.name == "thread")
            .expect("thread command should exist");
        let payload = build_example_payload(&command);
        assert!(payload.contains("\"texts\""));
        assert!(payload.contains("follow-up post"));
    }

    #[test]
    fn mcp_input_schema_uses_manifest_required_fields() {
        let state = test_state();
        let schema = build_mcp_input_schema(&state, "reply");
        let required = schema["required"].as_array().expect("required array");
        assert!(required.iter().any(|item| item == "url"));
        assert!(required.iter().any(|item| item == "text"));
    }

    #[test]
    fn mcp_error_response_contains_jsonrpc_shape() {
        let payload = mcp_error_response(
            Some(serde_json::json!("abc")),
            -32602,
            "bad input",
            Some("INVALID_PARAMS".to_string()),
        );
        assert_eq!(payload["jsonrpc"], "2.0");
        assert_eq!(payload["id"], "abc");
        assert_eq!(payload["error"]["code"], -32602);
        assert_eq!(payload["error"]["data"], "INVALID_PARAMS");
    }

    #[test]
    fn summarize_success_handles_array_and_message_object() {
        assert_eq!(
            summarize_success(&serde_json::json!([1, 2, 3])),
            "3 item(s)"
        );
        assert_eq!(
            summarize_success(&serde_json::json!({ "message": "done" })),
            "done"
        );
    }
}
