use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use serde::Serialize;

use crate::config;
use crate::errors::{AppError, AppResult};

const DB_FILE_NAME: &str = "data.db";

#[derive(Debug, Clone, Serialize)]
pub struct PreviewPost {
    pub id: String,
    pub cdp_port: String,
    pub content: String,
    pub image: Option<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountEntry {
    pub cdp_port: String,
    pub username: String,
    pub display_name: String,
    pub avatar_url: String,
    pub online: bool,
    pub last_checked: u64,
    /// Persona description keyed by username in the personas table.
    /// Empty string means no persona is set.
    pub persona: String,
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
            );
            CREATE TABLE IF NOT EXISTS personas (
                username    TEXT PRIMARY KEY,
                persona     TEXT NOT NULL DEFAULT '',
                updated_at  INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS preview_posts (
                id          TEXT PRIMARY KEY,
                cdp_port    TEXT NOT NULL,
                content     TEXT NOT NULL,
                image       TEXT,
                created_at  INTEGER NOT NULL
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
        let conn = self.conn.lock().map_err(|_| AppError::Internal("db lock poisoned".to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT a.cdp_port, a.username, a.display_name, a.avatar_url, a.online, a.last_checked,
                        COALESCE(p.persona, '') AS persona
                 FROM accounts a
                 LEFT JOIN personas p ON p.username = a.username AND a.username != ''
                 ORDER BY a.cdp_port",
            )
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
                    persona: row.get(6)?,
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
        let conn = self.conn.lock().map_err(|_| AppError::Internal("db lock poisoned".to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT a.cdp_port, a.username, a.display_name, a.avatar_url, a.online, a.last_checked,
                        COALESCE(p.persona, '') AS persona
                 FROM accounts a
                 LEFT JOIN personas p ON p.username = a.username AND a.username != ''
                 WHERE a.cdp_port = ?1",
            )
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
                    persona: row.get(6)?,
                })
            })
            .map_err(|e| AppError::Internal(e.to_string()))?;
        match rows.next() {
            Some(row) => Ok(Some(row.map_err(|e| AppError::Internal(e.to_string()))?)),
            None => Ok(None),
        }
    }

    /// Save or update the persona for a given username.
    pub fn upsert_persona(&self, username: &str, persona: &str) -> AppResult<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let conn = self.conn.lock().map_err(|_| AppError::Internal("db lock poisoned".to_string()))?;
        conn.execute(
            "INSERT INTO personas (username, persona, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(username) DO UPDATE SET persona = excluded.persona, updated_at = excluded.updated_at",
            rusqlite::params![username, persona, now],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn upsert_account(&self, entry: &AccountEntry) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|_| AppError::Internal("db lock poisoned".to_string()))?;
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
        let conn = self.conn.lock().map_err(|_| AppError::Internal("db lock poisoned".to_string()))?;
        conn.execute(
            "UPDATE accounts SET online = 0, last_checked = ?1 WHERE cdp_port = ?2",
            rusqlite::params![now, cdp_port],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Ensure a row exists for this port (insert if missing, don't overwrite existing data).
    pub fn ensure_port(&self, cdp_port: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|_| AppError::Internal("db lock poisoned".to_string()))?;
        conn.execute(
            "INSERT OR IGNORE INTO accounts (cdp_port) VALUES (?1)",
            [cdp_port],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn insert_preview_post(
        &self,
        id: &str,
        cdp_port: &str,
        content: &str,
        image: Option<&str>,
    ) -> AppResult<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let conn = self.conn.lock().map_err(|_| AppError::Internal("db lock poisoned".to_string()))?;
        conn.execute(
            "INSERT INTO preview_posts (id, cdp_port, content, image, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, cdp_port, content, image, now],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn list_preview_posts(&self) -> AppResult<Vec<PreviewPost>> {
        let conn = self.conn.lock().map_err(|_| AppError::Internal("db lock poisoned".to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, cdp_port, content, image, created_at FROM preview_posts ORDER BY created_at DESC")
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(PreviewPost {
                    id: row.get(0)?,
                    cdp_port: row.get(1)?,
                    content: row.get(2)?,
                    image: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
        }
        Ok(result)
    }

    pub fn get_preview_post(&self, id: &str) -> AppResult<Option<PreviewPost>> {
        let conn = self.conn.lock().map_err(|_| AppError::Internal("db lock poisoned".to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, cdp_port, content, image, created_at FROM preview_posts WHERE id = ?1")
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut rows = stmt
            .query_map([id], |row| {
                Ok(PreviewPost {
                    id: row.get(0)?,
                    cdp_port: row.get(1)?,
                    content: row.get(2)?,
                    image: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| AppError::Internal(e.to_string()))?;
        match rows.next() {
            Some(row) => Ok(Some(row.map_err(|e| AppError::Internal(e.to_string()))?)),
            None => Ok(None),
        }
    }

    pub fn update_preview_post(
        &self,
        id: &str,
        content: &str,
        image: Option<&str>,
    ) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|_| AppError::Internal("db lock poisoned".to_string()))?;
        conn.execute(
            "UPDATE preview_posts SET content = ?1, image = ?2 WHERE id = ?3",
            rusqlite::params![content, image, id],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn delete_preview_post(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|_| AppError::Internal("db lock poisoned".to_string()))?;
        conn.execute(
            "DELETE FROM preview_posts WHERE id = ?1",
            [id],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }
}
