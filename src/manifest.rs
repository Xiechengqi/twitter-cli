use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ParamSpec {
    pub name: &'static str,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub required: bool,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandSpec {
    pub name: &'static str,
    pub category: &'static str,
    pub wave: u8,
    pub execution_mode: &'static str,
    pub summary: &'static str,
    pub requires_auth: bool,
    pub params: Vec<ParamSpec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolSpec {
    pub name: &'static str,
    pub command: &'static str,
    pub read_only: bool,
    pub requires_auth: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillStep {
    pub r#use: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillSpec {
    pub name: &'static str,
    pub summary: &'static str,
    pub requires_auth: bool,
    pub steps: Vec<SkillStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SiteSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeSpec {
    pub binary: &'static str,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerDefaults {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthModelSpec {
    pub mode: &'static str,
    pub cookie_name: &'static str,
    pub bearer_format: &'static str,
    pub first_run_requires_password_setup: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentBrowserSpec {
    pub binding: &'static str,
    pub binary_auto_detect: bool,
    pub supports_cdp_url: bool,
    pub default_session_name: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct DescribeManifest {
    pub site: SiteSpec,
    pub runtime: RuntimeSpec,
    pub server_defaults: ServerDefaults,
    pub auth_model: AuthModelSpec,
    pub agent_browser: AgentBrowserSpec,
    pub commands: Vec<CommandSpec>,
    pub mcp_tools: Vec<ToolSpec>,
    pub skills: Vec<SkillSpec>,
}

pub fn build_manifest(config_path: String, host: String, port: u16) -> DescribeManifest {
    DescribeManifest {
        site: SiteSpec {
            id: "twitter",
            name: "Twitter CLI",
            version: env!("CARGO_PKG_VERSION"),
        },
        runtime: RuntimeSpec {
            binary: "twitter-cli",
            config_path,
        },
        server_defaults: ServerDefaults {
            base_url: format!("http://{host}:{port}"),
            host,
            port,
        },
        auth_model: AuthModelSpec {
            mode: "shared-password",
            cookie_name: "twitter_cli_token",
            bearer_format: "Authorization: Bearer <password>",
            first_run_requires_password_setup: true,
        },
        agent_browser: AgentBrowserSpec {
            binding: "cli",
            binary_auto_detect: true,
            supports_cdp_url: true,
            default_session_name: "twitter-cli",
        },
        commands: command_specs(),
        mcp_tools: tool_specs(),
        skills: skill_specs(),
    }
}

pub fn command_specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            name: "profile",
            category: "read",
            wave: 1,
            execution_mode: "api-first",
            summary: "Fetch a Twitter profile",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "username",
                kind: "string",
                required: false,
                description: "Twitter handle without @",
            }],
        },
        CommandSpec {
            name: "timeline",
            category: "read",
            wave: 1,
            execution_mode: "api-first",
            summary: "Fetch the Twitter timeline",
            requires_auth: true,
            params: vec![
                ParamSpec {
                    name: "type",
                    kind: "string",
                    required: false,
                    description: "for-you or following",
                },
                ParamSpec {
                    name: "limit",
                    kind: "integer",
                    required: false,
                    description: "Max items to return",
                },
            ],
        },
        CommandSpec {
            name: "trending",
            category: "read",
            wave: 1,
            execution_mode: "api-first",
            summary: "Fetch trending topics",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "limit",
                kind: "integer",
                required: false,
                description: "Max items to return",
            }],
        },
        CommandSpec {
            name: "bookmarks",
            category: "read",
            wave: 2,
            execution_mode: "api-first",
            summary: "Fetch bookmarks",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "limit",
                kind: "integer",
                required: false,
                description: "Max items to return",
            }],
        },
        CommandSpec {
            name: "search",
            category: "read",
            wave: 1,
            execution_mode: "hybrid",
            summary: "Search tweets",
            requires_auth: true,
            params: vec![
                ParamSpec {
                    name: "query",
                    kind: "string",
                    required: true,
                    description: "Search query",
                },
                ParamSpec {
                    name: "limit",
                    kind: "integer",
                    required: false,
                    description: "Max items to return",
                },
            ],
        },
        CommandSpec {
            name: "followers",
            category: "read",
            wave: 1,
            execution_mode: "ui-first",
            summary: "List accounts following a user",
            requires_auth: true,
            params: vec![
                ParamSpec {
                    name: "username",
                    kind: "string",
                    required: false,
                    description: "Twitter handle without @",
                },
                ParamSpec {
                    name: "limit",
                    kind: "integer",
                    required: false,
                    description: "Max items to return",
                },
            ],
        },
        CommandSpec {
            name: "followings",
            category: "read",
            wave: 1,
            execution_mode: "ui-first",
            summary: "List accounts a user follows",
            requires_auth: true,
            params: vec![
                ParamSpec {
                    name: "username",
                    kind: "string",
                    required: false,
                    description: "Twitter handle without @",
                },
                ParamSpec {
                    name: "limit",
                    kind: "integer",
                    required: false,
                    description: "Max items to return",
                },
            ],
        },
        CommandSpec {
            name: "like",
            category: "write",
            wave: 2,
            execution_mode: "ui-first",
            summary: "Like a tweet",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "url",
                kind: "string",
                required: true,
                description: "Tweet URL",
            }],
        },
        CommandSpec {
            name: "bookmark",
            category: "write",
            wave: 2,
            execution_mode: "ui-first",
            summary: "Bookmark a tweet",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "url",
                kind: "string",
                required: true,
                description: "Tweet URL",
            }],
        },
        CommandSpec {
            name: "retweet",
            category: "write",
            wave: 2,
            execution_mode: "ui-first",
            summary: "Retweet a tweet",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "url",
                kind: "string",
                required: true,
                description: "Tweet URL",
            }],
        },
        CommandSpec {
            name: "follow",
            category: "write",
            wave: 2,
            execution_mode: "ui-first",
            summary: "Follow a user",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "username",
                kind: "string",
                required: true,
                description: "Twitter handle without @",
            }],
        },
        CommandSpec {
            name: "unlike",
            category: "write",
            wave: 2,
            execution_mode: "ui-first",
            summary: "Unlike a tweet",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "url",
                kind: "string",
                required: true,
                description: "Tweet URL",
            }],
        },
        CommandSpec {
            name: "unbookmark",
            category: "write",
            wave: 2,
            execution_mode: "ui-first",
            summary: "Remove a bookmark from a tweet",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "url",
                kind: "string",
                required: true,
                description: "Tweet URL",
            }],
        },
        CommandSpec {
            name: "unfollow",
            category: "write",
            wave: 2,
            execution_mode: "ui-first",
            summary: "Unfollow a user",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "username",
                kind: "string",
                required: true,
                description: "Twitter handle without @",
            }],
        },
        CommandSpec {
            name: "post",
            category: "write",
            wave: 3,
            execution_mode: "ui-first",
            summary: "Post a new tweet",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "text",
                kind: "string",
                required: true,
                description: "Tweet text",
            }],
        },
        CommandSpec {
            name: "reply",
            category: "write",
            wave: 3,
            execution_mode: "ui-first",
            summary: "Reply to a tweet",
            requires_auth: true,
            params: vec![
                ParamSpec {
                    name: "url",
                    kind: "string",
                    required: true,
                    description: "Tweet URL",
                },
                ParamSpec {
                    name: "text",
                    kind: "string",
                    required: true,
                    description: "Reply text",
                },
            ],
        },
        CommandSpec {
            name: "thread",
            category: "write",
            wave: 3,
            execution_mode: "ui-first",
            summary: "Post a new thread",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "texts",
                kind: "array",
                required: true,
                description: "Ordered list of thread post texts",
            }],
        },
        CommandSpec {
            name: "delete",
            category: "write",
            wave: 3,
            execution_mode: "ui-first",
            summary: "Delete a tweet",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "url",
                kind: "string",
                required: true,
                description: "Tweet URL",
            }],
        },
        // Wave 4: New commands
        CommandSpec {
            name: "likes",
            category: "read",
            wave: 4,
            execution_mode: "api-first",
            summary: "Fetch liked tweets for a user",
            requires_auth: true,
            params: vec![
                ParamSpec {
                    name: "username",
                    kind: "string",
                    required: false,
                    description: "Twitter handle without @",
                },
                ParamSpec {
                    name: "limit",
                    kind: "integer",
                    required: false,
                    description: "Max items to return",
                },
            ],
        },
        CommandSpec {
            name: "notifications",
            category: "read",
            wave: 4,
            execution_mode: "ui-first",
            summary: "Fetch notifications",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "limit",
                kind: "integer",
                required: false,
                description: "Max items to return",
            }],
        },
        CommandSpec {
            name: "article",
            category: "read",
            wave: 4,
            execution_mode: "api-first",
            summary: "Fetch a Twitter article from a tweet",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "url",
                kind: "string",
                required: true,
                description: "Tweet URL",
            }],
        },
        CommandSpec {
            name: "download",
            category: "read",
            wave: 4,
            execution_mode: "ui-first",
            summary: "Extract media URLs from a tweet or profile",
            requires_auth: true,
            params: vec![
                ParamSpec {
                    name: "url",
                    kind: "string",
                    required: true,
                    description: "Tweet or profile URL",
                },
                ParamSpec {
                    name: "limit",
                    kind: "integer",
                    required: false,
                    description: "Max items to return",
                },
            ],
        },
        CommandSpec {
            name: "block",
            category: "write",
            wave: 4,
            execution_mode: "ui-first",
            summary: "Block a user",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "username",
                kind: "string",
                required: true,
                description: "Twitter handle without @",
            }],
        },
        CommandSpec {
            name: "unblock",
            category: "write",
            wave: 4,
            execution_mode: "ui-first",
            summary: "Unblock a user",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "username",
                kind: "string",
                required: true,
                description: "Twitter handle without @",
            }],
        },
        CommandSpec {
            name: "hide_reply",
            category: "write",
            wave: 4,
            execution_mode: "ui-first",
            summary: "Hide a reply to your tweet",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "url",
                kind: "string",
                required: true,
                description: "Tweet URL",
            }],
        },
        CommandSpec {
            name: "accept_dm",
            category: "write",
            wave: 4,
            execution_mode: "ui-first",
            summary: "Accept DM requests matching keywords (comma-separated for OR logic)",
            requires_auth: true,
            params: vec![
                ParamSpec {
                    name: "query",
                    kind: "string",
                    required: true,
                    description: "Keywords to match, comma-separated for OR (e.g. \"hello,微信\")",
                },
                ParamSpec {
                    name: "max",
                    kind: "integer",
                    required: false,
                    description: "Max requests to accept",
                },
            ],
        },
        CommandSpec {
            name: "reply_dm",
            category: "write",
            wave: 4,
            execution_mode: "ui-first",
            summary: "Send a message to recent DM conversations",
            requires_auth: true,
            params: vec![
                ParamSpec {
                    name: "text",
                    kind: "string",
                    required: true,
                    description: "Message text to send",
                },
                ParamSpec {
                    name: "max",
                    kind: "integer",
                    required: false,
                    description: "Max conversations to reply to (default: 20)",
                },
                ParamSpec {
                    name: "skip_replied",
                    kind: "boolean",
                    required: false,
                    description: "Skip conversations where this message was already sent (default: true)",
                },
            ],
        },
        CommandSpec {
            name: "tweet",
            category: "read",
            wave: 4,
            execution_mode: "api-first",
            summary: "Fetch a single tweet's details by URL",
            requires_auth: true,
            params: vec![ParamSpec {
                name: "url",
                kind: "string",
                required: true,
                description: "Tweet URL",
            }],
        },
        CommandSpec {
            name: "replies",
            category: "read",
            wave: 4,
            execution_mode: "api-first",
            summary: "Fetch replies to a tweet",
            requires_auth: true,
            params: vec![
                ParamSpec {
                    name: "url",
                    kind: "string",
                    required: true,
                    description: "Tweet URL",
                },
                ParamSpec {
                    name: "limit",
                    kind: "integer",
                    required: false,
                    description: "Max items to return",
                },
            ],
        },
    ]
}

