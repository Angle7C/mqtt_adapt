use std::sync::Arc;

use nacos_sdk::api::{
    config::{ConfigChangeListener, ConfigResponse, ConfigServiceBuilder},
    props::ClientProps,
};

pub async fn init_cluster_config(address: &str) -> anyhow::Result<()> {
    let config_service = ConfigServiceBuilder::new(
        ClientProps::new()
            .server_addr(address)
            .namespace("")
            .app_name("TransportConfig")
            .auth_username("")
            .auth_password(""),
    )
    .enable_auth_plugin_http()
    .build()
    .await?;

    config_service
        .add_listener(
            "TransportConfig".to_string(),
            "TransportConfig".to_string(),
            Arc::new(ConfigListener),
        )
        .await?;
    
    Ok(())
}
pub struct ConfigListener;
impl ConfigChangeListener for ConfigListener {
    fn notify(&self, config_resp: ConfigResponse) {
        println!("{:?}", config_resp);
    }
}
