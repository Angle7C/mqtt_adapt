use bytes::Bytes;
use nom::{IResult, bytes::complete::take, number::complete::{be_u8, be_u16}};

/// CONNECT数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectPacket {
    pub protocol_name: String,
    pub protocol_level: u8,
    pub connect_flags: u8,
    pub keep_alive: u16,
    pub client_id: String,
    pub will_topic: Option<String>,
    pub will_message: Option<Bytes>,
    pub username: Option<String>,
    pub password: Option<Bytes>,
}

/// 解析MQTT字符串
fn parse_mqtt_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, length) = be_u16(input)?;
    let (input, bytes) = take(length)(input)?;
    let string = String::from_utf8_lossy(bytes).to_string();
    Ok((input, string))
}

/// 解析MQTT二进制数据
fn parse_mqtt_binary(input: &[u8]) -> IResult<&[u8], Bytes> {
    let (input, length) = be_u16(input)?;
    let (input, bytes) = take(length)(input)?;
    Ok((input, Bytes::copy_from_slice(bytes)))
}

/// 解析CONNECT数据包
pub fn parse_connect(input: &[u8]) -> IResult<&[u8], ConnectPacket> {
    let (input, protocol_name) = parse_mqtt_string(input)?;
    let (input, protocol_level) = be_u8(input)?;
    let (input, connect_flags) = be_u8(input)?;
    let (input, keep_alive) = be_u16(input)?;
    let (input, client_id) = parse_mqtt_string(input)?;
    
    let mut will_topic = None;
    let mut will_message = None;
    let mut username = None;
    let mut password = None;
    
    let mut input = input;
    
    // 解析Will Topic和Will Message
    if (connect_flags & 0x04) != 0 {
        let (rest, topic) = parse_mqtt_string(input)?;
        will_topic = Some(topic);
        input = rest;
        
        let (rest, message) = parse_mqtt_binary(input)?;
        will_message = Some(message);
        input = rest;
    }
    
    // 解析Username
    if (connect_flags & 0x80) != 0 {
        let (rest, user) = parse_mqtt_string(input)?;
        username = Some(user);
        input = rest;
    }
    
    // 解析Password
    if (connect_flags & 0x40) != 0 {
        let (rest, pass) = parse_mqtt_binary(input)?;
        password = Some(pass);
        input = rest;
    }
    
    Ok((input, ConnectPacket {
        protocol_name,
        protocol_level,
        connect_flags,
        keep_alive,
        client_id,
        will_topic,
        will_message,
        username,
        password,
    }))
}
