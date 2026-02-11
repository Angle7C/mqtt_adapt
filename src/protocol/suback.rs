use nom::{IResult, number::complete::{be_u8, be_u16}};

/// SUBACK数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubAckPacket {
    pub packet_id: u16,
    pub return_codes: Vec<u8>,
}

/// 解析SUBACK数据包
pub fn parse_suback(input: &[u8]) -> IResult<&[u8], SubAckPacket> {
    let (input, packet_id) = be_u16(input)?;
    
    let mut return_codes = Vec::new();
    let mut input = input;
    
    while !input.is_empty() {
        let (rest, code) = be_u8(input)?;
        return_codes.push(code);
        input = rest;
    }
    
    Ok((input, SubAckPacket {
        packet_id,
        return_codes,
    }))
}
