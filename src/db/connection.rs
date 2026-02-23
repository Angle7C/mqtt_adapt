use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{PgPool, Sqlite, SqlitePool};

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
        
        Ok(Self { pool })
    }
    
    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }
}