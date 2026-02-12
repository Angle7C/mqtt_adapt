use mqtt_adapt::server::Server;
use std::net::SocketAddr;
use tracing::Level;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    // 初始化tracing日志系统
    init_tracing();

    // 服务器地址
    let addr: SocketAddr = "127.0.0.1:1883".parse().unwrap();

    // 创建服务器
    let server = Server::new(addr);

    // 启动服务器
    server.start().await;
}
fn init_tracing() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
}
