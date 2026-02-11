use nom::{IResult, number::complete::be_u16};

/// PUBREC数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubRecPacket {
    pub packet_id: u16,
}

/// 解析PUBREC数据包
pub fn parse_pubrec(input: &[u8]) -> IResult<&[u8], PubRecPacket> {
    let (input, packet_id) = be_u16(input)?;
    Ok((input, PubRecPacket { packet_id }))
}
