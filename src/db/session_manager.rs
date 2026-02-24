
use anyhow::Result;
use sqlx::SqlitePool;
use log::{info, error};

use crate::db::models::session::{Session, SessionSubscription, OfflineMessage};
use crate::protocol::PublishPacket;

pub struct SessionManager;

impl SessionManager {
    pub async fn handle_connect(
        pool: &SqlitePool,
        client_id: &str,
        clean_session: bool,
    ) -> Result<(Session, bool, Vec<SessionSubscription>, Vec<OfflineMessage>)> {
        let existing_session = Session::find_by_client_id(pool, client_id).await?;
        
        let (session, session_present, subscriptions, offline_messages) = if let Some(mut existing) = existing_session {
            if clean_session {
                info!("Clean session requested for client {}, clearing old session", client_id);
                SessionSubscription::delete_all_by_session_id(pool, existing.id).await?;
                OfflineMessage::delete_all_by_session_id(pool, existing.id).await?;
                
                existing.clean_session = true;
                Session::update_connected(pool, client_id, true).await?;
                
                (existing, false, vec![], vec![])
            } else {
                info!("Resuming session for client {}", client_id);
                Session::update_connected(pool, client_id, true).await?;
                
                let subscriptions = SessionSubscription::find_by_session_id(pool, existing.id).await?;
                let offline_messages = OfflineMessage::find_by_session_id(pool, existing.id).await?;
                
                (existing, true, subscriptions, offline_messages)
            }
        } else {
            info!("Creating new session for client {}", client_id);
            let session = Session::create(pool, client_id, clean_session).await?;
            (session, false, vec![], vec![])
        };
        
        Ok((session, session_present, subscriptions, offline_messages))
    }

    pub async fn handle_disconnect(
        pool: &SqlitePool,
        client_id: &str,
    ) -> Result<()> {
        if let Some(session) = Session::find_by_client_id(pool, client_id).await? {
            if session.clean_session {
                info!("Cleaning up session for client {}", client_id);
                SessionSubscription::delete_all_by_session_id(pool, session.id).await?;
                OfflineMessage::delete_all_by_session_id(pool, session.id).await?;
            } else {
                Session::update_connected(pool, client_id, false).await?;
            }
        }
        Ok(())
    }

    pub async fn add_subscription(
        pool: &SqlitePool,
        client_id: &str,
        topic: &str,
        qos: u8,
    ) -> Result<()> {
        if let Some(session) = Session::find_by_client_id(pool, client_id).await? {
            if !session.clean_session {
                SessionSubscription::create(pool, session.id, topic, qos).await?;
            }
        }
        Ok(())
    }

    pub async fn remove_subscription(
        pool: &SqlitePool,
        client_id: &str,
        topic: &str,
    ) -> Result<()> {
        if let Some(session) = Session::find_by_client_id(pool, client_id).await? {
            if !session.clean_session {
                SessionSubscription::delete(pool, session.id, topic).await?;
            }
        }
        Ok(())
    }

    pub async fn store_offline_message(
        pool: &SqlitePool,
        client_id: &str,
        publish: &PublishPacket,
    ) -> Result<()> {
        if let Some(session) = Session::find_by_client_id(pool, client_id).await? {
            if !session.clean_session && !session.connected {
                OfflineMessage::create(
                    pool,
                    session.id,
                    &publish.topic_name,
                    &publish.payload,
                    publish.qos,
                    publish.packet_id,
                ).await?;
            }
        }
        Ok(())
    }

    pub async fn delete_offline_message(
        pool: &SqlitePool,
        message_id: i64,
    ) -> Result<()> {
        OfflineMessage::delete(pool, message_id).await?;
        Ok(())
    }

    pub async fn is_client_connected(
        pool: &SqlitePool,
        client_id: &str,
    ) -> Result<bool> {
        if let Some(session) = Session::find_by_client_id(pool, client_id).await? {
            Ok(session.connected)
        } else {
            Ok(false)
        }
    }
}
