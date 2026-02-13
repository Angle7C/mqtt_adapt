use super::Packet;
use super::write_remaining_length;
use bytes::{Buf, BufMut, BytesMut};
use anyhow::Result;
/// SUBACK数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubAckPacket {
    pub packet_id: u16,
    pub return_codes: u8,
}

/// 解析SUBACK数据包
pub fn parse_suback(input: &mut BytesMut) -> Result<SubAckPacket> { 
    SubAckPacket::parse(input, None)
}

impl Packet for SubAckPacket {
    /// 将SUBACK数据包序列化为字节并写入缓冲区
    fn write(&self, buf: &mut BytesMut) {
        // 计算可变头和载荷长度
        let variable_header_length = 2; // 数据包ID
        
        // 总剩余长度
        let remaining_length = variable_header_length + 1;
        
        // 写入固定头
        let packet_type = 9; // SUBACK
        let flags = 0x00;
        let first_byte = (packet_type << 4) | flags;
        buf.put_u8(first_byte);
        
        // 写入剩余长度
        write_remaining_length(buf, remaining_length);
        
        // 写入可变头（数据包ID）
        buf.put_u16(self.packet_id);
        buf.put_u8(self.return_codes);
        // 写入载荷（返回码列表）
       
    }
    
    /// 从BytesMut解析SUBACK数据包
    fn parse(input: &mut BytesMut, _flags: Option<u8>) -> Result<Self> {
        if input.len() < 2 {
            return Err(anyhow::format_err!("Insufficient data for SUBACK packet")); 
        }
        
        let packet_id = input.get_u16();
        
        let code = input.get_u8();
     
        Ok(SubAckPacket {
            packet_id,
            return_codes: code,
        })
    }
}
