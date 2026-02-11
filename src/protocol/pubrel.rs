use nom::{IResult, number::complete::be_u16};

/// PUBREL数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubRelPacket {
    pub packet_id: u16,
}

/// 解析PUBREL数据包
pub fn parse_pubrel(input: &[u8]) -> IResult<&[u8], PubRelPacket> {
    let (input, packet_id) = be_u16(input)?;
    Ok((input, PubRelPacket { packet_id }))
}
