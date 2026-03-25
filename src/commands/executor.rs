use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::agent_browser::types::AgentBrowserOptions;
use crate::commands::registry::CommandRegistry;
use crate::config::AppConfig;
use crate::errors::{AppError, AppResult};
use crate::twitter::commands::{actions, bookmarks, people, profile, search, timeline, trending};

#[derive(Clone)]
pub struct CommandExecutor {
    registry: CommandRegistry,
}

impl CommandExecutor {
    pub fn new(registry: CommandRegistry) -> Self {
        Self { registry }
    }

    pub async fn execute(
        &self,
        command_name: &str,
        params: Value,
        config: &AppConfig,
    ) -> AppResult<Value> {
        let command = self
            .registry
            .get(command_name)
            .ok_or_else(|| AppError::CommandNotFound(command_name.to_string()))?;

        let client = AgentBrowserClient::new(AgentBrowserOptions {
            binary: config.agent_browser.binary.clone(),
            session_name: config.agent_browser.session_name.clone(),
            cdp_url: if config.agent_browser.cdp_url.is_empty() {
                None
            } else {
                Some(config.agent_browser.cdp_url.clone())
            },
        });

        match command.name {
            "bookmarks" => bookmarks::execute(&client, &params).await,
            "bookmark" => actions::execute_bookmark(&client, &params).await,
            "follow" => actions::execute_follow(&client, &params).await,
            "followers" => people::execute_followers(&client, &params).await,
            "followings" => people::execute_followings(&client, &params).await,
            "like" => actions::execute_like(&client, &params).await,
            "delete" => actions::execute_delete(&client, &params).await,
            "post" => actions::execute_post(&client, &params).await,
            "reply" => actions::execute_reply(&client, &params).await,
            "thread" => actions::execute_thread(&client, &params).await,
            "unbookmark" => actions::execute_unbookmark(&client, &params).await,
            "unfollow" => actions::execute_unfollow(&client, &params).await,
            "unlike" => actions::execute_unlike(&client, &params).await,
            "profile" => profile::execute(&client, &params).await,
            "search" => search::execute(&client, &params).await,
            "timeline" => timeline::execute(&client, &params).await,
            "trending" => trending::execute(&client, &params).await,
            _ => Ok(json!({
                "status": "planned",
                "message": format!("Command `{}` is registered but not implemented yet", command.name),
                "params": params,
            })),
        }
    }
}
