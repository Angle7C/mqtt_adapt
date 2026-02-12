use super::Packet;
use super::parse_mqtt_string;
use super::write_mqtt_string;
use super::write_remaining_length;
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// PUBLISH数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishPacket {
    pub dup: bool,
    pub qos: u8,
    pub retain: bool,
    pub topic_name: String,
    pub packet_id: Option<u16>,
    pub payload: Bytes,
}



/// 解析PUBLISH数据包
pub fn parse_publish(input: &mut BytesMut, flags: u8) -> Result<PublishPacket, String> {
    let dup = (flags & 0x08) != 0;
    let qos = (flags & 0x06) >> 1;
    let retain = (flags & 0x01) != 0;
    
    let topic_name = parse_mqtt_string(input)?;
    
    let mut packet_id = None;
    
    if qos > 0 {
        if input.len() < 2 {
            return Err("Insufficient data for packet ID".to_string());
        }
        
        let id = input.get_u16();
        packet_id = Some(id);
    }
    
    // 剩余的都是payload
    let payload = input.split().freeze();
    
    Ok(PublishPacket {
        dup,
        qos,
        retain,
        topic_name,
        packet_id,
        payload,
    })
}

impl Packet for PublishPacket {
    /// 将PUBLISH数据包序列化为字节并写入缓冲区
    fn write(&self, buf: &mut BytesMut) {
        // 计算可变头和载荷长度
        let mut variable_header_length = 0;
        
        // 主题名称长度
        variable_header_length += 2 + self.topic_name.len();
        
        // 数据包ID长度（仅当QoS > 0时）
        if self.qos > 0 {
            variable_header_length += 2;
        }
        
        // 载荷长度
        let payload_length = self.payload.len();
        
        // 总剩余长度
        let remaining_length = variable_header_length + payload_length;
        
        // 构建固定头的标志位
        let mut flags = 0;
        if self.dup {
            flags |= 0x08;
        }
        flags |= (self.qos & 0x03) << 1;
        if self.retain {
            flags |= 0x01;
        }
        
        // 写入固定头
        let packet_type = 3; // PUBLISH
        let first_byte = (packet_type << 4) | flags;
        buf.put_u8(first_byte);
        
        // 写入剩余长度
        write_remaining_length(buf, remaining_length);
        
        // 写入可变头
        // 主题名称
        write_mqtt_string(buf, &self.topic_name);
        
        // 数据包ID（仅当QoS > 0时）
        if self.qos > 0 {
            if let Some(id) = self.packet_id {
                buf.put_u16(id);
            }
        }
        
        // 写入载荷
        buf.put_slice(&self.payload);
    }
    
    /// 从BytesMut解析PUBLISH数据包
    /// 注意：此方法使用默认的flags值，实际使用中应使用parse_publish函数
    fn parse(input: &mut BytesMut, flags: Option<u8>) -> Result<Self, String> {
        // 使用传入的flags值或默认值（无dup，QoS 0，无retain）
        parse_publish(input, flags.unwrap_or(0x00))
    }
}
