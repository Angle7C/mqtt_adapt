use anyhow::Result;
use async_trait::async_trait;
use crate::mq::message::MqMessage;

/// MQ消费者trait
#[async_trait]
pub trait MqConsumer: Send + Sync {

    /// 连接到MQ服务器
    async fn connect(&mut self) -> Result<()>;
    
    /// 订阅主题
    async fn subscribe(&mut self, topics: &[&str]) -> Result<()>;
    
    /// 订阅单个主题
    async fn subscribe_one(&mut self, topic: &str) -> Result<()> {
        self.subscribe(&[topic]).await
    }
    
    /// 接收消息
    async fn receive_message(&mut self) -> Result<MqMessage>;
    
    /// 提交偏移量
    async fn commit_offset(&mut self) -> Result<()>;
    
    /// 关闭连接
    async fn disconnect(&mut self) -> Result<()>;
    
    /// 检查连接状态
    fn is_connected(&self) -> bool;
}

/// 消息处理回调类型
type MessageHandler = Box<dyn Fn(MqMessage) -> Result<()> + Send + Sync + 'static>;

/// MQ消费者服务
pub struct MqConsumerService {
    /// MQ消费者
    consumer: Box<dyn MqConsumer>,
    /// 消息处理回调
    message_handler: Option<MessageHandler>,
}

impl MqConsumerService {
    /// 创建新的MQ消费者服务
    pub async fn new(mut consumer: Box<dyn MqConsumer>) -> Result<Self> {
        // 连接消费者
        consumer.connect().await?;
        
        Ok(Self {
            consumer,
            message_handler: None,
        })
    }
    
    /// 设置消息处理回调
    pub fn set_message_handler(&mut self, handler: MessageHandler) {
        self.message_handler = Some(handler);
    }
    
    /// 订阅主题
    pub async fn subscribe(&mut self, topics: &[&str]) -> Result<()> {
        self.consumer.subscribe(topics).await
    }
    
    /// 订阅单个主题
    pub async fn subscribe_one(&mut self, topic: &str) -> Result<()> {
        self.consumer.subscribe_one(topic).await
    }
    
    /// 启动消息处理循环
    pub async fn start_consuming(&mut self) -> Result<()> {
        if self.message_handler.is_none() {
            return Err(anyhow::anyhow!("Message handler not set"));
        }
        
        let handler = self.message_handler.as_ref().unwrap();
        
        loop {
            match self.consumer.receive_message().await {
                Ok(message) => {
                    if let Err(e) = handler(message) {
                        log::error!("Error handling message: {:?}", e);
                    }
                    
                    // 提交偏移量
                    if let Err(e) = self.consumer.commit_offset().await {
                        log::error!("Error committing offset: {:?}", e);
                    }
                }
                Err(e) => {
                    log::error!("Error receiving message: {:?}", e);
                    // 可以添加重试逻辑
                }
            }
        }
    }
    
    /// 关闭服务
    pub async fn close(&mut self) -> Result<()> {
        self.consumer.disconnect().await
    }
}
