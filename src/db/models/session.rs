use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub id: i64,
    pub client_id: String,
    pub clean_session: bool,
    pub connected: bool,
    pub last_connected_at: Option<DateTime<Utc>>,
    pub last_disconnected_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SessionSubscription {
    pub id: i64,
    pub session_id: i64,
    pub topic: String,
    pub qos: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct OfflineMessage {
    pub id: i64,
    pub session_id: i64,
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: i32,
    pub packet_id: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub async fn create(
        pool: &sqlx::SqlitePool,
        client_id: &str,
        clean_session: bool,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        
        let id = sqlx::query(
            r#"
            INSERT INTO sessions (client_id, clean_session, connected, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(client_id)
        .bind(clean_session)
        .bind(true)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?
        .last_insert_rowid();
        
        Ok(Self {
            id,
            client_id: client_id.to_string(),
            clean_session,
            connected: true,
            last_connected_at: Some(now),
            last_disconnected_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn find_by_client_id(
        pool: &sqlx::SqlitePool,
        client_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT id, client_id, clean_session, connected, last_connected_at, 
                   last_disconnected_at, created_at, updated_at
            FROM sessions
            WHERE client_id = ?
            "#
        )
        .bind(client_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_connected(
        pool: &sqlx::SqlitePool,
        client_id: &str,
        connected: bool,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        
        if connected {
            sqlx::query(
                r#"
                UPDATE sessions 
                SET connected = true, last_connected_at = ?, updated_at = ?
                WHERE client_id = ?
                "#
            )
            .bind(now)
            .bind(now)
            .bind(client_id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                r#"
                UPDATE sessions 
                SET connected = false, last_disconnected_at = ?, updated_at = ?
                WHERE client_id = ?
                "#
            )
            .bind(now)
            .bind(now)
            .bind(client_id)
            .execute(pool)
            .await?;
        }
        
        Ok(())
    }
}

impl SessionSubscription {
    pub async fn create(
        pool: &sqlx::SqlitePool,
        session_id: i64,
        topic: &str,
        qos: u8,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        
        let id = sqlx::query(
            r#"
            INSERT INTO session_subscriptions (session_id, topic, qos, created_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(session_id, topic) DO UPDATE SET
                qos = excluded.qos,
                created_at = excluded.created_at
            "#
        )
        .bind(session_id)
        .bind(topic)
        .bind(qos as i32)
        .bind(now)
        .execute(pool)
        .await?
        .last_insert_rowid();
        
        Ok(Self {
            id,
            session_id,
            topic: topic.to_string(),
            qos: qos as i32,
            created_at: now,
        })
    }

    pub async fn find_by_session_id(
        pool: &sqlx::SqlitePool,
        session_id: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT id, session_id, topic, qos, created_at
            FROM session_subscriptions
            WHERE session_id = ?
            ORDER BY topic
            "#
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
    }

    pub fn qos_u8(&self) -> u8 {
        self.qos as u8
    }

    pub async fn delete(
        pool: &sqlx::SqlitePool,
        session_id: i64,
        topic: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"DELETE FROM session_subscriptions WHERE session_id = ? AND topic = ?"#
        )
        .bind(session_id)
        .bind(topic)
        .execute(pool)
        .await?;
        
        Ok(())
    }

    pub async fn delete_all_by_session_id(
        pool: &sqlx::SqlitePool,
        session_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"DELETE FROM session_subscriptions WHERE session_id = ?"#
        )
        .bind(session_id)
        .execute(pool)
        .await?;
        
        Ok(())
    }
}

impl OfflineMessage {
    pub async fn create(
        pool: &sqlx::SqlitePool,
        session_id: i64,
        topic: &str,
        payload: &[u8],
        qos: u8,
        packet_id: Option<u16>,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        
        let id = sqlx::query(
            r#"
            INSERT INTO offline_messages (session_id, topic, payload, qos, packet_id, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(session_id)
        .bind(topic)
        .bind(payload)
        .bind(qos as i32)
        .bind(packet_id.map(|id| id as i32))
        .bind(now)
        .execute(pool)
        .await?
        .last_insert_rowid();
        
        Ok(Self {
            id,
            session_id,
            topic: topic.to_string(),
            payload: payload.to_vec(),
            qos: qos as i32,
            packet_id: packet_id.map(|id| id as i32),
            created_at: now,
        })
    }

    pub async fn find_by_session_id(
        pool: &sqlx::SqlitePool,
        session_id: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT id, session_id, topic, payload, qos, packet_id, created_at
            FROM offline_messages
            WHERE session_id = ?
            ORDER BY created_at ASC
            "#
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
    }

    pub fn payload_bytes(&self) -> Vec<u8> {
        self.payload.clone()
    }

    pub fn qos_u8(&self) -> u8 {
        self.qos as u8
    }

    pub fn packet_id_u16(&self) -> Option<u16> {
        self.packet_id.map(|id| id as u16)
    }

    pub async fn delete(
        pool: &sqlx::SqlitePool,
        id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"DELETE FROM offline_messages WHERE id = ?"#
        )
        .bind(id)
        .execute(pool)
        .await?;
        
        Ok(())
    }

    pub async fn delete_all_by_session_id(
        pool: &sqlx::SqlitePool,
        session_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"DELETE FROM offline_messages WHERE session_id = ?"#
        )
        .bind(session_id)
        .execute(pool)
        .await?;
        
        Ok(())
    }
}
