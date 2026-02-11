use bytes::{Buf, BytesMut};

/// UNSUBACK数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsubAckPacket {
    pub packet_id: u16,
}

/// 解析UNSUBACK数据包
pub fn parse_unsuback(input: &mut BytesMut) -> Result<UnsubAckPacket, String> {
    if input.len() < 2 {
        return Err("Insufficient data for UNSUBACK packet".to_string());
    }
    
    let packet_id = input.get_u16();
    Ok(UnsubAckPacket { packet_id })
}
