use nom::{IResult, bytes::complete::take, number::complete::{be_u8, be_u16}};

/// SUBSCRIBE数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribePacket {
    pub packet_id: u16,
    pub topics: Vec<(String, u8)>,
}

/// 解析MQTT字符串
fn parse_mqtt_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, length) = be_u16(input)?;
    let (input, bytes) = take(length)(input)?;
    let string = String::from_utf8_lossy(bytes).to_string();
    Ok((input, string))
}

/// 解析SUBSCRIBE数据包
pub fn parse_subscribe(input: &[u8]) -> IResult<&[u8], SubscribePacket> {
    let (input, packet_id) = be_u16(input)?;
    
    let mut topics = Vec::new();
    let mut input = input;
    
    while !input.is_empty() {
        let (rest, topic) = parse_mqtt_string(input)?;
        let (rest, qos) = be_u8(rest)?;
        topics.push((topic, qos));
        input = rest;
    }
    
    Ok((input, SubscribePacket {
        packet_id,
        topics,
    }))
}
