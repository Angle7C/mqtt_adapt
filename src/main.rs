
use mqtt_adapt::server::Server;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // 服务器地址
    let addr: SocketAddr = "127.0.0.1:1883".parse().unwrap();
    
    // 创建服务器
    let server = Server::new(addr);
    
    // 启动服务器
    server.start().await;
}
