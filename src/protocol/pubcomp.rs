use bytes::{Buf, BytesMut};

/// PUBCOMP数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubCompPacket {
    pub packet_id: u16,
}

/// 解析PUBCOMP数据包
pub fn parse_pubcomp(input: &mut BytesMut) -> Result<PubCompPacket, String> {
    if input.len() < 2 {
        return Err("Insufficient data for PUBCOMP packet".to_string());
    }
    
    let packet_id = input.get_u16();
    Ok(PubCompPacket { packet_id })
}
