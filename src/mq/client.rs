use anyhow::Result;
use crate::mq::message::MqMessage;

/// MQ客户端trait
#[async_trait::async_trait]
pub trait MqClient {
    /// 连接到MQ服务器
    async fn connect(&mut self) -> Result<()>;
    
    /// 发送消息到MQ
    async fn send_message(&self, message: MqMessage) -> Result<()>;
    
    /// 发送原始消息（字节形式）
    async fn send_raw_message(&self, topic: &str, payload: &[u8], qos: u8, retain: bool) -> Result<()> {
        let message = MqMessage::new(topic, bytes::Bytes::copy_from_slice(payload), qos, retain, "default");
        self.send_message(message).await
    }
    
    /// 关闭连接
    async fn disconnect(&mut self) -> Result<()>;
    
    /// 检查连接状态
    fn is_connected(&self) -> bool;
}
