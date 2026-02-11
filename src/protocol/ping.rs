use nom::{IResult};

/// 解析PINGREQ数据包
pub fn parse_ping_req(input: &[u8]) -> IResult<&[u8], ()> {
    Ok((input, ()))
}

/// 解析PINGRESP数据包
pub fn parse_ping_resp(input: &[u8]) -> IResult<&[u8], ()> {
    Ok((input, ()))
}
