use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::consumer::{MqConsumer, MqConsumerService};

/// MQ消费者管理器 - 专门负责管理消费者服务
pub struct MqConsumerManager {
    /// 消费者服务列表
    consumer_services: Arc<Mutex<Vec<Arc<Mutex<MqConsumerService>>>>>,
}

impl MqConsumerManager {
    /// 创建新的MQ消费者管理器
    pub fn new() -> Self {
        Self {
            consumer_services: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// 添加消费者服务
    pub async fn add_consumer(&self, consumer: Box<dyn MqConsumer>) -> Result<Arc<Mutex<MqConsumerService>>> {
        let consumer_service = MqConsumerService::new(consumer).await?;
        let consumer_service = Arc::new(Mutex::new(consumer_service));
        
        self.consumer_services.lock().await.push(consumer_service.clone());
        
        Ok(consumer_service)
    }
    
    /// 启动所有消费者服务
    pub async fn start_all(&self) {
        let consumer_services: Vec<_> = self.consumer_services.lock().await.clone();
        
        for consumer_service in consumer_services.iter() {
            let service_clone = consumer_service.clone();
            tokio::spawn(async move {
                let mut service = service_clone.lock().await;
                if let Err(e) = service.start_consuming().await {
                    log::error!("Error starting consumer: {:?}", e);
                }
            });
        }
    }
    
    /// 关闭所有消费者服务
    pub async fn close_all(&self) -> Result<()> {
        let consumer_services: Vec<_> = self.consumer_services.lock().await.clone();
        
        for consumer_service in consumer_services.iter() {
            let mut service = consumer_service.lock().await;
            service.close().await?;
        }
        
        Ok(())
    }
    
    /// 获取消费者服务数量
    pub async fn consumer_count(&self) -> usize {
        self.consumer_services.lock().await.len()
    }
}