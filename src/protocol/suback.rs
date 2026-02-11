use super::write_remaining_length;
use bytes::{Buf, BufMut, BytesMut};

/// SUBACK数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubAckPacket {
    pub packet_id: u16,
    pub return_codes: Vec<u8>,
}

/// 解析SUBACK数据包
pub fn parse_suback(input: &mut BytesMut) -> Result<SubAckPacket, String> {
    if input.len() < 2 {
        return Err("Insufficient data for SUBACK packet".to_string());
    }
    
    let packet_id = input.get_u16();
    
    let mut return_codes = Vec::new();
    
    while !input.is_empty() {
        let code = input.get_u8();
        return_codes.push(code);
    }
    
    Ok(SubAckPacket {
        packet_id,
        return_codes,
    })
}

impl SubAckPacket {
    /// 将SUBACK数据包序列化为字节并写入缓冲区
    pub fn write(&self, buf: &mut BytesMut) {
        // 计算可变头和载荷长度
        let variable_header_length = 2; // 数据包ID
        let payload_length = self.return_codes.len(); // 返回码数量
        
        // 总剩余长度
        let remaining_length = variable_header_length + payload_length;
        
        // 写入固定头
        let packet_type = 9; // SUBACK
        let flags = 0x00;
        let first_byte = (packet_type << 4) | flags;
        buf.put_u8(first_byte);
        
        // 写入剩余长度
        write_remaining_length(buf, remaining_length);
        
        // 写入可变头（数据包ID）
        buf.put_u16(self.packet_id);
        
        // 写入载荷（返回码列表）
        for code in &self.return_codes {
            buf.put_u8(*code);
        }
    }
}
