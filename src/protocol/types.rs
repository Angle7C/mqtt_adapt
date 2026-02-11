use bytes::Bytes;

/// MQTT控制报文类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    Connect = 1,
    ConnAck = 2,
    Publish = 3,
    PubAck = 4,
    PubRec = 5,
    PubRel = 6,
    PubComp = 7,
    Subscribe = 8,
    SubAck = 9,
    Unsubscribe = 10,
    UnsubAck = 11,
    PingReq = 12,
    PingResp = 13,
    Disconnect = 14,
}

impl PacketType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Connect),
            2 => Some(Self::ConnAck),
            3 => Some(Self::Publish),
            4 => Some(Self::PubAck),
            5 => Some(Self::PubRec),
            6 => Some(Self::PubRel),
            7 => Some(Self::PubComp),
            8 => Some(Self::Subscribe),
            9 => Some(Self::SubAck),
            10 => Some(Self::Unsubscribe),
            11 => Some(Self::UnsubAck),
            12 => Some(Self::PingReq),
            13 => Some(Self::PingResp),
            14 => Some(Self::Disconnect),
            _ => None,
        }
    }
}

/// MQTT固定头
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedHeader {
    pub packet_type: PacketType,
    pub flags: u8,
    pub remaining_length: usize,
}

/// MQTT连接返回码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectReturnCode {
    Accepted = 0,
    RefusedBadProtocolVersion = 1,
    RefusedIdentifierRejected = 2,
    RefusedServerUnavailable = 3,
    RefusedBadUsernameOrPassword = 4,
    RefusedNotAuthorized = 5,
}

impl ConnectReturnCode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Accepted),
            1 => Some(Self::RefusedBadProtocolVersion),
            2 => Some(Self::RefusedIdentifierRejected),
            3 => Some(Self::RefusedServerUnavailable),
            4 => Some(Self::RefusedBadUsernameOrPassword),
            5 => Some(Self::RefusedNotAuthorized),
            _ => None,
        }
    }
}

/// MQTT数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MqttPacket {
    Connect(super::connect::ConnectPacket),
    ConnAck(super::connack::ConnAckPacket),
    Publish(super::publish::PublishPacket),
    PubAck(super::puback::PubAckPacket),
    PubRec(super::pubrec::PubRecPacket),
    PubRel(super::pubrel::PubRelPacket),
    PubComp(super::pubcomp::PubCompPacket),
    Subscribe(super::subscribe::SubscribePacket),
    SubAck(super::suback::SubAckPacket),
    Unsubscribe(super::unsubscribe::UnsubscribePacket),
    UnsubAck(super::unsuback::UnsubAckPacket),
    PingReq,
    PingResp,
    Disconnect,
}
