use anyhow::Result;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pub pool: SqlitePool,
}

impl DatabaseConnection {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        
        let conn = Self { pool };
        conn.initialize_tables().await?;
        
        Ok(conn)
    }
    
    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }
    
    async fn initialize_tables(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS retained_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                topic TEXT NOT NULL UNIQUE,
                payload BLOB NOT NULL,
                qos INTEGER NOT NULL CHECK (qos IN (0, 1, 2)),
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_retained_messages_topic ON retained_messages(topic)"
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                client_id TEXT NOT NULL UNIQUE,
                clean_session BOOLEAN NOT NULL DEFAULT 0,
                connected BOOLEAN NOT NULL DEFAULT 0,
                last_connected_at DATETIME,
                last_disconnected_at DATETIME,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_sessions_client_id ON sessions(client_id)"
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS session_subscriptions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                topic TEXT NOT NULL,
                qos INTEGER NOT NULL CHECK (qos IN (0, 1, 2)),
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
                UNIQUE(session_id, topic)
            )"
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_session_subscriptions_session_id ON session_subscriptions(session_id)"
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS offline_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                topic TEXT NOT NULL,
                payload BLOB NOT NULL,
                qos INTEGER NOT NULL CHECK (qos IN (0, 1, 2)),
                packet_id INTEGER,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            )"
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_offline_messages_session_id ON offline_messages(session_id)"
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_offline_messages_created_at ON offline_messages(created_at)"
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
