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
        
        Ok(())
    }
}
