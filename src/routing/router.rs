use crate::ClinetId;
use crate::routing::channel::RouterChannel;
use crate::routing::event::Event;
use crate::topic::{TopicManager};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use flume::{Sender, unbounded};

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
    pub router_receiver: RouterChannel,
}

impl MessageRouter {
    /// 创建新的消息路由器
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            topic_manager: Arc::new(Mutex::new(TopicManager::new())),
            sender: Arc::new(Mutex::new(HashMap::new())),
            event_sender: tx,
            router_receiver: RouterChannel {
                rx,
            },
        }
    }

    /// 注册客户端通道
    pub fn register_client(&self, client_id: ClinetId, sender: Sender<Event>) {
        let mut senders = self.sender.lock().unwrap();
        senders.insert(client_id, sender);
    }

    /// 移除客户端通道
    pub fn remove_client(&self, client_id: &ClinetId) {
        let mut senders = self.sender.lock().unwrap();
        senders.remove(client_id);
    }

    /// 处理事件
    pub fn handle_event(&self, event: Event) {
        match event {
            Event::ClientConnected(client_id) => {
                println!("Client connected: {}", client_id);
                
                // 发送CONNACK响应
                use crate::protocol::{ConnAckPacket, ConnectReturnCode, MqttPacket};
                
                // 创建CONNACK数据包
                let connack_packet = ConnAckPacket {
                    session_present: false, // 暂时设置为false
                    return_code: ConnectReturnCode::Accepted, // 连接成功
                };
                
                // 转换为MqttPacket
                let mqtt_packet = MqttPacket::ConnAck(connack_packet);
                
                // 获取客户端通道并发送消息
                let senders = self.sender.lock().unwrap();
                if let Some(tx) = senders.get(&client_id) {
                    let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                    if let Err(e) = tx.try_send(event) {
                        println!("Error sending CONNACK to {}: {:?}", client_id, e);
                    }
                }
            }
            Event::ClientDisconnected(client_id) => {
                println!("Client disconnected: {}", client_id);
                self.remove_client(&client_id);
            }
            Event::MessageReceived(client_id, packet) => {
                println!("Message received from {}: {:?}", client_id, packet);
                // 这里应该添加消息路由逻辑
            }
            Event::MessageSent(client_id, packet) => {
                println!("Message sent to {}: {:?}", client_id, packet);
            }
            Event::BroadcastMessage(packet) => {
                println!("Broadcast message: {:?}", packet);
                // 这里应该添加广播逻辑
            }
        }
    }

    /// 启动路由器
    pub async fn start(&self) {
        while let Ok(event) = self.router_receiver.rx.recv_async().await {
            self.handle_event(event);
        }
    }

    /// 发送事件到路由器
    pub fn send_event(&self, event: Event) -> Result<(), flume::TrySendError<Event>> {
        self.event_sender.try_send(event)
    }
} 
