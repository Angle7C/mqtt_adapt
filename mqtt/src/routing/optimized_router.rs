use crate::ClinetId;
use crate::routing::event::Event;
use crate::routing::qos::QoSManager;
use crate::topic::{TopicManager, TopicSubscription};
use log::{error, info};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use flume::{Receiver, Sender, unbounded};
use anyhow::Result;
use crate::protocol::{ConnAckPacket, ConnectReturnCode, MqttPacket, PublishPacket, PubAckPacket, PubRecPacket, PubRelPacket, PubCompPacket};
use lru::LruCache;
use tokio::task::spawn;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::sync::mpsc;
use bytes::Bytes;

// 主题分片数量
const TOPIC_SHARDS: usize = 16;

// 订阅缓存大小
const SUBSCRIPTION_CACHE_SIZE: usize = 10000;

// 消息批量处理大小
const BATCH_SIZE: usize = 100;

// 主题分片
#[derive(Debug, Clone)]
pub struct TopicShard {
    pub shard_id: usize,
    topic_manager: Arc<Mutex<TopicManager>>,
}

impl TopicShard {
    pub fn new(shard_id: usize) -> Self {
        Self {
            shard_id,
            topic_manager: Arc::new(Mutex::new(TopicManager::new())),
        }
    }
}

// 订阅缓存项
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionKey {
    pub topic: String,
}

// 优化的消息路由器
#[derive(Debug, Clone)]
pub struct OptimizedMessageRouter {
    // 主题分片
    topic_shards: Arc<Vec<TopicShard>>,
    // QoS 管理器
    qos_manager: Arc<Mutex<QoSManager>>,
    // 客户端发送器
    senders: Arc<RwLock<HashMap<ClinetId, Sender<Event>>>>,
    // 事件发送器
    event_sender: Sender<Event>,
    event_receiver: Receiver<Event>,
    // 订阅缓存
    subscription_cache: Arc<Mutex<LruCache<SubscriptionKey, Vec<TopicSubscription>>>>,
    // 批量处理队列
    batch_queue: Arc<Mutex<Vec<(ClinetId, MqttPacket)>>>,
    // MQ 消息发送器
    mq_sender: Option<mpsc::Sender<(String, Bytes)>>,
}

impl OptimizedMessageRouter {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        
        // 初始化主题分片
        let mut shards = Vec::with_capacity(TOPIC_SHARDS);
        for i in 0..TOPIC_SHARDS {
            shards.push(TopicShard::new(i));
        }
        
