use crate::{client::Client, routing::router::MessageRouter};
use log::{error, info};
use std::{net::{SocketAddr, ToSocketAddrs}, os::windows::io::AsRawSocket, thread::{self, Thread}};
use tokio::net::{TcpListener, TcpSocket};

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

        Self { addr, router }
    }

    /// 启动服务器
    pub async fn start(&self) {
        // 启动路由器
        let router_clone = self.router.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                router_clone.start().await;
            });
        });

        // 绑定TCP监听器
        let listener = TcpListener::bind(&self.addr)
            .await
            .expect("Failed to bind address");

        info!("MQTT server started on {}", self.addr);

        // 处理客户端连接
        while let Ok((socket, addr)) = listener.accept().await {
            info!("Accepted connection from {}", addr);
            // 克隆路由器引用
            let router_clone = self.router.clone();
            socket.set_nodelay(true).expect("close Nagle算法");
            // 处理客户端连接
            tokio::spawn(async move {
                if let Ok(client) =
                    crate::client::create_client_with_connect(socket, addr, &router_clone).await
                {
                    if let Err(e) = client.handle().await {
                        error!("Error handling client: {:?}", e);
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
