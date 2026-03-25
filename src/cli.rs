use clap::{Args, Parser, Subcommand};

use crate::commands::executor::CommandExecutor;
use crate::commands::registry::CommandRegistry;
use crate::config;
use crate::errors::{AppError, AppResult};
use crate::manifest::build_manifest;
use crate::server;

#[derive(Debug, Parser)]
#[command(
    name = "twitter-cli",
    version,
    about = "Twitter native CLI and local control plane"
)]
pub struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    Describe(DescribeArgs),
    Execute(ExecuteArgs),
    Serve,
}

#[derive(Debug, Args)]
struct DescribeArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct ExecuteArgs {
    command: String,
    #[arg(long, default_value = "{}")]
    params: String,
}

impl Cli {
    pub async fn run(self) -> AppResult<()> {
        match self.command {
            CliCommand::Describe(args) => {
                let (config, path, _) = config::load_or_init().await?;
                let manifest = build_manifest(
                    path.display().to_string(),
                    config.server.host.clone(),
                    config.server.port,
                );
                if args.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&manifest)
                            .map_err(|err| AppError::Internal(err.to_string()))?
                    );
                } else {
                    println!("twitter-cli");
                    println!("commands: {}", manifest.commands.len());
                    println!("mcp tools: {}", manifest.mcp_tools.len());
                    println!("skills: {}", manifest.skills.len());
                    println!("server: {}", manifest.server_defaults.base_url);
                }
                Ok(())
            }
            CliCommand::Execute(args) => {
                let (config, _, _) = config::load_or_init().await?;
                let params = serde_json::from_str(&args.params)
                    .map_err(|err| AppError::InvalidParams(err.to_string()))?;
                let executor = CommandExecutor::new(CommandRegistry::new());
                let result = executor.execute(&args.command, params, &config).await?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result)
                        .map_err(|err| AppError::Internal(err.to_string()))?
                );
                Ok(())
            }
            CliCommand::Serve => server::serve().await,
        }
    }
}
