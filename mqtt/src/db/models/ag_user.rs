use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub created_at: Option<String>,
}

impl User {
    pub async fn find_by_username(
        pool: &sqlx::SqlitePool,
        username: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT id, username, password, created_at 
            FROM users 
            WHERE username = ?
            "#
        )
        .bind(username)
        .fetch_optional(pool)
        .await
    }
    
    pub async fn create(
        &self,
        pool: &sqlx::SqlitePool,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO users (id, username, password, created_at) 
            VALUES (?, ?, ?, ?) 
            RETURNING id, username, password, created_at
            "#
        )
        .bind(self.id)
        .bind(&self.username)
        .bind(&self.password)
        .bind(&self.created_at)
        .fetch_one(pool)
        .await
    }
}