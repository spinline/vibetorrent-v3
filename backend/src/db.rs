use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Row};
use std::time::Duration;
use anyhow::Result;

#[derive(Clone)]
pub struct Db {
    pool: Pool<Sqlite>,
}

impl Db {
    pub async fn new(db_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(db_url)
            .await?;

        let db = Self { pool };
        db.init().await?;
        Ok(db)
    }

    async fn init(&self) -> Result<()> {
        // Create users table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&self.pool)
        .await?;

        // Create sessions table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                token TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL,
                expires_at DATETIME NOT NULL,
                FOREIGN KEY(user_id) REFERENCES users(id)
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // --- User Operations ---

    pub async fn create_user(&self, username: &str, password_hash: &str) -> Result<()> {
        sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
            .bind(username)
            .bind(password_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

        pub async fn get_user_by_username(&self, username: &str) -> Result<Option<(i64, String)>> {
            let row = sqlx::query("SELECT id, password_hash FROM users WHERE username = ?")
                .bind(username)
                .fetch_optional(&self.pool)
                .await?;

            Ok(row.map(|r| (r.get(0), r.get(1))))
        }

        pub async fn get_username_by_id(&self, id: i64) -> Result<Option<String>> {
            let row = sqlx::query("SELECT username FROM users WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;

            Ok(row.map(|r| r.get(0)))
        }
    pub async fn has_users(&self) -> Result<bool> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0 > 0)
    }

    // --- Session Operations ---

    pub async fn create_session(&self, user_id: i64, token: &str, expires_at: i64) -> Result<()> {
        sqlx::query("INSERT INTO sessions (token, user_id, expires_at) VALUES (?, ?, datetime(?, 'unixepoch'))")
            .bind(token)
            .bind(user_id)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_session_user(&self, token: &str) -> Result<Option<i64>> {
        let row = sqlx::query("SELECT user_id FROM sessions WHERE token = ? AND expires_at > datetime('now')")
            .bind(token)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.get(0)))
    }

    pub async fn delete_session(&self, token: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_password(&self, user_id: i64, password_hash: &str) -> Result<()> {
        sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
            .bind(password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_all_sessions_for_user(&self, user_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
