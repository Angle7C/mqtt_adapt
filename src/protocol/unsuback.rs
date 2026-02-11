use nom::{IResult, number::complete::be_u16};

/// UNSUBACK数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsubAckPacket {
    pub packet_id: u16,
}

/// 解析UNSUBACK数据包
pub fn parse_unsuback(input: &[u8]) -> IResult<&[u8], UnsubAckPacket> {
    let (input, packet_id) = be_u16(input)?;
    Ok((input, UnsubAckPacket { packet_id }))
}
