use bytes::{Buf, Bytes, BytesMut};

pub mod connect;
pub mod connack;
pub mod publish;
pub mod puback;
pub mod pubrec;
pub mod pubrel;
pub mod pubcomp;
pub mod subscribe;
pub mod suback;
pub mod unsubscribe;
pub mod unsuback;

/// MQTT固定头结构
/// MQTT固定头
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedHeader {
    pub packet_type: PacketType,
    pub flags: u8,
    pub remaining_length: usize,
}

impl FixedHeader {
    /// 解析MQTT固定头
    pub fn parse(input: &mut BytesMut) -> Result<Self, String> {
        // 至少需要1字节来读取消息类型和标志位
        if input.len() < 1 {
            return Err("Insufficient data for fixed header".to_string());
        }
        
        // 读取第一个字节：高4位是消息类型，低4位是标志位
        let first_byte = input.get_u8();
        let packet_type_value = (first_byte >> 4) & 0x0F;
        let flags = first_byte & 0x0F;
    
        
        // 解析剩余长度
        let mut remaining_length = 0;
        let mut multiplier = 1;
        let mut bytes_read = 0;
        
        loop {
            if input.is_empty() {
                return Err("Insufficient data for remaining length".to_string());
            }
            
            let byte = input.get_u8();
            
            remaining_length += ((byte & 0x7F) as u32) * multiplier;
            bytes_read += 1;
            
            // 检查是否有更多字节（最高位为1表示后续还有字节）
            if (byte & 0x80) == 0 {
                break;
            }
            
            // 检查剩余长度是否超过4字节（MQTT协议限制）
            if bytes_read > 3 {
                return Err("Invalid remaining length: more than 4 bytes".to_string());
            }
            
            // 更新乘数（每次乘以128）
            multiplier *= 128;
            
            // 检查是否溢出
            if multiplier > 128 * 128 * 128 * 128 {
                return Err("Invalid remaining length: multiplier overflow".to_string());
            }
        }
        
        let packet_type = PacketType::from_u8(packet_type_value).ok_or("Invalid packet type".to_string())?;
        
        Ok(Self {
            packet_type,
            flags,
            remaining_length: remaining_length as usize,
        })
    }
}


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
    Connect(connect::ConnectPacket),
    ConnAck(connack::ConnAckPacket),
    Publish(publish::PublishPacket),
    PubAck(puback::PubAckPacket),
    PubRec(pubrec::PubRecPacket),
    PubRel(pubrel::PubRelPacket),
    PubComp(pubcomp::PubCompPacket),
    Subscribe(subscribe::SubscribePacket),
    SubAck(suback::SubAckPacket),
    Unsubscribe(unsubscribe::UnsubscribePacket),
    UnsubAck(unsuback::UnsubAckPacket),
    PingReq,
    PingResp,
    Disconnect,
}


pub use connect::*;
pub use connack::*;
pub use publish::*;
pub use puback::*;
pub use pubrec::*;
pub use pubrel::*;
pub use pubcomp::*;
pub use subscribe::*;
pub use suback::*;
pub use unsubscribe::*;
pub use unsuback::*;

/// 解析MQTT数据包
pub fn parse_mqtt_packet(input: &[u8]) -> Result<MqttPacket, String> {
    // 将输入转换为BytesMut
    let mut buffer = BytesMut::from(input);
    
    // 解析固定头
    let fixed_header = FixedHeader::parse(&mut buffer)?;
    
    // 检查剩余数据长度是否足够
    if buffer.len() < fixed_header.remaining_length {
        return Err("Insufficient data for remaining length".to_string());
    }
    
    // 提取剩余的数据部分
    let mut remaining_data = buffer.split_to(fixed_header.remaining_length);
    
    // 根据数据包类型解析剩余部分
    match fixed_header.packet_type {
        PacketType::Connect => {
            let connect_packet = connect::parse_connect(&mut remaining_data)?;
            Ok(MqttPacket::Connect(connect_packet))
        },
        PacketType::ConnAck => {
            let connack_packet = connack::parse_connack(&mut remaining_data)?;
            Ok(MqttPacket::ConnAck(connack_packet))
        },
        PacketType::Publish => {
            let publish_packet = publish::parse_publish(&mut remaining_data, fixed_header.flags)?;
            Ok(MqttPacket::Publish(publish_packet))
        },
        PacketType::PubAck => {
            let puback_packet = puback::parse_puback(&mut remaining_data)?;
            Ok(MqttPacket::PubAck(puback_packet))
        },
        PacketType::PubRec => {
            let pubrec_packet = pubrec::parse_pubrec(&mut remaining_data)?;
            Ok(MqttPacket::PubRec(pubrec_packet))
        },
        PacketType::PubRel => {
            let pubrel_packet = pubrel::parse_pubrel(&mut remaining_data)?;
            Ok(MqttPacket::PubRel(pubrel_packet))
        },
        PacketType::PubComp => {
            let pubcomp_packet = pubcomp::parse_pubcomp(&mut remaining_data)?;
            Ok(MqttPacket::PubComp(pubcomp_packet))
        },
        PacketType::Subscribe => {
            let subscribe_packet = subscribe::parse_subscribe(&mut remaining_data)?;
            Ok(MqttPacket::Subscribe(subscribe_packet))
        },
        PacketType::SubAck => {
            let suback_packet = suback::parse_suback(&mut remaining_data)?;
            Ok(MqttPacket::SubAck(suback_packet))
        },
        PacketType::Unsubscribe => {
            let unsubscribe_packet = unsubscribe::parse_unsubscribe(&mut remaining_data)?;
            Ok(MqttPacket::Unsubscribe(unsubscribe_packet))
        },
        PacketType::UnsubAck => {
            let unsuback_packet = unsuback::parse_unsuback(&mut remaining_data)?;
            Ok(MqttPacket::UnsubAck(unsuback_packet))
        },
        PacketType::PingReq => {
            Ok(MqttPacket::PingReq)
        },
        PacketType::PingResp => {
            Ok(MqttPacket::PingResp)
        },
        PacketType::Disconnect => {
            Ok(MqttPacket::Disconnect)
        },
    }
}
