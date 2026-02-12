use anyhow::Result;
use bytes::{Buf, BytesMut};
use flume::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::routing::event::Event;

/// 客户端连接状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientState {
    /// 未连接
    Disconnected,
    /// 已连接
    Connected,
}

/// 客户端结构体
#[derive(Debug)]
pub struct Client {
    /// 连接状态
    state: ClientState,
    /// 客户端ID
    client_id: Option<String>,
    /// 读取缓冲区
    read_buf: BytesMut,
    /// 写入缓冲区
    write_buf: BytesMut,
    /// 底层TCP连接
    socket: Box<TcpStream>,
    /// 消息接收通道
    rx: Receiver<Event>,
}

impl Client {
    /// 创建新的客户端
    pub fn new(socket: TcpStream, rx: Receiver<Event>) -> Self {
        Self {
            state: ClientState::Connected,
            client_id: None,
            read_buf: BytesMut::new(),
            write_buf: BytesMut::new(),
            socket: Box::new(socket),
            rx,
        }
    }

    /// 处理客户端连接
    pub async fn handle(mut self) -> Result<()> {
        loop {
            // 创建一个缓冲区
            let mut buf = [0; 4096];

            tokio::select! {
                // 读取来自TCP连接的消息
                result = self.socket.read(&mut buf) => {
                    match result {
                        Ok(n) => {
                            if n == 0 {
                                // 连接关闭
                                self.state = ClientState::Disconnected;
                                break;
                            }

                            // 这里可以添加消息解析逻辑
                            // 暂时简单处理
                            println!("Received {} bytes from client", n);
                        }
                        Err(e) => {
                            println!("Error reading from client: {:?}", e);
                            break;
                        }
                    }
                }

                // 接收来自消息路由的消息
                result = self.rx.recv_async() => {
                    match result {
                        Ok(event) => {
                            println!("Received event from router: {:?}", event);

                            // 处理router事件
                            if let Err(e) = self.handle_router_event(event).await {
                                println!("Error handling router event: {:?}", e);
                                break;
                            }
                        }
                        Err(_) => {
                            // 通道关闭，退出循环
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 写入数据到客户端
    pub async fn write(&mut self) -> Result<()> {
        if !self.write_buf.is_empty() {
            self.socket.write_all(&self.write_buf).await?;
            self.write_buf.clear();
        }
        Ok(())
    }

    pub async fn read(&mut self) -> Result<usize> {
        let n = self.socket.read_buf(&mut self.read_buf).await?;
        Ok(n)
    }

    /// 设置客户端ID
    pub fn set_client_id(&mut self, client_id: String) {
        self.client_id = Some(client_id);
    }

    /// 获取客户端ID
    pub fn client_id(&self) -> Option<&String> {
        self.client_id.as_ref()
    }

    /// 获取客户端状态
    pub fn state(&self) -> &ClientState {
        &self.state
    }

    /// 处理来自router的事件
    pub async fn handle_router_event(&mut self, event: Event) -> Result<()> {
        use crate::protocol::Packet;

        match event {
            Event::MessageSent(_, packet) => {
                // 将数据包序列化为字节
                let mut buf = bytes::BytesMut::new();

                // 使用Packet trait的write方法
                match packet {
                    crate::protocol::MqttPacket::ConnAck(connack) => {
                        connack.write(&mut self.write_buf);
                    }
                    crate::protocol::MqttPacket::Publish(publish) => {
                        publish.write(&mut self.write_buf);
                    }
                    crate::protocol::MqttPacket::PubAck(puback) => {
                        puback.write(&mut self.write_buf);
                    }
                    crate::protocol::MqttPacket::PubRec(pubrec) => {
                        pubrec.write(&mut self.write_buf);
                    }
                    crate::protocol::MqttPacket::PubRel(pubrel) => {
                        pubrel.write(&mut self.write_buf);
                    }
                    crate::protocol::MqttPacket::PubComp(pubcomp) => {
                        pubcomp.write(&mut self.write_buf);
                    }
                    crate::protocol::MqttPacket::Subscribe(subscribe) => {
                        subscribe.write(&mut self.write_buf);
                    }
                    crate::protocol::MqttPacket::SubAck(suback) => {
                        suback.write(&mut self.write_buf);
                    }
                    crate::protocol::MqttPacket::Unsubscribe(unsubscribe) => {
                        unsubscribe.write(&mut self.write_buf);
                    }
                    crate::protocol::MqttPacket::UnsubAck(unsuback) => {
                        unsuback.write(&mut self.write_buf);
                    }
                    crate::protocol::MqttPacket::PingReq(pingreq) => {
                        pingreq.write(&mut self.write_buf);
                    }
                    crate::protocol::MqttPacket::PingResp(pingresp) => {
                        pingresp.write(&mut self.write_buf);
                    }
                    crate::protocol::MqttPacket::Disconnect(disconnect) => {
                        disconnect.write(&mut self.write_buf);
                    }
                    _ => {
                        // 处理其他类型的数据包
                        println!("Other packet type: {:?}", packet);
                    }
                }

                self.write().await?;
            }
            Event::ClientConnected(client_id) => {
                println!("Client connected: {}", client_id);
            }
            Event::ClientDisconnected(client_id) => {
                println!("Client disconnected: {}", client_id);
            }
            Event::MessageReceived(client_id, packet) => {
                println!("Message received from {}: {:?}", client_id, packet);
            }
            Event::BroadcastMessage(packet) => {
                println!("Broadcast message: {:?}", packet);
            }
        }

        Ok(())
    }
}

/// 从TCP流创建客户端并处理CONNECT数据包
pub async fn create_client_with_connect(
    socket: TcpStream,
    router: &crate::routing::router::MessageRouter,
) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
    use crate::protocol::MqttPacket;
    use crate::routing::event::Event;
    use flume::unbounded;

    // 创建客户端事件通道
    let (tx, rx) = unbounded();

    // 创建初始客户端实例
    let mut client = Client::new(socket, rx);

    // 读取并解析CONNECT数据包
    let mut read_buf = bytes::BytesMut::with_capacity(1024 * 10);

    // 读取数据到缓冲区
    let n = client.read().await?;
    if n == 0 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "Connection closed by client",
        )));
    }

    // 尝试解析MQTT数据包
    match MqttPacket::read(&mut client.read_buf) {
        Ok(packet) => {
            if let MqttPacket::Connect(connect_packet) = packet {
                // 克隆客户端ID
                let client_id = connect_packet.client_id.clone();

                // 设置客户端ID
                client.set_client_id(client_id.clone());

                // 注册客户端到路由器
                router.register_client(client_id.clone(), tx);

                // 发送客户端连接事件
                let event = Event::ClientConnected(client_id.clone());
                if let Err(e) = router.send_event(event) {
                    println!("Error sending client connected event: {:?}", e);
                }

                println!("Client connected with ID: {}", client_id);
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Expected CONNECT packet",
                )));
            }
        }
        Err(e) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse CONNECT packet: {}", e),
            )));
        }
    }

    Ok(client)
}
