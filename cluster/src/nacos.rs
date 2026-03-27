use std::{cmp::Ordering, sync::Arc};

use anyhow::Result;
use nacos_sdk::api::{
    config::{ConfigService, ConfigServiceBuilder},
    naming::{
        NamingChangeEvent, NamingEventListener, NamingService, NamingServiceBuilder,
        ServiceInstance,
    },
    props::ClientProps,
};
// 一个常驻变量，用于26个字母的随机数
const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890-";
/// Nacos 客户端配置
#[derive(Debug, Clone)]
pub struct NacosConfig {
    /// Nacos 服务器地址
    pub server_addr: String,
    /// 命名空间
    pub namespace: Option<String>,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
}
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Nacos 客户端
    id: uuid::Uuid,
    /// 服务名称
    service_name: String,
    /// 实例ID
    instance_id: u64,
}

/// 分区信息结构体
#[derive(Debug, Clone)]
pub struct PartitionInfo {
    /// 分区ID
    pub partition_id: u64,
    /// 分区名称
    pub partition_name: String,
    /// 分区状态
    pub partition_type: PartitionType,
}

/// 分区状态枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitionType {
    Core,
    RuleEngine,
    Actor,
}

/// Nacos 客户端
pub struct NacosClient {
    config_service: ConfigService,
    naming_service: NamingService,
    nodeId: u64,
}

impl NacosClient {
    /// 创建新的 Nacos 客户端
    pub async fn new(config: NacosConfig, server: ServiceConfig) -> Result<Self> {
        let node_id = rand::random::<u64>();
        let config_service = ConfigServiceBuilder::new(
            ClientProps::new()
                .server_addr(config.server_addr.clone())
                .namespace(config.namespace.clone().unwrap_or_else(|| "".to_string()))
                .auth_username(config.username.clone().unwrap_or_else(|| "".to_string()))
                .auth_password(config.password.clone().unwrap_or_else(|| "".to_string())),
        )
        .enable_auth_plugin_http()
        .build()
        .await?;
        let naming_service = NamingServiceBuilder::new(
            ClientProps::new()
                .server_addr(config.server_addr.clone())
                .namespace(config.namespace.clone().unwrap_or_else(|| "".to_string()))
                .auth_username(config.username.clone().unwrap_or_else(|| "".to_string()))
                .auth_password(config.password.clone().unwrap_or_else(|| "".to_string())),
        )
        .enable_auth_plugin_http()
        .build()
        .await?;
        let mut service_instance = ServiceInstance::default();
        service_instance
            .metadata
            .insert("NodeId".to_string(), node_id.to_string());

        naming_service
            .register_instance(server.service_name.clone(), None, service_instance)
            .await?;

        Ok(Self {
            nodeId: node_id,
            config_service,
            naming_service,
        })
    }

    pub async fn listen(&self, service_name: &str) -> Result<()> {
        self.naming_service
            .subscribe(
                service_name.to_string(),
                None,
                Vec::new(),
                Arc::new(ListenerService {
                    nodeId: self.nodeId,
                }),
            )
            .await?;
        Ok(())
    }
}

struct ListenerService {
    nodeId: u64,
}
impl NamingEventListener for ListenerService {
    fn event(&self, event: Arc<NamingChangeEvent>) {
        if let Some(ref instances) = event.instances {
            let mut nodeIds = instances
                .iter()
                .map(|instance| instance.metadata.get("NodeId").unwrap())
                .map(|id| id.parse::<u64>().unwrap())
                .collect::<Vec<_>>();
            nodeIds.sort();

            let alphabet_len = ALPHABET.len();
            let sub_size = alphabet_len / nodeIds.len();

            for (index, node_id) in nodeIds.iter().enumerate() {
                //
                let start_index = index * sub_size;
                let end_index = (index + 1) * sub_size;
                println!("{}", ALPHABET[start_index..end_index].to_string());
                if *node_id == self.nodeId {
                    // 打印当前节点的分区
                    println!("当前节点的分区: {}", ALPHABET[start_index..end_index].to_string());
                };
                // 打印其他节点的分区
                println!("其他节点的分区: {}", ALPHABET[start_index..end_index].to_string());
            }
        }

        println!("{:?}", event);
    }
}
