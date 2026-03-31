use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tokio::sync::RwLock;

use crate::agent_browser::client::AgentBrowserClient;
use crate::agent_browser::types::AgentBrowserOptions;
use crate::db::{AccountEntry, Db};
use crate::twitter::commands::profile;

/// Probe a single CDP port: navigate to x.com/home, detect logged-in user, run profile query.
async fn probe_port(binary: &str, cdp_port: &str, timeout_secs: u64) -> Option<AccountEntry> {
    let client = AgentBrowserClient::new(AgentBrowserOptions {
        binary: binary.to_string(),
        cdp_port: cdp_port.to_string(),
        session_name: format!("discovery-{cdp_port}"),
        timeout_secs,
    });

    let params = json!({ "username": "" });
    let result = profile::execute(&client, &params).await.ok()?;
    let profiles = result.as_array()?;
    let first = profiles.first()?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Some(AccountEntry {
        cdp_port: cdp_port.to_string(),
        username: first.get("screen_name")?.as_str()?.to_string(),
        display_name: first
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        avatar_url: String::new(),
        online: true,
        last_checked: now,
        persona: String::new(),
    })
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Run discovery for a list of ports. Skips ports that already have a cached online entry in db.
pub async fn discover(
    db: &Db,
    binary: &str,
    cdp_ports: &[String],
    timeout_secs: u64,
    skip_cached: bool,
) {
    for port in cdp_ports {
        if skip_cached {
            if let Ok(Some(entry)) = db.get_account(port) {
                if entry.online && !entry.username.is_empty() {
                    continue;
                }
            }
        }

        match probe_port(binary, port, timeout_secs).await {
            Some(entry) => {
                if let Err(e) = db.upsert_account(&entry) {
                    eprintln!("[discovery] upsert_account({port}): {e}");
                }
            }
            None => {
                if let Err(e) = db.ensure_port(port) {
                    eprintln!("[discovery] ensure_port({port}): {e}");
                }
                if let Err(e) = db.set_offline(port, now_secs()) {
                    eprintln!("[discovery] set_offline({port}): {e}");
                }
            }
        }
    }
}

/// Spawn a background task that runs full discovery every hour.
pub fn spawn_periodic(
    db: Db,
    binary: String,
    cdp_ports: Arc<RwLock<Vec<String>>>,
    timeout_secs: u64,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let ports = cdp_ports.read().await.clone();
            discover(&db, &binary, &ports, timeout_secs, false).await;
        }
    });
}