pub fn tool_specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "twitter_profile",
            command: "profile",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_timeline",
            command: "timeline",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_trending",
            command: "trending",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_search",
            command: "search",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_followers",
            command: "followers",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_followings",
            command: "followings",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_bookmarks",
            command: "bookmarks",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_like",
            command: "like",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_bookmark",
            command: "bookmark",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_retweet",
            command: "retweet",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_follow",
            command: "follow",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_unlike",
            command: "unlike",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_unbookmark",
            command: "unbookmark",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_unfollow",
            command: "unfollow",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_post",
            command: "post",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_reply",
            command: "reply",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_thread",
            command: "thread",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_delete",
            command: "delete",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_likes",
            command: "likes",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_notifications",
            command: "notifications",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_article",
            command: "article",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_download",
            command: "download",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_block",
            command: "block",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_unblock",
            command: "unblock",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_hide_reply",
            command: "hide_reply",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_accept_dm",
            command: "accept_dm",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_reply_dm",
            command: "reply_dm",
            read_only: false,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_tweet",
            command: "tweet",
            read_only: true,
            requires_auth: true,
        },
        ToolSpec {
            name: "twitter_replies",
            command: "replies",
            read_only: true,
            requires_auth: true,
        },
    ]
}

pub fn skill_specs() -> Vec<SkillSpec> {
    vec![
        SkillSpec {
            name: "research_account",
            summary: "Analyze account profile, timeline, and followings",
            requires_auth: true,
            steps: vec![
                SkillStep { r#use: "profile" },
                SkillStep { r#use: "timeline" },
                SkillStep {
                    r#use: "followings",
                },
            ],
        },
        SkillSpec {
            name: "monitor_keyword",
            summary: "Monitor keyword and trend activity",
            requires_auth: true,
            steps: vec![
                SkillStep { r#use: "search" },
                SkillStep { r#use: "trending" },
            ],
        },
        SkillSpec {
            name: "prepare_reply_context",
            summary: "Collect context before composing a reply",
            requires_auth: true,
            steps: vec![
                SkillStep { r#use: "profile" },
                SkillStep { r#use: "search" },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{command_specs, skill_specs, tool_specs};

    #[test]
    fn write_wave_three_commands_include_thread_and_delete() {
        let commands = command_specs();
        assert!(commands.iter().any(|command| command.name == "thread"));
        assert!(commands.iter().any(|command| command.name == "delete"));
    }

    #[test]
    fn tool_specs_include_new_write_tools() {
        let tools = tool_specs();
        assert!(tools.iter().any(|tool| tool.name == "twitter_thread"));
        assert!(tools.iter().any(|tool| tool.name == "twitter_delete"));
        assert!(tools.iter().any(|tool| tool.name == "twitter_retweet"));
    }

    #[test]
    fn skill_specs_include_prepare_reply_context() {
        let skills = skill_specs();
        assert!(
            skills
                .iter()
                .any(|skill| skill.name == "prepare_reply_context")
        );
    }
}
