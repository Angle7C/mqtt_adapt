use crate::db::connection::DatabaseConnection;
use crate::routing::router::MessageRouter;
use log::{error, info};
use std::{net::SocketAddr, thread::{self}};
use tokio::net::TcpListener;

/// MQTT服务器结构体
#[derive(Debug, Clone)]
pub struct Server {
    /// 服务器地址
    addr: SocketAddr,
    /// 消息路由器
    router: MessageRouter,
    /// 数据库连接
    db: Option<DatabaseConnection>,
}

impl Server {
    /// 创建新的MQTT服务器
    pub fn new(addr: SocketAddr) -> Self {
        // 创建路由器
        let router = MessageRouter::new();

        Self { addr, router, db: None }
    }
    
    /// 设置数据库连接
    pub fn with_database(mut self, db: DatabaseConnection) -> Self {
        self.db = Some(db);
        self
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
            // 克隆路由器和数据库连接引用
            let router_clone = self.router.clone();
            let db_clone = self.db.clone();
            socket.set_nodelay(true).expect("close Nagle算法");
            // 处理客户端连接
            tokio::spawn(async move {
                if let Some(db) = db_clone {
                    if let Ok(client) =
                        crate::client::create_client_with_connect(socket, addr, &router_clone, &db).await
                    {
                        if let Err(e) = client.handle().await {
                            error!("Error handling client: {:?}", e);
                        }
                    }
                } else {
                    error!("Database connection not available for client: {}", addr);
                    // 这里可以添加代码来关闭连接或发送拒绝消息
                }
            });
        }
    }

    /// 获取路由器实例
    pub fn router(&self) -> &MessageRouter {
        

        &self.router
    }


}
