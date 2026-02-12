use super::Packet;
use super::parse_mqtt_string;
use super::write_mqtt_string;
use super::write_remaining_length;
use bytes::{Buf, BufMut, BytesMut};
use anyhow::Result;
/// SUBSCRIBE数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribePacket {
    pub packet_id: u16,
    pub topics: Vec<(String, u8)>,
}




impl Packet for SubscribePacket {
    /// 将SUBSCRIBE数据包序列化为字节并写入缓冲区
    fn write(&self, buf: &mut BytesMut) {
        // 计算可变头和载荷长度
        let mut variable_header_length = 2; // 数据包ID
        let mut payload_length = 0;
        
        // 计算每个主题和QoS的长度
        for (topic, _) in &self.topics {
            payload_length += 2 + topic.len(); // 主题名长度前缀 + 主题名
            payload_length += 1; // QoS级别
        }
        
        // 总剩余长度
        let remaining_length = variable_header_length + payload_length;
        
        // 写入固定头
        let packet_type = 8; // SUBSCRIBE
        let flags = 0x02; // SUBSCRIBE固定标志位为0x02
        let first_byte = (packet_type << 4) | flags;
        buf.put_u8(first_byte);
        
        // 写入剩余长度
        write_remaining_length(buf, remaining_length);
        
        // 写入可变头（数据包ID）
        buf.put_u16(self.packet_id);
        
        // 写入载荷（主题列表和QoS级别）
        for (topic, qos) in &self.topics {
            write_mqtt_string(buf, topic);
            buf.put_u8(*qos);
        }
    }
    
    /// 从BytesMut解析SUBSCRIBE数据包
    fn parse(input: &mut BytesMut, _flags: Option<u8>) -> Result<Self> {
        if input.len() < 2 {
            return Err(anyhow::format_err!("Insufficient data for SUBSCRIBE packet"));  
        }
        
        let packet_id = input.get_u16();
        
        let mut topics = Vec::new();
        
        while !input.is_empty() {
            let topic = parse_mqtt_string(input)?;
            
            if input.is_empty() {
                return Err(anyhow::format_err!("Insufficient data for QoS level")); 
            }
            
            let qos = input.get_u8();
            topics.push((topic, qos));
        }
        
        Ok(SubscribePacket {
            packet_id,
            topics,
        })
    }
}
