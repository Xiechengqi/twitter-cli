use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use serde::Serialize;

use crate::config;
use crate::errors::{AppError, AppResult};

const DB_FILE_NAME: &str = "data.db";

#[derive(Debug, Clone, Serialize)]
pub struct AccountEntry {
    pub cdp_port: String,
    pub username: String,
    pub display_name: String,
    pub avatar_url: String,
    pub online: bool,
    pub last_checked: u64,
}

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

fn db_path() -> AppResult<PathBuf> {
    Ok(config::config_dir()?.join(DB_FILE_NAME))
}

impl Db {
    fn init(conn: Connection) -> AppResult<Self> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS accounts (
                cdp_port     TEXT PRIMARY KEY,
                username     TEXT NOT NULL DEFAULT '',
                display_name TEXT NOT NULL DEFAULT '',
                avatar_url   TEXT NOT NULL DEFAULT '',
                online       INTEGER NOT NULL DEFAULT 0,
                last_checked INTEGER NOT NULL DEFAULT 0
            );",
        )
        .map_err(|e| AppError::Internal(format!("init db: {e}")))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn open() -> AppResult<Self> {
        let path = db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::Internal(format!("create db dir: {e}")))?;
        }
        let conn = Connection::open(&path)
            .map_err(|e| AppError::Internal(format!("open db: {e}")))?;
        Self::init(conn)
    }

    pub fn open_in_memory() -> Self {
        let conn = Connection::open_in_memory().expect("in-memory db");
        Self::init(conn).expect("init in-memory db")
    }

    pub fn list_accounts(&self) -> AppResult<Vec<AccountEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT cdp_port, username, display_name, avatar_url, online, last_checked FROM accounts ORDER BY cdp_port")
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(AccountEntry {
                    cdp_port: row.get(0)?,
                    username: row.get(1)?,
                    display_name: row.get(2)?,
                    avatar_url: row.get(3)?,
                    online: row.get::<_, i32>(4)? != 0,
                    last_checked: row.get(5)?,
                })
            })
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
        }
        Ok(result)
    }

    pub fn get_account(&self, cdp_port: &str) -> AppResult<Option<AccountEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT cdp_port, username, display_name, avatar_url, online, last_checked FROM accounts WHERE cdp_port = ?1")
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut rows = stmt
            .query_map([cdp_port], |row| {
                Ok(AccountEntry {
                    cdp_port: row.get(0)?,
                    username: row.get(1)?,
                    display_name: row.get(2)?,
                    avatar_url: row.get(3)?,
                    online: row.get::<_, i32>(4)? != 0,
                    last_checked: row.get(5)?,
                })
            })
            .map_err(|e| AppError::Internal(e.to_string()))?;
        match rows.next() {
            Some(row) => Ok(Some(row.map_err(|e| AppError::Internal(e.to_string()))?)),
            None => Ok(None),
        }
    }

    pub fn upsert_account(&self, entry: &AccountEntry) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO accounts (cdp_port, username, display_name, avatar_url, online, last_checked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(cdp_port) DO UPDATE SET
                username = excluded.username,
                display_name = excluded.display_name,
                avatar_url = excluded.avatar_url,
                online = excluded.online,
                last_checked = excluded.last_checked",
            rusqlite::params![
                entry.cdp_port,
                entry.username,
                entry.display_name,
                entry.avatar_url,
                entry.online as i32,
                entry.last_checked,
            ],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn set_offline(&self, cdp_port: &str, now: u64) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE accounts SET online = 0, last_checked = ?1 WHERE cdp_port = ?2",
            rusqlite::params![now, cdp_port],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Ensure a row exists for this port (insert if missing, don't overwrite existing data).
    pub fn ensure_port(&self, cdp_port: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO accounts (cdp_port) VALUES (?1)",
            [cdp_port],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }
}
