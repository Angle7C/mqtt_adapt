
use anyhow::Result;
use flume::unbounded;
use tokio::net::TcpStream;

use crate::client::client::Client;
use crate::db::connection::DatabaseConnection;
use crate::db::models::ag_user::User;
use crate::protocol::MqttPacket;
use crate::protocol::{ConnAckPacket, ConnectReturnCode};
use crate::routing::router::MessageRouter;
/// 从TCP流创建客户端并处理CONNECT数据包
///
/// 1. 创建客户端事件通道
/// 2. 读取并解析CONNECT数据包
/// 3. 设置客户端ID和保活时间
/// 4. 验证用户凭据
/// 5. 注册客户端到路由器
/// 6. 发送客户端连接事件
pub async fn create_client_with_connect(
    socket: TcpStream,
    addr: std::net::SocketAddr,
    router: &MessageRouter,
    db: &DatabaseConnection,
) -> Result<Client> {
    // 创建客户端事件通道
    let (tx, rx) = unbounded();
    //获取router的event_sender
    let router_event_sender = router.get_sender().clone();
    // 创建初始客户端实例
    let mut client = Client::new(socket,addr, rx, router_event_sender, "unknown".to_string());

    // 读取并解析CONNECT数据包

    // 读取数据到缓冲区
    let n = client.read().await?;
    if n == 0 {
        return Err(anyhow::format_err!("ConnectionPacket is empty"));
    }
    let packet = MqttPacket::read(&mut client.read_buf)?;

    if let MqttPacket::Connect(connect_packet) = packet {
        // 克隆客户端ID
        let client_id = connect_packet.client_id;

        // 设置客户端ID
        client.set_client_id(client_id.clone());

        // 设置保活时间
        client.set_keepalive(connect_packet.keep_alive);

        // 设置遗嘱消息相关信息
        client.will_topic = connect_packet.will_topic;
        client.will_message = connect_packet.will_message;
        
        // 从connect_flags中提取遗嘱QoS和保留标志
        // 遗嘱QoS: 第3-4位
        client.will_qos = (connect_packet.connect_flags >> 3) & 0x03;
        // 遗嘱保留标志: 第5位
        client.will_retain = (connect_packet.connect_flags & 0x20) != 0;

        // 验证用户凭据
        let return_code = match (connect_packet.username, connect_packet.password) {
            (Some(username), Some(password)) => {
                // 从密码Bytes转换为字符串
                let password_str = String::from_utf8_lossy(&password).to_string();
                
                // 使用用户名作为access_key进行数据库认证
                match User::find_by_username(db.get_pool(), &username).await {
                    Ok(Some(user)) => {
                        // 验证密码是否匹配
                        if user.password == password_str {
                            ConnectReturnCode::Accepted
                        } else {
                            ConnectReturnCode::RefusedBadUsernameOrPassword
                        }
                    },
                    _ => {
                        // 用户不存在或查询失败
                        ConnectReturnCode::RefusedBadUsernameOrPassword
                    }
                }
            },
            _ => {
                // 用户名和密码必须同时提供
                ConnectReturnCode::RefusedBadUsernameOrPassword
            },
        };
        
        // 创建CONNACK数据包
        let connack_packet = ConnAckPacket {
            session_present: false,
            return_code,
        };
        
        let mqtt_packet = MqttPacket::ConnAck(connack_packet);
        mqtt_packet.write(&mut client.write_buf);
        // 发送数据包
        client.write().await?;
        
        // 只有认证成功才注册客户端
        if return_code == ConnectReturnCode::Accepted {
            // 注册客户端到路由器
            router
                .register_client(&client_id, tx.clone())
                .await?;
        } else {
            // 认证失败，关闭连接
            return Err(anyhow::format_err!("Authentication failed"));
        }
    } else {
        return Err(anyhow::format_err!("Expected CONNECT packet"));
    }
    Ok(client)
}
