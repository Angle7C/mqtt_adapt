use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// 集群节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    /// 节点ID
    pub node_id: String,
    /// 节点地址
    pub address: SocketAddr,
    /// 是否为主节点
    pub is_leader: bool,
    /// 节点状态
    pub status: String,
}

/// 集群配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// 集群名称
    pub cluster_name: String,
    /// 当前节点ID
    pub current_node_id: String,
    /// 所有节点
    pub nodes: Vec<ClusterNode>,
    /// 选举超时时间（毫秒）
    pub election_timeout: u64,
    /// 心跳间隔（毫秒）
    pub heartbeat_interval: u64,
}

impl ClusterConfig {
    /// 创建默认集群配置
    pub fn default() -> Self {
        Self {
            cluster_name: "mqtt-cluster".to_string(),
            current_node_id: format!("node-{}", uuid::Uuid::new_v4()),
            nodes: Vec::new(),
            election_timeout: 3000,
            heartbeat_interval: 1000,
        }
    }
    
    /// 添加节点
    pub fn add_node(&mut self, node: ClusterNode) {
        self.nodes.push(node);
    }
    
    /// 获取主节点
    pub fn get_leader(&self) -> Option<&ClusterNode> {
        self.nodes.iter().find(|node| node.is_leader)
    }
    
    /// 获取当前节点
    pub fn get_current_node(&self) -> Option<&ClusterNode> {
        self.nodes.iter().find(|node| node.node_id == self.current_node_id)
    }
}
