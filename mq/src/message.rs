use bytes::Bytes;

use crate::NodeId;

/// MQ消息结构
#[derive(Debug, Clone)]
pub struct MqMessage {
    /// 消息主题
    pub topic: String,
    /// 消息内容
    pub payload: Bytes,
    /// 节点ID
    pub node_id: NodeId,
    /// 分区ID（可选）
    pub partition: Option<i32>,
    /// 分区键（可选）
    pub partition_key: Option<String>,
}

impl MqMessage {
    /// 创建新的MQ消息
    pub fn new(topic: impl Into<String>, payload: impl Into<Bytes>, 
         node_id: impl Into<NodeId>) -> Self {
        Self {
            topic: topic.into(),
            payload: payload.into(),
            node_id: node_id.into(),
            partition: None,
            partition_key: None,
        }
    }
    
    /// 创建带分区信息的MQ消息
    pub fn with_partition(
        topic: impl Into<String>, 
        payload: impl Into<Bytes>, 
        node_id: impl Into<NodeId>,
        partition: Option<i32>,
        partition_key: Option<impl Into<String>>
    ) -> Self {
        Self {
            topic: topic.into(),
            payload: payload.into(),
            node_id: node_id.into(),
            partition,
            partition_key: partition_key.map(|k| k.into()),
        }
    }
    
    /// 创建默认QoS为0的消息
    pub fn with_default_qos(topic: impl Into<String>, payload: impl Into<Bytes>, 
        node_id: impl Into<NodeId>) -> Self {
        Self::new(topic, payload, node_id)
    }
    
    /// 设置分区ID
    pub fn set_partition(&mut self, partition: Option<i32>) {
        self.partition = partition;
    }
    
    /// 设置分区键
    pub fn set_partition_key(&mut self, partition_key: Option<impl Into<String>>) {
        self.partition_key = partition_key.map(|k| k.into());
    }
}
