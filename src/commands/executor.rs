use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::agent_browser::types::AgentBrowserOptions;
use crate::commands::registry::CommandRegistry;
use crate::config::AppConfig;
use crate::errors::{AppError, AppResult};
use crate::twitter::commands::{
    actions, article, bookmarks, download, likes, notifications, people, profile, replies, search,
    timeline, trending, tweet,
};

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

        if config.agent_browser.cdp_url.is_empty() {
            return Err(AppError::InvalidParams(
                "agent_browser.cdp_url is required".to_string(),
            ));
        }

        let client = AgentBrowserClient::new(AgentBrowserOptions {
            binary: config.agent_browser.binary.clone(),
            session_name: config.agent_browser.session_name.clone(),
            timeout_secs: config.agent_browser.timeout_secs,
        });

        match command.name {
            "article" => article::execute(&client, &params).await,
            "bookmarks" => bookmarks::execute(&client, &params).await,
            "bookmark" => actions::execute_bookmark(&client, &params).await,
            "block" => actions::execute_block(&client, &params).await,
            "accept_dm" => actions::execute_accept_dm(&client, &params).await,
            "reply_dm" => actions::execute_reply_dm(&client, &params).await,
            "delete" => actions::execute_delete(&client, &params).await,
            "download" => download::execute(&client, &params).await,
            "follow" => actions::execute_follow(&client, &params).await,
            "followers" => people::execute_followers(&client, &params).await,
            "followings" => people::execute_followings(&client, &params).await,
            "hide_reply" => actions::execute_hide_reply(&client, &params).await,
            "like" => actions::execute_like(&client, &params).await,
            "likes" => likes::execute(&client, &params).await,
            "notifications" => notifications::execute(&client, &params).await,
            "post" => actions::execute_post(&client, &params).await,
            "profile" => profile::execute(&client, &params).await,
            "reply" => actions::execute_reply(&client, &params).await,
            "retweet" => actions::execute_retweet(&client, &params).await,
            "replies" => replies::execute(&client, &params).await,
            "search" => search::execute(&client, &params).await,
            "thread" => actions::execute_thread(&client, &params).await,
            "timeline" => timeline::execute(&client, &params).await,
            "trending" => trending::execute(&client, &params).await,
            "tweet" => tweet::execute(&client, &params).await,
            "unbookmark" => actions::execute_unbookmark(&client, &params).await,
            "unblock" => actions::execute_unblock(&client, &params).await,
            "unfollow" => actions::execute_unfollow(&client, &params).await,
            "unlike" => actions::execute_unlike(&client, &params).await,
            _ => Ok(json!({
                "status": "planned",
                "message": format!("Command `{}` is registered but not implemented yet", command.name),
                "params": params,
            })),
        }
    }
}
