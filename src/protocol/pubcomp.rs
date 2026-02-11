use nom::{IResult, number::complete::be_u16};

/// PUBCOMP数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubCompPacket {
    pub packet_id: u16,
}

/// 解析PUBCOMP数据包
pub fn parse_pubcomp(input: &[u8]) -> IResult<&[u8], PubCompPacket> {
    let (input, packet_id) = be_u16(input)?;
    Ok((input, PubCompPacket { packet_id }))
}
