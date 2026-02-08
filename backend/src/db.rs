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
        db.run_migrations().await?;
        Ok(db)
    }

    async fn run_migrations(&self) -> Result<()> {
        // WAL mode - enables concurrent reads while writing
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&self.pool)
            .await?;

        // NORMAL synchronous - faster than FULL, still safe enough
        sqlx::query("PRAGMA synchronous=NORMAL")
            .execute(&self.pool)
            .await?;

        // 5 second busy timeout - reduces "database locked" errors
        sqlx::query("PRAGMA busy_timeout=5000")
            .execute(&self.pool)
            .await?;

        sqlx::migrate!("./migrations").run(&self.pool).await?;
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

    // --- Push Subscription Operations ---

    pub async fn save_push_subscription(&self, endpoint: &str, p256dh: &str, auth: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO push_subscriptions (endpoint, p256dh, auth) VALUES (?, ?, ?)
             ON CONFLICT(endpoint) DO UPDATE SET p256dh = EXCLUDED.p256dh, auth = EXCLUDED.auth"
        )
        .bind(endpoint)
        .bind(p256dh)
        .bind(auth)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_push_subscription(&self, endpoint: &str) -> Result<()> {
        sqlx::query("DELETE FROM push_subscriptions WHERE endpoint = ?")
            .bind(endpoint)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_all_push_subscriptions(&self) -> Result<Vec<(String, String, String)>> {
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT endpoint, p256dh, auth FROM push_subscriptions"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}
