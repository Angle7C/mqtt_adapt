use bytes::{Buf, BytesMut};

/// PUBREC数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubRecPacket {
    pub packet_id: u16,
}

/// 解析PUBREC数据包
pub fn parse_pubrec(input: &mut BytesMut) -> Result<PubRecPacket, String> {
    if input.len() < 2 {
        return Err("Insufficient data for PUBREC packet".to_string());
    }
    
    let packet_id = input.get_u16();
    Ok(PubRecPacket { packet_id })
}
