use bytes::{Buf, BytesMut};

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
