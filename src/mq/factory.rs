use anyhow::Result;
use crate::mq::producer::MqProducer;

/// MQ客户端配置
#[derive(Debug, Clone)]
pub struct MqClientConfig {
    /// 服务器地址
    pub broker_url: String,
    /// 客户端ID
    pub client_id: String,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 连接超时（秒）
    pub connection_timeout: u64,
    /// 保持连接（秒）
    pub keep_alive: u64,
}

impl Default for MqClientConfig {
    fn default() -> Self {
        Self {
            broker_url: "localhost:1883".to_string(),
            client_id: format!("mq-client-{}", uuid::Uuid::new_v4()),
            username: None,
            password: None,
            connection_timeout: 30,
            keep_alive: 60,
        }
    }
}

/// MQ客户端工厂trait
pub trait MqClientFactory {
    /// 创建MQ客户端
    async fn create_client(&self, config: MqClientConfig) -> Result<Box<dyn MqProducer>>;
}
