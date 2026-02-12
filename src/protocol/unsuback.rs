use super::Packet;
use super::write_remaining_length;
use bytes::{Buf, BufMut, BytesMut};

/// UNSUBACK数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsubAckPacket {
    pub packet_id: u16,
}


impl Packet for UnsubAckPacket {
    /// 将UNSUBACK数据包序列化为字节并写入缓冲区
    fn write(&self, buf: &mut BytesMut) {
        // 可变头长度（数据包ID）
        let variable_header_length = 2;
        
        // 写入固定头
        let packet_type = 11; // UNSUBACK
        let flags = 0x00;
        let first_byte = (packet_type << 4) | flags;
        buf.put_u8(first_byte);
        
        // 写入剩余长度
        write_remaining_length(buf, variable_header_length);
        
        // 写入可变头（数据包ID）
        buf.put_u16(self.packet_id);
    }
    
    /// 从BytesMut解析UNSUBACK数据包
    fn parse(input: &mut BytesMut, _flags: Option<u8>) -> Result<Self, String> {
        if input.len() < 2 {
            return Err("Insufficient data for UNSUBACK packet".to_string());
        }
        
        let packet_id = input.get_u16();
        Ok(UnsubAckPacket { packet_id })
    }
}
