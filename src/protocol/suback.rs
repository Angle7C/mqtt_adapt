use bytes::{Buf, BytesMut};

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
