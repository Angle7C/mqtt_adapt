use sqlx::FromRow;
use chrono::{DateTime, Utc};
use bytes::Bytes;

#[derive(Debug, Clone, FromRow)]
pub struct RetainedMessage {
    pub id: i64,
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RetainedMessage {
    pub async fn store(
        pool: &sqlx::SqlitePool,
        topic: &str,
        payload: Bytes,
        qos: u8,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO retained_messages (topic, payload, qos, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(topic) DO UPDATE SET
                payload = excluded.payload,
                qos = excluded.qos,
                updated_at = excluded.updated_at
            "#
        )
        .bind(topic)
        .bind(payload.as_ref())
        .bind(qos as i32)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn delete(pool: &sqlx::SqlitePool, topic: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM retained_messages WHERE topic = ?
            "#
        )
        .bind(topic)
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn find_by_topic(
        pool: &sqlx::SqlitePool,
        topic: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT id, topic, payload, qos, created_at, updated_at
            FROM retained_messages
            WHERE topic = ?
            "#
        )
        .bind(topic)
        .fetch_optional(pool)
        .await
    }
    
    pub async fn find_matching(
        pool: &sqlx::SqlitePool,
        topic_filter: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let like_pattern = Self::topic_filter_to_like(topic_filter);
        
        sqlx::query_as::<_, Self>(
            r#"
            SELECT id, topic, payload, qos, created_at, updated_at
            FROM retained_messages
            WHERE topic LIKE ?
            ORDER BY topic
            "#
        )
        .bind(like_pattern)
        .fetch_all(pool)
        .await
    }
    
    fn topic_filter_to_like(filter: &str) -> String {
        if filter == "#" {
            return "%".to_string();
        }
        
        let parts: Vec<&str> = filter.split('/').collect();
        let mut result = String::new();
        
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                result.push('/');
            }
            
            match *part {
                "+" => result.push_str("[^/]*"),
                "#" => {
                    result.push_str("%");
                    break;
                }
                _ => result.push_str(part),
            }
        }
        
        result
    }
    
    pub fn payload_bytes(&self) -> Bytes {
        Bytes::from(self.payload.clone())
    }
    
    pub fn qos_u8(&self) -> u8 {
        self.qos as u8
    }
}
