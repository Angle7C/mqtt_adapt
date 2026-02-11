use bytes::Bytes;
use nom::{IResult, bytes::complete::take, number::complete::{be_u8, be_u16}};

/// PUBLISH数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishPacket {
    pub dup: bool,
    pub qos: u8,
    pub retain: bool,
    pub topic_name: String,
    pub packet_id: Option<u16>,
    pub payload: Bytes,
}

/// 解析MQTT字符串
fn parse_mqtt_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, length) = be_u16(input)?;
    let (input, bytes) = take(length)(input)?;
    let string = String::from_utf8_lossy(bytes).to_string();
    Ok((input, string))
}

/// 解析PUBLISH数据包
pub fn parse_publish(input: &[u8], flags: u8) -> IResult<&[u8], PublishPacket> {
    let dup = (flags & 0x08) != 0;
    let qos = (flags & 0x06) >> 1;
    let retain = (flags & 0x01) != 0;
    
    let (input, topic_name) = parse_mqtt_string(input)?;
    
    let mut packet_id = None;
    let mut input = input;
    
    if qos > 0 {
        let (rest, id) = be_u16(input)?;
        packet_id = Some(id);
        input = rest;
    }
    
    // 剩余的都是payload
    let (input, payload) = take(input.len())(input)?;
    
    Ok((input, PublishPacket {
        dup,
        qos,
        retain,
        topic_name,
        packet_id,
        payload: Bytes::copy_from_slice(payload),
    }))
}
