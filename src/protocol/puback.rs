use bytes::{Buf, BytesMut};

/// PUBACK数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubAckPacket {
    pub packet_id: u16,
}

/// 解析PUBACK数据包
pub fn parse_puback(input: &mut BytesMut) -> Result<PubAckPacket, String> {
    if input.len() < 2 {
        return Err("Insufficient data for PUBACK packet".to_string());
    }
    
    let packet_id = input.get_u16();
    Ok(PubAckPacket { packet_id })
}
