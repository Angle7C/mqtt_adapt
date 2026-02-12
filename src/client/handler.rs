use anyhow::Result;
use bytes::BytesMut;
use flume::RecvError;
use log::{info, error};
use tokio::time::Duration;

use crate::client::client::Client;
use crate::protocol::{DisconnectPacket, Packet};
use crate::routing::event::Event;

impl Client {
    /// 处理客户端连接
    pub async fn handle(mut self) -> Result<()> {
        loop {
            // 计算超时时间：keepalive的1.5倍
            let timeout_duration = Duration::from_secs((self.keepalive as f32 * 1.5) as u64);
            let rx = self.rx.clone();
            
            tokio::select! {
                // 1. 读取来自TCP连接的消息
                read_result = self.read() => {
                    info!("read buffer");
                    self.handle_read_result(read_result).await?;
                },

                // 2. 接收来自消息路由的消息
                event_result = rx.recv_async() => {
                    info!("Received event from router: {:?}", event_result);
                    self.handle_event_result(event_result).await?;
                },

                // 3. 超时处理
                _ = tokio::time::sleep(timeout_duration) => {
                    info!("Timeout after {} seconds", timeout_duration.as_secs());
                    self.handle_timeout(timeout_duration).await?;
                    break;
                },
            }
        }

        Ok(())
    }
    
    /// 处理读取结果
    async fn handle_read_result(&mut self, result: Result<usize>) -> Result<()> {
        match result {
            Ok(n) => {
                if n == 0 {
                    // 连接关闭
                    self.state = super::client::ClientState::Disconnected;
                    return Err(anyhow::anyhow!("Connection closed by client"));
                }

                // 这里可以添加消息解析逻辑
                // 暂时简单处理
                info!("Received {} bytes from client", n);
            }
            Err(e) => {
                error!("Error reading from client: {:?}", e);
                self.state = super::client::ClientState::Disconnected;
                return Err(e.into());
            }
        }
        Ok(())
    }
    
    /// 处理事件结果
    async fn handle_event_result(&mut self, result: Result<Event, RecvError>) -> Result<()> {
        match result {
            Ok(event) => {
                info!("Received event from router: {:?}", event);
                self.handle_router_event(event).await?;
            }
            Err(_) => {
                // 通道关闭，退出循环
                self.state = super::client::ClientState::Disconnected;
                return Err(anyhow::anyhow!("Channel closed"));
            }
        }
        Ok(())
    }
    
    /// 处理超时
    async fn handle_timeout(&mut self, timeout_duration: Duration) -> Result<()> {
        // 超时处理
        info!("Client timeout: no activity for {} seconds", timeout_duration.as_secs());
        
        // 发送断开连接数据包
        self.send_disconnect_packet().await?;
        
        // 通知客户端断开连接
        self.notify_disconnection().await?;
        
        // 更新客户端状态
        self.state = super::client::ClientState::Disconnected;
        
        Ok(())
    }
    
    /// 发送断开连接数据包
    async fn send_disconnect_packet(&mut self) -> Result<()> {
        // 创建Disconnect数据包
        let disconnect_packet = DisconnectPacket;
        
        // 写入到缓冲区
        let mut buf = BytesMut::new();
        disconnect_packet.write(&mut buf);
        self.write_buf.extend_from_slice(&buf);
        
        // 发送数据包
        self.write().await?;
        
        Ok(())
    }
    
    /// 通知客户端断开连接
    async fn notify_disconnection(&mut self) -> Result<()> {
        if let Some(client_id) = &self.client_id {
            let event = Event::ClientDisconnected(client_id.clone());
            self.send_event(event)?;
        }
        Ok(())
    }
    
    /// 处理来自router的事件
    pub async fn handle_router_event(&mut self, event: Event) -> Result<()> {
        use crate::protocol::Packet;

        match event {
            Event::MessageSent(_, packet) => {
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
                        info!("Other packet type: {:?}", packet);
                    }
                }

                // 发送数据包
                self.write().await?;
            }
            Event::ClientConnected(client_id) => {
                info!("Client connected: {}", client_id);
            }
            Event::ClientDisconnected(client_id) => {
                info!("Client disconnected: {}", client_id);
            }
            Event::MessageReceived(client_id, packet) => {
                info!("Message received from {}: {:?}", client_id, packet);
            }
            Event::BroadcastMessage(packet) => {
                info!("Broadcast message: {:?}", packet);
            }
        }

        Ok(())
    }
}
