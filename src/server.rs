use crate::{client::Client, routing::router::MessageRouter};
use log::{error, info};
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// MQTT服务器结构体
#[derive(Debug, Clone)]
pub struct Server {
    /// 服务器地址
    addr: SocketAddr,
    /// 消息路由器
    router: MessageRouter,
}

impl Server {
    /// 创建新的MQTT服务器
    pub fn new(addr: SocketAddr) -> Self {
        // 创建路由器
        let router = MessageRouter::new();
        
        Self {
            addr,
            router,
        }
    }

    /// 启动服务器
    pub async fn start(&self) {
        // 启动路由器
        let router_clone = self.router.clone();
        tokio::spawn(async move {
            router_clone.start().await;
        });

        // 绑定TCP监听器
        let listener = TcpListener::bind(&self.addr)
            .await
            .expect("Failed to bind address");      

        info!("MQTT server started on {}", self.addr);

        // 处理客户端连接
        while let Ok((socket, _)) = listener.accept().await {
            // 克隆路由器引用
            let router_clone = self.router.clone();
            
            // 处理客户端连接
            tokio::spawn(async move {
                match crate::client::create_client_with_connect(socket, &router_clone).await {
                    Ok(client) => {
                        // 处理客户端连接
                        if let Err(e) = client.handle().await {
                            error!("Error handling client: {:?}", e);
                        }
                    },
                    Err(e) => {
                        error!("Error creating client: {:?}", e);
                    }
                }
            });
        }
    }

    /// 获取路由器实例
    pub fn router(&self) -> &MessageRouter {
        &self.router
    }
}
