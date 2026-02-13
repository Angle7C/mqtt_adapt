use anyhow::Result;
use bytes::BytesMut;
use flume::RecvError;
use log::{error, info};
use tokio::time::Duration;

use crate::client::client::Client;
use crate::protocol::{DisconnectPacket, MqttPacket, Packet};
use crate::routing::event::Event;

impl Client {
    /// 处理客户端连接
    pub async fn handle(mut self) -> Result<()> {
        // 计算超时时间：keepalive的1.5倍
        // 如果keepalive为0，则使用默认值60秒
        let keepalive = if self.keepalive == 0 { 60 } else { self.keepalive };
        let time = (keepalive as f32 * 1.5) as u64;

        let timeout_duration = Duration::from_secs(time);
        let rx = self.event_receiver.clone();
        loop {
            tokio::select! {
                // 1. 读取来自TCP连接的消息
                read_result = self.read() => {
                    self.handle_read_result(read_result).await?;
                },

                // 2. 接收来自消息路由的消息
                event_result =rx.recv_async() => {
                    self.handle_event_result(event_result).await?;
                },

                // 3. 超时处理
                _ = tokio::time::sleep(timeout_duration) => {
                    self.close().await?;
                },
            }

            if self.state == super::client::ClientState::Disconnected {
                drop(self);
                break;
            }
        }
        Ok(())
    }

    /// 处理读取结果
    async fn handle_read_result(&mut self, result: Result<usize>) -> Result<()> {
        match result {
            Ok(n) if n <= 0 => {
                self.close().await?;
                return Ok(());
            }
            Ok(_) => {
                // 处理读取到的数据
                let packet = MqttPacket::read(&mut self.read_buf)?;

                // 对于某些只是用来保持连接的包，直接处理而不发送到路由
                match &packet {
                    MqttPacket::PingReq(_) => {
                        // 直接回复PingResp
                        self.handle_ping_req().await?;
                    }
                    MqttPacket::Disconnect(_) => {
                        // 直接关闭连接
                        self.close().await?;
                    }
                    _ => {
                        // 其他包发送到路由中
                        let event = Event::MessageReceived(self.client_id.clone(), packet);
                        self.send_event(event)?;
                    }
                }
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
    async fn close(&mut self) -> Result<()> {
        // if self.socket.writable().await.is_ok() {
        //     // 发送断开连接数据包
        //     self.send_disconnect_packet().await?;
        // }

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
        disconnect_packet.write(&mut self.write_buf);
        // 发送数据包
        self.write().await?;

        Ok(())
    }

    /// 处理PingReq数据包
    async fn handle_ping_req(&mut self) -> Result<()> {
        // 创建PingResp数据包
        use crate::protocol::PingRespPacket;

        // 写入到缓冲区
        PingRespPacket.write(&mut self.write_buf);

        // 发送数据包
        self.write().await?;

        Ok(())
    }

    /// 通知客户端断开连接
    async fn notify_disconnection(&mut self) -> Result<()> {
        let event = Event::ClientDisconnected(self.client_id.clone());
        self.send_event(event)
        // Ok(())
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
                // info!("Message received from {}: {:?}", client_id, packet);
            }
            Event::BroadcastMessage(packet) => {
                info!("Broadcast message: {:?}", packet);
            }
        }

        Ok(())
    }
}