        Self {
            topic_shards: Arc::new(shards),
            qos_manager: Arc::new(Mutex::new(QoSManager::new())),
            senders: Arc::new(RwLock::new(HashMap::new())),
            event_sender: tx,
            event_receiver: rx,
            subscription_cache: Arc::new(Mutex::new(LruCache::new(std::num::NonZeroUsize::new(SUBSCRIPTION_CACHE_SIZE).unwrap()))),
            batch_queue: Arc::new(Mutex::new(Vec::new())),
            mq_sender: None,
        }
    }
    
    /// 设置 MQ 消息发送器
    pub fn set_mq_sender(&mut self, sender: mpsc::Sender<(String, Bytes)>) {
        self.mq_sender = Some(sender);
    }
    
    pub fn get_sender(&self) -> Sender<Event> {
        self.event_sender.clone()
    }
    
    // 计算主题的分片ID
    fn get_topic_shard(&self, topic: &str) -> &TopicShard {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        topic.hash(&mut hasher);
        let hash = hasher.finish() as usize;
        &self.topic_shards[hash % TOPIC_SHARDS]
    }
    
    pub async fn register_client(&self, client_id: &str, sender: Sender<Event>) -> Result<()> {
        let mut senders = self.senders.write().await;
        senders.insert(client_id.to_string(), sender);
        Ok(())
    }

    pub async fn remove_client(&self, client_id: &str) {
        let mut senders = self.senders.write().await;
        senders.remove(client_id);
    }

    pub async fn handle_event(&self, event: Event) {
        match event {
            Event::ClientConnected(client_id) => {
                self.handle_client_connected(client_id).await;
            }
            Event::ClientDisconnected(client_id) => {
                self.handle_client_disconnected(client_id).await;
            }
            Event::MessageReceived(client_id, packet) => {
                self.handle_message_received(client_id, packet).await;
            }
            Event::MessageSent(_client_id, _packet) => {
            }
            Event::BroadcastMessage(_packet) => {
            }
        }
    }

    pub async fn start(self) {
        // 启动批量处理任务
        let batch_queue = self.batch_queue.clone();
        let senders = self.senders.clone();
        spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Self::process_batch(&batch_queue, &senders).await;
            }
        });
        
        // 处理事件
        while let Ok(event) = self.event_receiver.recv_async().await {
            self.handle_event(event).await;
        }
    }
    
    // 批量处理消息
    async fn process_batch(batch_queue: &Arc<Mutex<Vec<(ClinetId, MqttPacket)>>>, senders: &Arc<RwLock<HashMap<ClinetId, Sender<Event>>>>) {
        let mut batch = { 
            let mut queue = batch_queue.lock().await;
            if queue.len() < BATCH_SIZE {
                return;
            }
            std::mem::take(&mut *queue)
        };
        
        if batch.is_empty() {
            return;
        }
        
        // 按客户端分组
        let mut client_messages: HashMap<ClinetId, Vec<MqttPacket>> = HashMap::new();
        for (client_id, packet) in batch {
            client_messages.entry(client_id).or_insert_with(Vec::new).push(packet);
        }
        
        // 发送消息
        let senders = senders.read().await;
        for (client_id, packets) in client_messages {
            if let Some(tx) = senders.get(&client_id) {
                for packet in packets {
                    let event = Event::MessageSent(client_id.clone(), packet);
                    if let Err(e) = tx.try_send(event) {
                        error!("Error sending message to {}: {:?}", client_id, e);
                    }
                }
            }
        }
    }
    
    async fn handle_client_connected(&self, client_id: ClinetId) {
        let connack_packet = ConnAckPacket {
            session_present: false,
            return_code: ConnectReturnCode::Accepted,
        };
        
        let mqtt_packet = MqttPacket::ConnAck(connack_packet);
        
        let senders = self.senders.read().await;
        if let Some(tx) = senders.get(&client_id) {
            let event = Event::MessageSent(client_id.clone(), mqtt_packet);
            if let Err(e) = tx.try_send(event) {
                error!("Error sending CONNACK to {}: {:?}", client_id, e);
            }
        }
    }
    
    async fn handle_client_disconnected(&self, client_id: ClinetId) {
        self.remove_client(&client_id).await;
    }
    
    async fn handle_message_received(&self, client_id: ClinetId, packet: MqttPacket) {
        match packet {
            MqttPacket::Subscribe(subscribe_packet) => {
                self.handle_subscribe(client_id, subscribe_packet).await;
            }
            MqttPacket::Unsubscribe(unsubscribe_packet) => {
                self.handle_unsubscribe(client_id, unsubscribe_packet).await;
            }
            MqttPacket::Publish(publish_packet) => {
                self.handle_publish(client_id, publish_packet).await;
            }
            MqttPacket::PubAck(puback_packet) => {
                self.handle_puback(client_id, puback_packet).await;
            }
            MqttPacket::PubRec(pubrec_packet) => {
                self.handle_pubrec(client_id, pubrec_packet).await;
            }
            MqttPacket::PubRel(pubrel_packet) => {
                self.handle_pubrel(client_id, pubrel_packet).await;
            }
            MqttPacket::PubComp(pubcomp_packet) => {
                self.handle_pubcomp(client_id, pubcomp_packet).await;
            }
            _ => {
                info!("Other packet type: {:?}", packet);
            }
        }
    }
    
    async fn handle_subscribe(&self, client_id: ClinetId, subscribe_packet: crate::protocol::SubscribePacket) {
        let mut code = 0x80;
        
        for (topic_filter, qos) in &subscribe_packet.topics {
            let shard = self.get_topic_shard(topic_filter);
            let mut topic_manager = shard.topic_manager.lock().await;
            topic_manager.add_subscription(client_id.clone(), topic_filter.to_string(), *qos).await;
            code = *qos;
        }
        
        // 清除订阅缓存
        let mut cache = self.subscription_cache.lock().await;
        for (topic_filter, _) in &subscribe_packet.topics {
            cache.pop(&SubscriptionKey { topic: topic_filter.to_string() });
        }
        
        let suback_packet = crate::protocol::SubAckPacket {
            packet_id: subscribe_packet.packet_id,
            return_codes: code,
        };
        
        let mqtt_packet = MqttPacket::SubAck(suback_packet);
        let senders = self.senders.read().await;
        if let Some(tx) = senders.get(&client_id) {
            let event = Event::MessageSent(client_id.clone(), mqtt_packet);
            if let Err(e) = tx.try_send(event) {
                error!("Error sending SUBACK to {}: {:?}", client_id, e);
            }
        }
        
        // 发送保留消息
        for (topic_filter, qos) in &subscribe_packet.topics {
            self.send_retained_messages(client_id.clone(), topic_filter, *qos).await;
        }
    }
    
    async fn send_retained_messages(&self, client_id: ClinetId, topic_filter: &str, qos: u8) {
        let shard = self.get_topic_shard(topic_filter);
        let topic_manager = shard.topic_manager.lock().await;
        let retained_messages = topic_manager.get_retained_messages(topic_filter).await;
        
        if retained_messages.is_empty() {
            return;
        }
        
        let senders = self.senders.read().await;
        if let Some(tx) = senders.get(&client_id) {
            for (topic, retained) in retained_messages {
                let publish_packet = PublishPacket {
                    dup: false,
                    qos: std::cmp::min(qos, retained.qos),
                    retain: true,
                    topic_name: topic,
                    packet_id: None,
                    payload: retained.payload,
                };
                
                let mqtt_packet = MqttPacket::Publish(publish_packet);
                let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                if let Err(e) = tx.try_send(event) {
                    error!("Error sending retained message to {}: {:?}", client_id, e);
                }
            }
        }
    }
    
    async fn handle_unsubscribe(&self, client_id: ClinetId, unsubscribe_packet: crate::protocol::UnsubscribePacket) {
        for topic_filter in &unsubscribe_packet.topics {
            let shard = self.get_topic_shard(topic_filter);
            let mut topic_manager = shard.topic_manager.lock().await;
            topic_manager.remove_subscription(client_id.clone(), topic_filter.to_string()).await;
        }
        
        // 清除订阅缓存
        let mut cache = self.subscription_cache.lock().await;
        for topic_filter in &unsubscribe_packet.topics {
            cache.pop(&SubscriptionKey { topic: topic_filter.to_string() });
        }
        
        let unsuback_packet = crate::protocol::UnsubAckPacket {
            packet_id: unsubscribe_packet.packet_id,
        };
        
        let mqtt_packet = MqttPacket::UnsubAck(unsuback_packet);
        let senders = self.senders.read().await;
        if let Some(tx) = senders.get(&client_id) {
            let event = Event::MessageSent(client_id.clone(), mqtt_packet);
            if let Err(e) = tx.try_send(event) {
                error!("Error sending UNSUBACK to {}: {:?}", client_id, e);
            }
        }
    }
    
    async fn handle_publish(&self, client_id: ClinetId, publish_packet: crate::protocol::PublishPacket) {
        let topic = publish_packet.topic_name.clone();
        let retain = publish_packet.retain;
        let qos = publish_packet.qos;
        let payload = publish_packet.payload.clone();
        
        // 存储保留消息
        if retain {
            let shard = self.get_topic_shard(&topic);
            let mut topic_manager = shard.topic_manager.lock().await;
            topic_manager.store_retained_message(topic.clone(), payload.clone(), qos).await;
        }
        
        // 转发到 MQ
        if let Some(mq_sender) = &self.mq_sender {
            if let Err(e) = mq_sender.send((topic.clone(), payload.clone())).await {
                error!("Failed to send message to MQ: {:?}", e);
            } else {
                info!("Message forwarded to MQ: topic={}", topic);
            }
        }
        
        // 查找订阅者
        let subscribers = self.find_subscribers(&topic).await;
        
        if subscribers.is_empty() {
            return;
        }
        
        // 批量发送消息
        let mut batch = self.batch_queue.lock().await;
        for subscriber in subscribers {
            if subscriber.client_id == client_id {
                continue; // 跳过发布者自己
            }
            
            let mut msg_packet = publish_packet.clone();
            msg_packet.qos = subscriber.qos;
            msg_packet.retain = false;
            
            if subscriber.qos > 0 && msg_packet.packet_id.is_none() {
                let mut qos_manager = self.qos_manager.lock().await;
                msg_packet.packet_id = Some(qos_manager.next_packet_id());
            }
            
            // 添加到批量队列
            batch.push((subscriber.client_id.clone(), MqttPacket::Publish(msg_packet.clone())));
            
            // 处理 QoS 1
            if subscriber.qos == 1 && msg_packet.packet_id.is_some() {
                let mut qos_manager = self.qos_manager.lock().await;
                qos_manager.store_outgoing(msg_packet.packet_id.unwrap(), msg_packet);
            }
        }
        
        // 处理 QoS 响应
        if qos == 1 && publish_packet.packet_id.is_some() {
            let puback_packet = PubAckPacket {
                packet_id: publish_packet.packet_id.unwrap(),
            };
            let mqtt_packet = MqttPacket::PubAck(puback_packet);
            let senders = self.senders.read().await;
            if let Some(tx) = senders.get(&client_id) {
                let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                if let Err(e) = tx.try_send(event) {
                    error!("Error sending PUBACK to {}: {:?}", client_id, e);
                }
            }
        } else if qos == 2 && publish_packet.packet_id.is_some() {
            let packet_id = publish_packet.packet_id.unwrap();
            let mut qos_manager = self.qos_manager.lock().await;
            if qos_manager.store_incoming_qos2(packet_id, publish_packet) {
                drop(qos_manager);
                let pubrec_packet = PubRecPacket {
                    packet_id,
                };
                let mqtt_packet = MqttPacket::PubRec(pubrec_packet);
                let senders = self.senders.read().await;
                if let Some(tx) = senders.get(&client_id) {
                    let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                    if let Err(e) = tx.try_send(event) {
                        error!("Error sending PUBREC to {}: {:?}", client_id, e);
                    }
                }
            }
        }
    }
    
    // 查找订阅者（带缓存）
    async fn find_subscribers(&self, topic: &str) -> Vec<TopicSubscription> {
        // 检查缓存
        let cache_key = SubscriptionKey { topic: topic.to_string() };
        let mut cache = self.subscription_cache.lock().await;
        if let Some(subscribers) = cache.get(&cache_key) {
            return subscribers.clone();
        }
        
        // 从所有分片查找订阅者
        let mut subscribers = Vec::new();
        let mut seen_subscriptions = HashSet::new();
        
        for shard in &*self.topic_shards {
            let topic_manager = shard.topic_manager.lock().await;
            let shard_subscribers = topic_manager.find_subscribers(topic).await;
            
            for sub in shard_subscribers {
                let key = (sub.client_id.clone(), sub.topic.clone());
                if !seen_subscriptions.contains(&key) {
                    seen_subscriptions.insert(key);
                    subscribers.push(sub);
                }
            }
        }
        
        // 更新缓存
        cache.put(cache_key, subscribers.clone());
        
        subscribers
    }
    
    async fn handle_puback(&self, _client_id: ClinetId, puback_packet: PubAckPacket) {
        let mut qos_manager = self.qos_manager.lock().await;
        qos_manager.remove_outgoing(puback_packet.packet_id);
    }
    
    async fn handle_pubrec(&self, client_id: ClinetId, pubrec_packet: PubRecPacket) {
        let mut qos_manager = self.qos_manager.lock().await;
        if qos_manager.remove_outgoing(pubrec_packet.packet_id).is_some() {
            drop(qos_manager);
            let pubrel_packet = PubRelPacket {
                packet_id: pubrec_packet.packet_id,
            };
            let mqtt_packet = MqttPacket::PubRel(pubrel_packet);
            let senders = self.senders.read().await;
            if let Some(tx) = senders.get(&client_id) {
                let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                if let Err(e) = tx.try_send(event) {
                    error!("Error sending PUBREL to {}: {:?}", client_id, e);
                }
            }
        }
    }
    
    async fn handle_pubrel(&self, client_id: ClinetId, pubrel_packet: PubRelPacket) {
        let mut qos_manager = self.qos_manager.lock().await;
        if qos_manager.remove_incoming_qos2(pubrel_packet.packet_id).is_some() {
            drop(qos_manager);
            let pubcomp_packet = PubCompPacket {
                packet_id: pubrel_packet.packet_id,
            };
            let mqtt_packet = MqttPacket::PubComp(pubcomp_packet);
            let senders = self.senders.read().await;
            if let Some(tx) = senders.get(&client_id) {
                let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                if let Err(e) = tx.try_send(event) {
                    error!("Error sending PUBCOMP to {}: {:?}", client_id, e);
                }
            }
        }
    }
    
    async fn handle_pubcomp(&self, _client_id: ClinetId, pubcomp_packet: PubCompPacket) {
        let mut qos_manager = self.qos_manager.lock().await;
        qos_manager.remove_outgoing(pubcomp_packet.packet_id);
    }
    
}