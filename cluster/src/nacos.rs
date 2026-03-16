use std::sync::Arc;

use anyhow::Result;
use nacos_sdk::api::{
    config::{ConfigService, ConfigServiceBuilder},
    naming::{
        NamingChangeEvent, NamingEventListener, NamingService, NamingServiceBuilder,
        ServiceInstance,
    },
    props::ClientProps,
};

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
}

impl NacosClient {
    /// 创建新的 Nacos 客户端
    pub async fn new(config: NacosConfig, server: ServiceConfig) -> Result<Self> {
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
            .insert("id".to_string(), server.instance_id.to_string());
        naming_service
            .register_instance(server.service_name.clone(), None, service_instance)
            .await?;

        Ok(Self {
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
                Arc::new(ListenerService),
            )
            .await?;
        Ok(())
    }
}

struct ListenerService;
impl NamingEventListener for ListenerService {
    fn event(&self, event: Arc<NamingChangeEvent>) {
        let mut map: std::collections::BTreeMap<u64, ServiceInstance> =
            std::collections::BTreeMap::new();
        if let Some(ref mut instances) = event.instances.clone() {
            for instance in instances.iter() {
                if let Some(id) = instance.metadata.get("id") {
                    map.insert(id.parse::<u64>().unwrap(), instance.clone());
                }
            }
        }
        println!("{:?}", event);
    }
}
