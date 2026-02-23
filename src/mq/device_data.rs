use anyhow::Result;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use crate::mq::message::MqMessage;
use crate::mq::service::MqService;
use crate::mq::topic_resolver::TopicResolver;

/// 设备数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceData {
    /// 设备ID
    pub device_id: String,
    /// 节点ID
    pub node_id: String,
    /// 数据类型
    pub data_type: String,
    /// 数据内容
    pub data: serde_json::Value,
    /// 时间戳
    pub timestamp: u64,
    /// 可选的分区键
    pub partition_key: Option<String>,
}

/// 设备数据上报服务
pub struct DeviceDataService {
    /// MQ服务
    mq_service: MqService,
    /// Topic解析器
    topic_resolver: TopicResolver,
}

impl DeviceDataService {
    /// 创建新的设备数据上报服务
    pub async fn new(mq_service: MqService, topic_pattern: &str) -> Result<Self> {
        let topic_resolver = TopicResolver::new(topic_pattern)?;
        
        Ok(Self {
            mq_service,
            topic_resolver,
        })
    }
    
    /// 上报设备数据
    pub async fn report_device_data(&self, data: DeviceData) -> Result<()> {
        // 生成topic
        let topic = self.topic_resolver.generate_topic(
            &data.node_id,
            &data.device_id,
            None // 分区由MQ系统根据分区键决定
        );
        
        // 序列化数据
        let payload = serde_json::to_vec(&data)?;
        
        // 创建消息
        let mut message = MqMessage::new(
            &topic,
            Bytes::from(payload),
            1, // QoS 1，确保消息至少送达一次
            false,
            &data.node_id
        );
        
        // 设置分区键
        if let Some(partition_key) = data.partition_key {
            message.set_partition_key(Some(partition_key));
        }
        
        // 发送消息到MQ
        // 这里可以根据需要添加响应回调
        let response_topic = format!("{}/response", topic);
        
        self.mq_service.send_with_callback(
            message,
            response_topic,
            Box::new(|response| {
                log::info!("Received response for device data: {:?}", response);
                Ok(())
            })
        ).await
    }
    
    /// 批量上报设备数据
    pub async fn batch_report_device_data(&self, data_list: Vec<DeviceData>) -> Result<()> {
        for data in data_list {
            self.report_device_data(data).await?;
        }
        Ok(())
    }
    
    /// 上报简单的设备数据
    pub async fn report_simple_data(
        &self,
        device_id: &str,
        node_id: &str,
        data_type: &str,
        data: serde_json::Value
    ) -> Result<()> {
        let device_data = DeviceData {
            device_id: device_id.to_string(),
            node_id: node_id.to_string(),
            data_type: data_type.to_string(),
            data,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            partition_key: Some(device_id.to_string()),
        };
        
        self.report_device_data(device_data).await
    }
    
    /// 关闭服务
    pub async fn close(&self) -> Result<()> {
        self.mq_service.close().await
    }
}

/// 设备事件类型
pub enum DeviceEventType {
    /// 遥测数据
    Telemetry,
    /// 状态更新
    StatusUpdate,
    /// 告警
    Alert,
    /// 命令响应
    CommandResponse,
}

impl DeviceEventType {
    /// 获取事件类型的字符串表示
    pub fn as_str(&self) -> &str {
        match self {
            DeviceEventType::Telemetry => "telemetry",
            DeviceEventType::StatusUpdate => "status",
            DeviceEventType::Alert => "alert",
            DeviceEventType::CommandResponse => "response",
        }
    }
    
    /// 获取对应的topic模式
    pub fn get_topic_pattern(&self) -> &str {
        match self {
            DeviceEventType::Telemetry => "{node_id}/{device_id}/telemetry",
            DeviceEventType::StatusUpdate => "{node_id}/{device_id}/status",
            DeviceEventType::Alert => "{node_id}/{device_id}/alert",
            DeviceEventType::CommandResponse => "{node_id}/{device_id}/response",
        }
    }
}
