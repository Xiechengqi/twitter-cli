pub mod routes;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::auth::AuthState;
use crate::commands::executor::CommandExecutor;
use crate::commands::registry::CommandRegistry;
use crate::config::{self, AppConfig};
use crate::errors::{AppError, AppResult};
use crate::manifest::{DescribeManifest, build_manifest};

#[derive(Clone)]
pub struct RuntimeState {
    pub config: AppConfig,
    pub auth_state: AuthState,
    pub recent_executions: Vec<ExecutionRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionRecord {
    pub timestamp: u64,
    pub source: String,
    pub command: String,
    pub ok: bool,
    pub summary: String,
}

impl ExecutionRecord {
    pub fn new(
        source: impl Into<String>,
        command: impl Into<String>,
        ok: bool,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            source: source.into(),
            command: command.into(),
            ok,
            summary: summary.into(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config_path: String,
    pub first_run: bool,
    pub manifest: DescribeManifest,
    pub runtime: Arc<RwLock<RuntimeState>>,
    pub executor: CommandExecutor,
}

pub async fn serve(host_override: Option<String>, port_override: Option<u16>) -> AppResult<()> {
    let (mut config, path, first_run) = config::load_or_init().await?;
    if let Some(host) = host_override {
        config.server.host = host;
    }
    if let Some(port) = port_override {
        config.server.port = port;
    }

    let auth_state = AuthState::from_config(&config);
    let host = config.server.host.clone();
    let port = config.server.port;
    let state = Arc::new(AppState {
        manifest: build_manifest(path.display().to_string(), host.clone(), port),
        runtime: Arc::new(RwLock::new(RuntimeState {
            config,
            auth_state,
            recent_executions: Vec::new(),
        })),
        executor: CommandExecutor::new(CommandRegistry::new()),
        config_path: path.display().to_string(),
        first_run,
    });

    let app = routes::router(state.clone());
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|err: std::net::AddrParseError| AppError::InvalidParams(err.to_string()))?;

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;
    println!("twitter-cli listening on http://{addr}");

    axum::serve(listener, app)
        .await
        .map_err(|err| AppError::Internal(err.to_string()))
}
