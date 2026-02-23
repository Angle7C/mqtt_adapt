use anyhow::Result;
use bytes::Bytes;
use flume::{Receiver, Sender};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task;
use crate::mq::producer::MqProducer;
use crate::mq::consumer::{MqConsumer, MqConsumerService};
use crate::mq::message::MqMessage;

/// 消息响应回调类型
type ResponseCallback = Box<dyn FnOnce(MqMessage) -> Result<()> + Send + Sync>;

/// MQ服务结构体
pub struct MqService {
    /// MQ生产者
    producer: Arc<Mutex<Box<dyn MqProducer>>>,
    /// 消费者服务列表
    consumer_services: Arc<Mutex<Vec<Arc<Mutex<MqConsumerService>>>>>,
    /// 响应通道
    response_sender: Sender<MqMessage>,
    response_receiver: Receiver<MqMessage>,
    /// 回调映射
    callbacks: Arc<Mutex<HashMap<String, ResponseCallback>>>,
}

impl MqService {
    /// 创建新的MQ服务
    pub async fn new(mut producer: Box<dyn MqProducer>) -> Result<Self> {
        // 连接生产者
        producer.connect().await?;
        
        // 创建响应通道
        let (tx, rx) = flume::unbounded();
        
        Ok(Self {
            producer: Arc::new(Mutex::new(producer)),
            consumer_services: Arc::new(Mutex::new(Vec::new())),
            response_sender: tx,
            response_receiver: rx,
            callbacks: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// 启动响应处理任务
    pub async fn start(&self) {
        let receiver = self.response_receiver.clone();
        let callbacks = self.callbacks.clone();
        
        task::spawn(async move {
            while let Ok(message) = receiver.recv_async().await {
                let topic = message.topic.clone();
                
                // 查找并执行回调
                if let Some(callback) = callbacks.lock().unwrap().remove(&topic) {
                    if let Err(e) = callback(message) {
                        log::error!("Error executing callback for topic {}: {:?}", topic, e);
                    }
                }
            }
        });
    }
    
    /// 发送消息并注册响应回调
    pub async fn send_with_callback(
        &self, 
        message: MqMessage, 
        response_topic: String, 
        callback: ResponseCallback
    ) -> Result<()> {
        // 注册回调
        self.callbacks.lock().unwrap().insert(response_topic, callback);
        
        // 发送消息
        let producer = self.producer.lock().unwrap();
        producer.send_message(message).await
    }
    
    /// 发送原始消息并注册响应回调
    pub async fn send_raw_with_callback(
        &self, 
        topic: &str, 
        payload: &[u8], 
        qos: u8, 
        retain: bool, 
        node_id: &str,
        response_topic: String, 
        callback: ResponseCallback
    ) -> Result<()> {
        let message = MqMessage::new(topic, Bytes::copy_from_slice(payload), qos, retain, node_id);
        self.send_with_callback(message, response_topic, callback).await
    }
    
    /// 发送带分区信息的消息并注册响应回调
    pub async fn send_with_partition_and_callback(
        &self, 
        message: MqMessage, 
        response_topic: String, 
        callback: ResponseCallback
    ) -> Result<()> {
        self.send_with_callback(message, response_topic, callback).await
    }
    
    /// 处理接收到的响应消息
    pub fn handle_response(&self, message: MqMessage) {
        if let Err(e) = self.response_sender.send(message) {
            log::error!("Error sending response to channel: {:?}", e);
        }
    }
    
    /// 添加消费者服务
    pub async fn add_consumer_service(&self, consumer: Box<dyn MqConsumer>) -> Result<Arc<Mutex<MqConsumerService>>> {
        // 创建消费者服务
        let consumer_service = MqConsumerService::new(consumer).await?;
        let consumer_service = Arc::new(Mutex::new(consumer_service));
        
        // 添加到消费者服务列表
        self.consumer_services.lock().unwrap().push(consumer_service.clone());
        
        Ok(consumer_service)
    }
    
    /// 启动所有消费者服务
    pub async fn start_all_consumers(&self) {
        // 克隆消费者服务列表，避免在循环中持有锁
        let consumer_services: Vec<_> = self.consumer_services.lock().unwrap().clone();
        
        for consumer_service in consumer_services.iter() {
            // 启动消费者服务
            // 注意：这里需要在后台启动，否则会阻塞
            let service_clone = consumer_service.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                
                rt.block_on(async move {
                    let mut service = service_clone.lock().unwrap();
                    if let Err(e) = service.start_consuming().await {
                        log::error!("Error starting consumer: {:?}", e);
                    }
                });
            });
        }
    }
    
    /// 关闭所有消费者服务
    pub async fn close_all_consumers(&self) -> Result<()> {
        // 克隆消费者服务列表，避免在循环中持有锁
        let consumer_services: Vec<_> = self.consumer_services.lock().unwrap().clone();
        
        for consumer_service in consumer_services.iter() {
            let mut service = consumer_service.lock().unwrap();
            service.close().await?;
        }
        
        Ok(())
    }
    
    /// 关闭服务
    pub async fn close(&self) -> Result<()> {
        // 关闭所有消费者服务
        self.close_all_consumers().await?;
        
        // 关闭生产者
        let mut producer = self.producer.lock().unwrap();
        producer.disconnect().await
    }
}
