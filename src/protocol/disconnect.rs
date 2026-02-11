use nom::{IResult};

/// 解析DISCONNECT数据包
pub fn parse_disconnect(input: &[u8]) -> IResult<&[u8], ()> {
    Ok((input, ()))
}
