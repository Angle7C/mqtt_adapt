use crate::ClinetId;
use crate::routing::event::Event;
use crate::topic::{TopicManager};
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex};
use flume::{Receiver, Sender, unbounded};
use anyhow::Result;
                use crate::protocol::{ConnAckPacket, ConnectReturnCode, MqttPacket};
/// 消息路由结构体
#[derive(Debug, Clone)]
pub struct MessageRouter {
    /// 主题管理器
    topic_manager: Arc<Mutex<TopicManager>>,
    /// 客户端通道管理器
    sender: Arc<Mutex<HashMap<ClinetId, Sender<Event>>>>,
    /// 事件发送器
    event_sender: Sender<Event>,
    /// 事件通道管理器
    event_receiver: Receiver<Event>,
}

impl MessageRouter {
    /// 创建新的消息路由器
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            topic_manager: Arc::new(Mutex::new(TopicManager::new())),
            sender: Arc::new(Mutex::new(HashMap::new())),
            event_sender: tx,
            event_receiver: rx,
        }
    }
    
    pub fn get_sender(&self)->Sender<Event>{
        self.event_sender.clone()
    }
    /// 注册客户端通道
    pub async fn register_client(&self, client_id: ClinetId, sender: Sender<Event>) -> Result<()> {
        let mut senders = self.sender.lock().await;
        senders.insert(client_id, sender);
        Ok(())
    }

    /// 移除客户端通道
    pub async fn remove_client(&self, client_id: &ClinetId) {
        let mut senders = self.sender.lock().await;
        senders.remove(client_id);
    }

    /// 处理事件
    pub async fn handle_event(&self, event: Event) {
        match event {
            Event::ClientConnected(client_id) => {
                // 创建CONNACK数据包
                let connack_packet = ConnAckPacket {
                    session_present: false, // 暂时设置为false
                    return_code: ConnectReturnCode::Accepted, // 连接成功
                };
                
                // 转换为MqttPacket
                let mqtt_packet = MqttPacket::ConnAck(connack_packet);
                
                // 获取客户端通道并发送消息
                let senders = self.sender.lock().await;
                if let Some(tx) = senders.get(&client_id) {
                    let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                    if let Err(e) = tx.try_send(event) {
                        error!("Error sending CONNACK to {}: {:?}", client_id, e);
                    }
                }
            }
            Event::ClientDisconnected(client_id) => {
                self.remove_client(&client_id).await;
            }
            Event::MessageReceived(client_id, packet) => {
                // 处理不同类型的数据包
                match packet {
                    MqttPacket::Subscribe(subscribe_packet) => {
                        // 处理订阅请求
                        self.handle_subscribe(client_id, subscribe_packet).await;
                    }
                    MqttPacket::Unsubscribe(unsubscribe_packet) => {
                        // 处理取消订阅请求
                        self.handle_unsubscribe(client_id, unsubscribe_packet).await;
                    }
                    MqttPacket::Publish(publish_packet) => {
                        // 处理发布消息
                        self.handle_publish(client_id, publish_packet).await;
                    }
                    _ => {
                        // 其他类型的数据包
                        info!("Other packet type: {:?}", packet);
                    }
                }
            }
            Event::MessageSent(client_id, packet) => {
            }
            Event::BroadcastMessage(packet) => {
                // 这里应该添加广播逻辑
            }
        }
        // 由于 event 中的 packet 字段在 match 分支中已被部分移动，此处无法直接打印 event
    }

    /// 启动路由器
    pub async fn start(self) {
        while let Ok(event) = self.event_receiver.recv_async().await {
            self.handle_event(event).await;
        }
    }
    
    /// 处理订阅请求
    async fn handle_subscribe(&self, client_id: ClinetId, subscribe_packet: crate::protocol::SubscribePacket) {
        let mut code =0x80;
        let  topic_manager = self.topic_manager.lock().await;
        
        // 处理每个订阅主题
        for (topic_filter, qos) in &subscribe_packet.topics {
            // 验证主题过滤器是否有效
            if topic_manager.is_valid_topic_filter(topic_filter) {
                // 添加订阅
                topic_manager.add_subscription(topic_filter, client_id.clone(), *qos).await;
                code=*qos; // 返回实际的QoS级别
            }
        }
        
        // 构建SUBACK数据包
        let suback_packet = crate::protocol::SubAckPacket {
            packet_id: subscribe_packet.packet_id,
            return_codes: code,
        };
       
        
        // 发送SUBACK数据包
        let mqtt_packet = MqttPacket::SubAck(suback_packet);
        let senders = self.sender.lock().await;
        if let Some(tx) = senders.get(&client_id) {
            let event = Event::MessageSent(client_id.clone(), mqtt_packet);
            if let Err(e) = tx.try_send(event) {
                error!("Error sending SUBACK to {}: {:?}", client_id, e);
            }
        }
    }
    
    /// 处理取消订阅请求
    async fn handle_unsubscribe(&self, client_id: ClinetId, unsubscribe_packet: crate::protocol::UnsubscribePacket) {
        let mut topic_manager = self.topic_manager.lock().await;
        
        // 处理每个取消订阅主题
        for topic_filter in &unsubscribe_packet.topics {
            topic_manager.remove_subscription(topic_filter, &client_id).await;
        }
        
        // 构建UNSUBACK数据包
        let unsuback_packet = crate::protocol::UnsubAckPacket {
            packet_id: unsubscribe_packet.packet_id,
        };
        
        // 发送UNSUBACK数据包
        let mqtt_packet = MqttPacket::UnsubAck(unsuback_packet);
        let senders = self.sender.lock().await;
        if let Some(tx) = senders.get(&client_id) {
            let event = Event::MessageSent(client_id.clone(), mqtt_packet);
            if let Err(e) = tx.try_send(event) {
                error!("Error sending UNSUBACK to {}: {:?}", client_id, e);
            }
        }
    }
    
    /// 处理发布消息
    async fn handle_publish(&self, _client_id: ClinetId, publish_packet: crate::protocol::PublishPacket) {
        let topic = publish_packet.topic_name.clone();
        
        let subscribers = {
            let topic_manager = self.topic_manager.lock().await;
            topic_manager.find_subscribers(&topic).await
        };
        
        if subscribers.is_empty() {
            return;
        }
        
        let senders = self.sender.lock().await;
        for subscriber in subscribers {
            if let Some(tx) = senders.get(&subscriber.client_id) {
                let mut pub_packet = publish_packet.clone();
                pub_packet.qos = subscriber.qos;
                
                let mqtt_packet = MqttPacket::Publish(pub_packet);
                let event = Event::MessageSent(subscriber.client_id.clone(), mqtt_packet);
                if let Err(e) = tx.try_send(event) {
                }
            }
        }
    }
    
} 
