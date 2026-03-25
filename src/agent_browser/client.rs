use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::process::Command;

use crate::agent_browser::parser::parse_response;
use crate::agent_browser::types::{
    AgentBrowserEvalResult, AgentBrowserOptions, AgentBrowserResponse,
};
use crate::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct AgentBrowserClient {
    options: AgentBrowserOptions,
}

impl AgentBrowserClient {
    pub fn new(options: AgentBrowserOptions) -> Self {
        Self { options }
    }

    pub async fn run(&self, args: &[&str]) -> AppResult<AgentBrowserResponse> {
        let mut command = Command::new(&self.options.binary);
        command.arg("--json");
        command
            .arg("--session-name")
            .arg(&self.options.session_name);
        if let Some(cdp_url) = &self.options.cdp_url {
            if !cdp_url.is_empty() {
                command.arg("--cdp-url").arg(cdp_url);
            }
        }
        command.args(args);

        let output = command
            .output()
            .await
            .map_err(|err| AppError::BrowserExecutionFailed(err.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if stderr.contains("No such file") {
                AppError::BrowserNotFound
            } else {
                AppError::BrowserExecutionFailed(stderr)
            });
        }

        parse_response(&output.stdout)
    }

    pub async fn open(&self, url: &str) -> AppResult<Value> {
        let response = self.run(&["open", url]).await?;
        if response.success {
            Ok(response.data.unwrap_or(Value::Null))
        } else {
            Err(AppError::BrowserExecutionFailed(
                response
                    .error
                    .unwrap_or_else(|| "agent-browser open failed".to_string()),
            ))
        }
    }

    pub async fn eval(&self, script: &str) -> AppResult<AgentBrowserEvalResult> {
        let response = self.run(&["eval", script]).await?;
        if !response.success {
            return Err(AppError::BrowserExecutionFailed(
                response
                    .error
                    .unwrap_or_else(|| "agent-browser eval failed".to_string()),
            ));
        }

        let data = response.data.unwrap_or(Value::Null);
        let origin = data
            .get("origin")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        let result = data.get("result").cloned().unwrap_or(Value::Null);
        Ok(AgentBrowserEvalResult { origin, result })
    }

    pub async fn eval_json<T>(&self, script: &str) -> AppResult<T>
    where
        T: DeserializeOwned,
    {
        let evaluated = self.eval(script).await?;
        if let Some(value) = evaluated.result.as_str() {
            serde_json::from_str::<T>(value)
                .map_err(|err| AppError::BrowserExecutionFailed(err.to_string()))
        } else {
            serde_json::from_value::<T>(evaluated.result)
                .map_err(|err| AppError::BrowserExecutionFailed(err.to_string()))
        }
    }

    pub async fn get_url(&self) -> AppResult<String> {
        let response = self.run(&["get", "url"]).await?;
        if !response.success {
            return Err(AppError::BrowserExecutionFailed(
                response
                    .error
                    .unwrap_or_else(|| "agent-browser get url failed".to_string()),
            ));
        }

        response
            .data
            .as_ref()
            .and_then(|value| value.get("url"))
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| AppError::BrowserExecutionFailed("missing URL response".to_string()))
    }

    pub async fn wait_ms(&self, milliseconds: u64) -> AppResult<()> {
        let response = self.run(&["wait", &milliseconds.to_string()]).await?;
        if response.success {
            Ok(())
        } else {
            Err(AppError::BrowserExecutionFailed(
                response
                    .error
                    .unwrap_or_else(|| "agent-browser wait failed".to_string()),
            ))
        }
    }
}
