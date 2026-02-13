use std::hash::RandomState;

use anyhow::Result;
use bytes::BytesMut;
use flume::unbounded;
use log::{error, info};
use tokio::net::TcpStream;

use crate::client::client::Client;
use crate::protocol::MqttPacket;
use crate::protocol::{ConnAckPacket, ConnectReturnCode};
use crate::routing::router::MessageRouter;
/// 从TCP流创建客户端并处理CONNECT数据包
///
/// 1. 创建客户端事件通道
/// 2. 读取并解析CONNECT数据包
/// 3. 设置客户端ID和保活时间
/// 4. 注册客户端到路由器
/// 5. 发送客户端连接事件
pub async fn create_client_with_connect(
    socket: TcpStream,
    router: &MessageRouter,
) -> Result<Client> {
    // 创建客户端事件通道
    let (tx, rx) = unbounded();
    //获取router的event_sender
    let router_event_sender = router.get_sender().clone();
    // 创建初始客户端实例
    let mut client = Client::new(socket, rx, router_event_sender, "unknown".to_string());

    // 读取并解析CONNECT数据包

    // 读取数据到缓冲区
    let n = client.read().await?;
    if n == 0 {
        return Err(anyhow::format_err!("ConnectionPacket is empty"));
    }
    let packet = MqttPacket::read(&mut client.read_buf)?;

    if let MqttPacket::Connect(connect_packet) = packet {
        // 克隆客户端ID
        let client_id = connect_packet.client_id.clone();

        // 设置客户端ID
        client.set_client_id(client_id.clone());

        // 设置保活时间
        client.set_keepalive(connect_packet.keep_alive);

        let connack_packet = ConnAckPacket {
            session_present: false,
            return_code: ConnectReturnCode::Accepted,
        };
        let mqtt_packet = MqttPacket::ConnAck(connack_packet);

        mqtt_packet.write(&mut client.write_buf);
        // 发送数据包
        client.write().await?;

        // 注册客户端到路由器
        router
            .register_client(client_id.clone(), tx.clone())
            .await?;
    } else {
        return Err(anyhow::format_err!("Expected CONNECT packet"));
    }
    Ok(client)
}
