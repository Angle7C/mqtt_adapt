use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::producer::MqProducer;
use crate::message::MqMessage;

/// MQ生产者服务 - 专门负责消息发送
pub struct MqProducerService {
    /// MQ生产者
    producer: Arc<Mutex<Box<dyn MqProducer>>>,
}

impl MqProducerService {
    /// 创建新的MQ生产者服务
    pub async fn new(mut producer: Box<dyn MqProducer>) -> Result<Self> {
        producer.connect().await?;
        
        Ok(Self {
            producer: Arc::new(Mutex::new(producer)),
        })
    }
    
    /// 发送消息
    pub async fn send_message(&self, message: MqMessage) -> Result<()> {
        let producer = self.producer.lock().await;
        producer.send_message(message).await
    }
    
    /// 发送原始消息
    pub async fn send_raw(
        &self, 
        topic: &str, 
        payload: &[u8], 
        qos: u8, 
        retain: bool, 
        node_id: &str
    ) -> Result<()> {
        let message = MqMessage::new(topic, Bytes::copy_from_slice(payload), qos, retain, node_id);
        self.send_message(message).await
    }
    
    /// 发送带分区信息的消息
    pub async fn send_with_partition(
        &self, 
        message: MqMessage
    ) -> Result<()> {
        self.send_message(message).await
    }
    
    /// 关闭生产者服务
    pub async fn close(&self) -> Result<()> {
        let mut producer = self.producer.lock().await;
        producer.disconnect().await
    }
}