use nom::{IResult, bytes::complete::take, number::complete::{be_u8, be_u16}};

/// UNSUBSCRIBE数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsubscribePacket {
    pub packet_id: u16,
    pub topics: Vec<String>,
}

/// 解析MQTT字符串
fn parse_mqtt_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, length) = be_u16(input)?;
    let (input, bytes) = take(length)(input)?;
    let string = String::from_utf8_lossy(bytes).to_string();
    Ok((input, string))
}

/// 解析UNSUBSCRIBE数据包
pub fn parse_unsubscribe(input: &[u8]) -> IResult<&[u8], UnsubscribePacket> {
    let (input, packet_id) = be_u16(input)?;
    
    let mut topics = Vec::new();
    let mut input = input;
    
    while !input.is_empty() {
        let (rest, topic) = parse_mqtt_string(input)?;
        topics.push(topic);
        input = rest;
    }
    
    Ok((input, UnsubscribePacket {
        packet_id,
        topics,
    }))
}
