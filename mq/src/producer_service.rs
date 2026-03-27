use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use crate::producer::MqProducer;
use crate::message::MqMessage;
use log::{error, info};

/// MQ生产者服务 - 专门负责消息发送, 从MQTT接受到消息并发送到MQ中
pub struct MqProducerService {
    /// MQ生产者
    producer: Arc<Mutex<Box<dyn MqProducer>>>,
    /// 消息接收通道
    rx: mpsc::Receiver<MqMessage>,
    /// 节点ID
    node_id: String,
}

impl MqProducerService {
    /// 创建新的MQ生产者服务
    pub fn new(producer: Box<dyn MqProducer>, node_id: impl Into<String>) -> (Self, mpsc::Sender<MqMessage>) {
        let (tx, rx) = mpsc::channel(10000);
        
        let service = Self {
            producer: Arc::new(Mutex::new(producer)),
            rx,
            node_id: node_id.into(),
        };
        
        (service, tx)
    }
    
    /// 启动服务
    pub async fn start(mut self) {
        info!("MQ producer service started on node {}", self.node_id);
    
        while let Some(message) = self.rx.recv().await {
            self.handle_message(message).await;
        }
    }
    
    /// 处理消息
    async fn handle_message(&self, message: MqMessage) {
        let mut producer = self.producer.lock().await;
        
        if !producer.is_connected() {
            if let Err(e) = producer.connect().await {
                error!("Failed to connect to MQ: {:?}", e);
                return;
            }
        }
        
        if let Err(e) = producer.send_message(message.clone()).await {
            error!("Failed to send message to MQ: {:?}", e);
        } else {
            info!("Message sent to MQ: topic={}", message.topic);
        }
    }
    
    /// 发送消息（直接发送）
    pub async fn send_message(&self, message: MqMessage) -> Result<()> {
        let mut producer = self.producer.lock().await;
        
        if !producer.is_connected() {
            producer.connect().await?;
        }
        
        producer.send_message(message).await
    }
    
    /// 发送原始消息（直接发送）
    pub async fn send_raw_message(&self, topic: impl Into<String>, payload: impl Into<bytes::Bytes>) -> Result<()> {
        let message = MqMessage::new(topic, payload, &self.node_id);
        self.send_message(message).await
    }
    
    /// 批量发送消息（直接发送）
    pub async fn send_batch_messages(&self, messages: &[MqMessage]) -> Result<()> {
        let mut producer = self.producer.lock().await;
        
        if !producer.is_connected() {
            producer.connect().await?;
        }
        
        producer.send_batch_messages(messages).await
    }
    
    /// 关闭连接
    pub async fn close(&self) -> Result<()> {
        let mut producer = self.producer.lock().await;
        if producer.is_connected() {
            producer.disconnect().await
        } else {
            Ok(())
        }
    }
}

