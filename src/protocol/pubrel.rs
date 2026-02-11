use super::write_remaining_length;
use bytes::{Buf, BufMut, BytesMut};

/// PUBREL数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubRelPacket {
    pub packet_id: u16,
}

/// 解析PUBREL数据包
pub fn parse_pubrel(input: &mut BytesMut) -> Result<PubRelPacket, String> {
    if input.len() < 2 {
        return Err("Insufficient data for PUBREL packet".to_string());
    }
    
    let packet_id = input.get_u16();
    Ok(PubRelPacket { packet_id })
}

impl PubRelPacket {
    /// 将PUBREL数据包序列化为字节并写入缓冲区
    pub fn write(&self, buf: &mut BytesMut) {
        // 可变头长度（数据包ID）
        let variable_header_length = 2;
        
        // 写入固定头
        let packet_type = 6; // PUBREL
        let flags = 0x02; // PUBREL固定标志位为0x02
        let first_byte = (packet_type << 4) | flags;
        buf.put_u8(first_byte);
        
        // 写入剩余长度
        write_remaining_length(buf, variable_header_length);
        
        // 写入可变头（数据包ID）
        buf.put_u16(self.packet_id);
    }
}
