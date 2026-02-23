use super::Packet;
use super::parse_mqtt_string;
use super::write_mqtt_string;
use super::write_remaining_length;
use bytes::{Buf, BufMut, BytesMut};
use anyhow::Result;
/// UNSUBSCRIBE数据包
#[derive(Debug,  PartialEq, Eq)]
pub struct UnsubscribePacket {
    pub packet_id: u16,
    pub topics: Vec<String>,
}




impl Packet for UnsubscribePacket {
    /// 将UNSUBSCRIBE数据包序列化为字节并写入缓冲区
    fn write(&self, buf: &mut BytesMut) {
        // 计算可变头和载荷长度
        let variable_header_length = 2; // 数据包ID
        let mut payload_length = 0;
        
        // 计算每个主题的长度
        for topic in &self.topics {
            payload_length += 2 + topic.len(); // 主题名长度前缀 + 主题名
        }
        
        // 总剩余长度
        let remaining_length = variable_header_length + payload_length;
        
        // 写入固定头
        let packet_type = 10; // UNSUBSCRIBE
        let flags = 0x02; // UNSUBSCRIBE固定标志位为0x02
        let first_byte = (packet_type << 4) | flags;
        buf.put_u8(first_byte);
        
        // 写入剩余长度
        write_remaining_length(buf, remaining_length);
        
        // 写入可变头（数据包ID）
        buf.put_u16(self.packet_id);
        
        // 写入载荷（主题列表）
        for topic in &self.topics {
            write_mqtt_string(buf, topic);
        }
    }
    
    /// 从BytesMut解析UNSUBSCRIBE数据包
    fn parse(input: &mut BytesMut, _flags: Option<u8>) -> Result<Self> {
        if input.len() < 2 {
            return Err(anyhow::format_err!("Insufficient data for UNSUBSCRIBE packet"));
        }
        
        let packet_id = input.get_u16();
        
        let mut topics = Vec::new();
        
        while !input.is_empty() {
            let topic = parse_mqtt_string(input)?;
            topics.push(topic);
        }
        
        Ok(UnsubscribePacket {
            packet_id,
            topics,
        })
    }
}
