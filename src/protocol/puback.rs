use nom::{IResult, number::complete::be_u16};

/// PUBACK数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubAckPacket {
    pub packet_id: u16,
}

/// 解析PUBACK数据包
pub fn parse_puback(input: &[u8]) -> IResult<&[u8], PubAckPacket> {
    let (input, packet_id) = be_u16(input)?;
    Ok((input, PubAckPacket { packet_id }))
}
