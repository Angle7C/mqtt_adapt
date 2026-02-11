use super::ConnectReturnCode;
use nom::{IResult, number::complete::be_u8};

/// CONNACK数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnAckPacket {
    pub session_present: bool,
    pub return_code: ConnectReturnCode,
}

/// 解析CONNACK数据包
pub fn parse_connack(input: &[u8]) -> IResult<&[u8], ConnAckPacket> {
    let (input, flags) = be_u8(input)?;
    let (input, return_code) = be_u8(input)?;
    
    let session_present = (flags & 0x01) != 0;
    let return_code = ConnectReturnCode::from_u8(return_code).ok_or(nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))?;
    
    Ok((input, ConnAckPacket {
        session_present,
        return_code,
    }))
}
