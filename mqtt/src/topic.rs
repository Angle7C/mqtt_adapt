use std::collections::HashMap;
use bytes::Bytes;
use sqlx::SqlitePool;
use crate::db::models::retained_message::RetainedMessage as DbRetainedMessage;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicSubscription {
    pub client_id: String,
    pub topic: String,
    pub qos: u8,
}

#[derive(Debug, Clone)]
pub struct RetainedMessage {
    pub payload: Bytes,
    pub qos: u8,
}

#[derive(Debug, Clone)]
pub struct TopicNode {
    pub topic: String,
    pub children: HashMap<String, TopicNode>,
    pub subscribers: Vec<TopicSubscription>,
    pub retained_message: Option<RetainedMessage>,
}

impl TopicNode {
    pub fn new(topic: String) -> Self {
        Self {
            topic,
            children: HashMap::new(),
            subscribers: Vec::new(),
            retained_message: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TopicManager {
    pub root: TopicNode,
    db_pool: Option<SqlitePool>,
}

impl TopicManager {
    pub fn new() -> Self {
        Self {
            root: TopicNode::new("root".to_string()),
            db_pool: None,
        }
    }

    pub fn with_db(pool: SqlitePool) -> Self {
        Self {
            root: TopicNode::new("root".to_string()),
            db_pool: Some(pool),
        }
    }

    pub fn set_db_pool(&mut self, pool: SqlitePool) {
        self.db_pool = Some(pool);
    }

    pub async fn add_subscription(&mut self, client_id: String, topic: String, qos: u8) {
        let mut current = &mut self.root;
        let parts: Vec<&str> = topic.split('/').collect();

        for part in parts {
            current = current.children.entry(part.to_string()).or_insert_with(|| TopicNode::new(part.to_string()));
        }

        let existing_subscription = current.subscribers.iter().find(|s| s.client_id == client_id);
        if existing_subscription.is_none() {
            current.subscribers.push(TopicSubscription {
                client_id,
                topic: topic.clone(),
                qos,
            });
        }
    }

    pub async fn remove_subscription(&mut self, client_id: String, topic: String) {
        let mut current = &mut self.root;
        let parts: Vec<&str> = topic.split('/').collect();

        for part in parts {
            if let Some(child) = current.children.get_mut(part) {
                current = child;
            } else {
                return;
            }
        }

        current.subscribers.retain(|s| s.client_id != client_id);
    }

    pub async fn find_subscribers(&self, topic: &str) -> Vec<TopicSubscription> {
        let mut subscribers = Vec::new();
        let parts: Vec<&str> = topic.split('/').collect();

        self.match_topic(&self.root, &parts, 0, &mut subscribers);

        subscribers
    }

    fn match_topic(&self, node: &TopicNode, parts: &[&str], index: usize, subscribers: &mut Vec<TopicSubscription>) {
        if index == parts.len() {
            subscribers.extend(node.subscribers.clone());
            return;
        }

        let part = parts[index];

        if let Some(child) = node.children.get(part) {
            self.match_topic(child, parts, index + 1, subscribers);
        }

        if let Some(child) = node.children.get("#") {
            subscribers.extend(child.subscribers.clone());
        }

        if let Some(child) = node.children.get("+") {
            self.match_topic(child, parts, index + 1, subscribers);
        }
    }

    fn find_retained_messages(&self, node: &TopicNode, parts: &[&str], index: usize, messages: &mut Vec<(String, RetainedMessage)>, original_filter: &str) {
        if index == parts.len() {
            if let Some(retained) = &node.retained_message {
                // 构建完整的主题路径
                let topic = original_filter.to_string();
                messages.push((topic, retained.clone()));
            }
            return;
        }

        let part = parts[index];

        if let Some(child) = node.children.get(part) {
            self.find_retained_messages(child, parts, index + 1, messages, original_filter);
        }

        if let Some(child) = node.children.get("#") {
            if let Some(retained) = &child.retained_message {
                let topic = original_filter.to_string();
                messages.push((topic, retained.clone()));
            }
        }

        if let Some(child) = node.children.get("+") {
            self.find_retained_messages(child, parts, index + 1, messages, original_filter);
        }
    }

    pub async fn store_retained_message(&mut self, topic: String, payload: Bytes, qos: u8) {
        // 存储到内存中
        let mut current = &mut self.root;
        let parts: Vec<&str> = topic.split('/').collect();

        for part in parts {
            current = current.children.entry(part.to_string()).or_insert_with(|| TopicNode::new(part.to_string()));
        }

        if payload.is_empty() {
            current.retained_message = None;
        } else {
            current.retained_message = Some(RetainedMessage {
                payload: payload.clone(),
                qos,
            });
        }

        // 存储到数据库（如果有）
        if let Some(pool) = &self.db_pool {
            if payload.is_empty() {
                if let Err(e) = DbRetainedMessage::delete(pool, &topic).await {
                    log::error!("Failed to delete retained message: {}", e);
                }
            } else {
                if let Err(e) = DbRetainedMessage::store(pool, &topic, payload, qos).await {
                    log::error!("Failed to store retained message: {}", e);
                }
            }
        }
    }

    pub async fn get_retained_messages(&self, topic_filter: &str) -> Vec<(String, RetainedMessage)> {
        let mut messages = Vec::new();

        // 从内存中获取
        let parts: Vec<&str> = topic_filter.split('/').collect();
        self.find_retained_messages(&self.root, &parts, 0, &mut messages, topic_filter);

        // 从数据库中获取（如果有）
        if let Some(pool) = &self.db_pool {
            match DbRetainedMessage::find_matching(pool, topic_filter).await {
                Ok(db_messages) => {
                    for db_msg in db_messages {
                        let topic = db_msg.topic.clone();
                        let payload = db_msg.payload_bytes();
                        let qos = db_msg.qos_u8();
                        messages.push((
                            topic,
                            RetainedMessage {
                                payload,
                                qos,
                            },
                        ));
                    }
                }
                Err(e) => {
                    log::error!("Failed to get retained messages: {}", e);
                }
            }
        }

        messages
    }
}
