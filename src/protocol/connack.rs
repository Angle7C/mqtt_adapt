use super::ConnectReturnCode;
use bytes::{Buf, BytesMut};

/// CONNACK数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnAckPacket {
    pub session_present: bool,
    pub return_code: ConnectReturnCode,
}

/// 解析CONNACK数据包
pub fn parse_connack(input: &mut BytesMut) -> Result<ConnAckPacket, String> {
    if input.len() < 2 {
        return Err("Insufficient data for CONNACK packet".to_string());
    }
    
    let flags = input.get_u8();
    let return_code_value = input.get_u8();
    
    let session_present = (flags & 0x01) != 0;
    let return_code = ConnectReturnCode::from_u8(return_code_value).ok_or("Invalid return code".to_string())?;
    
    Ok(ConnAckPacket {
        session_present,
        return_code,
    })
}
