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
    pub async fn register_client(&self, client_id: ClinetId, sender: Sender<Event>) {
        let mut senders = self.sender.lock().await;
        senders.insert(client_id, sender);
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
                info!("Client connected: {}", client_id);
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
                info!("Client disconnected: {}", client_id);
                self.remove_client(&client_id).await;
            }
            Event::MessageReceived(client_id, packet) => {
                info!("Message received from {}: {:?}", client_id, packet);
                // 这里应该添加消息路由逻辑
            }
            Event::MessageSent(client_id, packet) => {
                info!("Message sent to {}: {:?}", client_id, packet);
            }
            Event::BroadcastMessage(packet) => {
                info!("Broadcast message: {:?}", packet);
                // 这里应该添加广播逻辑
            }
        }
        // 由于 event 中的 packet 字段在 match 分支中已被部分移动，此处无法直接打印 event
        info!("Event handled");
    }

    /// 启动路由器
    pub async fn start(&self) {
        while let Ok(event) = self.event_receiver.recv_async().await {
            self.handle_event(event).await;
        }
    }
    
    // 发送事件到路由器
    // pub async fn send_event(&self, event: Event) -> Result<()> {
    //     self.event_sender.send(event);
    //     Ok(())
    // }
} 
