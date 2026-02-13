use crate::{ClinetId, protocol::MqttPacket};

#[derive(Debug)]
pub enum Event {
    /// 客户端连接事件
    ClientConnected(ClinetId),
    /// 客户端断开连接事件
    ClientDisconnected(ClinetId),
    /// 消息接收事件
    MessageReceived(ClinetId, MqttPacket),
    /// 消息发送事件
    MessageSent(ClinetId, MqttPacket),
    /// 广播消息事件
    BroadcastMessage(MqttPacket),
}
