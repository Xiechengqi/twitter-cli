use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Redirect};
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
        .route("/logout", post(logout))
        .route("/health", get(health))
        .route("/mcp", get(crate::embedded::serve_static).post(call_mcp))
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
        .fallback(crate::embedded::serve_static)
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}

async fn bootstrap(State(state): State<Arc<AppState>>) -> Json<Value> {
    let runtime = state.runtime.read().await;
    Json(json!({
        "first_run": state.first_run,
        "password_required": !runtime.config.auth.password.is_empty() && !runtime.config.auth.password_changed,
        "server": {
            "host": runtime.config.server.host,
            "port": runtime.config.server.port,
        },
        "agent_browser": {
            "binary": runtime.config.agent_browser.binary,
            "detected": runtime.config.agent_browser.binary != "agent-browser",
            "cdp_port": runtime.config.agent_browser.cdp_port,
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
    reject_setup_if_initialized(&runtime.auth_state)?;
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
    let mut runtime = state.runtime.write().await;
    let updated = sanitize_config_update(&runtime.config, payload);

    let path = config::config_path()?;
    config::save(&path, &updated).await?;
    runtime.auth_state = crate::auth::AuthState::from_config(&updated);
    runtime.config = updated;

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
    #[allow(dead_code)]
    old_password: Option<String>,
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
    params: Option<Value>,
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
                    },
                    "instructions": "twitter-cli controls a shared browser session via agent-browser. All tools share one browser instance, so you MUST call them sequentially — never invoke multiple twitter-cli tools in parallel. Wait for each call to complete before starting the next one."
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
            let params_val = match payload.params {
                Some(v) => v,
                None => {
                    return Json(mcp_error_response(
                        payload.id,
                        -32602,
                        "params is required",
                        Some("INVALID_PARAMS".to_string()),
                    ));
                }
            };
            let params: McpToolCall = match serde_json::from_value(params_val) {
                Ok(p) => p,
                Err(e) => {
                    return Json(mcp_error_response(
                        payload.id,
                        -32602,
                        format!("invalid params: {e}"),
                        Some("INVALID_PARAMS".to_string()),
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

    let structured = match &result {
        Value::Array(_) => json!({ "results": result }),
        Value::Object(_) => result.clone(),
        _ => json!({ "value": result }),
    };

    Json(json!({
        "jsonrpc": "2.0",
        "id": payload.id,
        "result": {
            "tool": spec.name,
            "command": spec.command,
            "ok": true,
            "data": result,
            "structuredContent": structured,
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
        Value::Array(items) => {
            if items.is_empty() {
                return "0 item(s)".to_string();
            }
            // Try to extract meaningful content from the first item
            if let Some(first) = items.first().and_then(Value::as_object) {
                // Prefer text > content > title > summary > message > name > author
                for key in &[
                    "text",
                    "content",
                    "title",
                    "summary",
                    "message",
                    "name",
                    "author",
                ] {
                    if let Some(value) = first.get(*key).and_then(Value::as_str) {
                        let trimmed = value.trim();
                        if !trimmed.is_empty() {
                            let preview = if trimmed.len() > 80 {
                                let mut end = 80.min(trimmed.len());
                                while end > 0 && !trimmed.is_char_boundary(end) {
                                    end -= 1;
                                }
                                format!("{}…", &trimmed[..end])
                            } else {
                                trimmed.to_string()
                            };
                            return if items.len() == 1 {
                                preview
                            } else {
                                format!("{} (+{} more)", preview, items.len() - 1)
                            };
                        }
                    }
                }
            }
            format!("{} item(s)", items.len())
        }
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

fn reject_setup_if_initialized(auth_state: &crate::auth::AuthState) -> Result<(), AppError> {
    if auth_state.password_initialized {
        Err(AppError::InvalidParams(
            "password is already configured".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn sanitize_config_update(
    current: &crate::config::AppConfig,
    mut proposed: crate::config::AppConfig,
) -> crate::config::AppConfig {
    proposed.auth = current.auth.clone();
    proposed
}

async fn require_auth(headers: &HeaderMap, state: &AppState) -> Result<(), AppError> {
    let runtime = state.runtime.read().await;
    is_authenticated(headers, &runtime.auth_state)
        .then_some(())
        .ok_or(AppError::AuthRequired)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::RwLock;

    use super::{
        build_mcp_input_schema, mcp_error_response, sanitize_config_update, summarize_success,
    };
    use crate::auth::AuthState;
    use crate::commands::executor::CommandExecutor;
    use crate::commands::registry::CommandRegistry;
    use crate::config::AppConfig;
    use crate::manifest::build_manifest;
    use crate::server::{AppState, RuntimeState};

    fn test_state() -> AppState {
        let config = AppConfig::default();
        AppState {
            first_run: false,
            manifest: build_manifest("/tmp/config.toml".to_string(), "0.0.0.0".to_string(), 12233),
            runtime: Arc::new(RwLock::new(RuntimeState {
                auth_state: AuthState::from_config(&config),
                config,
                recent_executions: Vec::new(),
            })),
            executor: CommandExecutor::new(CommandRegistry::new()),
        }
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
            summarize_success(&serde_json::json!({"message":"done"})),
            "done"
        );
        assert_eq!(summarize_success(&serde_json::json!({"status":"ok"})), "ok");
    }

    #[test]
    fn sanitize_config_update_preserves_auth() {
        let mut current = AppConfig::default();
        current.auth.password = "secret".to_string();
        current.auth.password_changed = true;
        let mut proposed = AppConfig::default();
        proposed.auth.password = "hacker".to_string();
        proposed.server.port = 9999;
        let result = sanitize_config_update(&current, proposed);
        assert_eq!(result.auth.password, "secret");
        assert_eq!(result.server.port, 9999);
    }
}
