use bytes::{Bytes, BytesMut};
use nom::{IResult, bytes::complete::take, number::complete::{be_u8, be_u16}};
use anyhow::Result;

/// 从BytesMut中读取并解析MQTT数据包
pub fn read_from_bytes_mut(buf: &mut BytesMut) -> Option<Bytes> {
    if buf.len() < 2 {
        return None;
    }
    
    // 解析剩余长度
    let mut multiplier = 1;
    let mut remaining_length = 0;
    let mut i = 1;
    
    loop {
        if i >= buf.len() {
            return None;
        }
        
        let byte = buf[i];
        remaining_length += ((byte & 0x7F) as usize) * multiplier;
        multiplier *= 128;
        i += 1;
        
        if (byte & 0x80) == 0 {
            break;
        }
    }
    
    // 检查是否有足够的数据
    let total_length = i + remaining_length;
    if buf.len() < total_length {
        return None;
    }
    
    // 提取完整的数据包
    let packet = buf.split_to(total_length).freeze();
    Some(packet)
}

/// 解析剩余长度
pub fn parse_remaining_length(input: &[u8]) -> IResult<&[u8], usize> {
    let mut multiplier = 1;
    let mut value = 0;
    let mut input = input;
    
    loop {
        let (rest, byte) = be_u8(input)?;
        value += ((byte & 0x7F) as usize) * multiplier;
        multiplier *= 128;
        input = rest;
        
        if (byte & 0x80) == 0 {
            break;
        }
    }
    
    Ok((input, value))
}

/// 解析MQTT字符串
pub fn parse_mqtt_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, length) = be_u16(input)?;
    let (input, bytes) = take(length)(input)?;
    let string = String::from_utf8_lossy(bytes).to_string();
    Ok((input, string))
}

/// 解析MQTT二进制数据
pub fn parse_mqtt_binary(input: &[u8]) -> IResult<&[u8], Bytes> {
    let (input, length) = be_u16(input)?;
    let (input, bytes) = take(length)(input)?;
    Ok((input, Bytes::copy_from_slice(bytes)))
}

/// 解析固定头
pub fn parse_fixed_header(input: &[u8]) -> IResult<&[u8], super::FixedHeader> {
    use super::PacketType;
    
    let (input, first_byte) = be_u8(input)?;
    let packet_type = PacketType::from_u8((first_byte & 0xF0) >> 4).ok_or(nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))?;
    let flags = first_byte & 0x0F;
    let (input, remaining_length) = parse_remaining_length(input)?;
    
    Ok((input, super::FixedHeader {
        packet_type,
        flags,
        remaining_length,
    }))
}

/// 解析MQTT数据包
pub fn parse_packet(input: &[u8]) -> IResult<&[u8], super::MqttPacket> {
    let (input, fixed_header) = parse_fixed_header(input)?;
    let (input, variable_header_and_payload) = take(fixed_header.remaining_length)(input)?;
    
    let packet = match fixed_header.packet_type {
        super::PacketType::Connect => {
            let (_, connect) = parse_connect(variable_header_and_payload)?;
            super::MqttPacket::Connect(connect)
        }
        super::PacketType::ConnAck => {
            let (_, connack) = parse_connack(variable_header_and_payload)?;
            super::MqttPacket::ConnAck(connack)
        }
        super::PacketType::Publish => {
            let (_, publish) = parse_publish(variable_header_and_payload, fixed_header.flags)?;
            super::MqttPacket::Publish(publish)
        }
        super::PacketType::PubAck => {
            let (_, puback) = parse_puback(variable_header_and_payload)?;
            super::MqttPacket::PubAck(puback)
        }
        super::PacketType::PubRec => {
            let (_, pubrec) = parse_pubrec(variable_header_and_payload)?;
            super::MqttPacket::PubRec(pubrec)
        }
        super::PacketType::PubRel => {
            let (_, pubrel) = parse_pubrel(variable_header_and_payload)?;
            super::MqttPacket::PubRel(pubrel)
        }
        super::PacketType::PubComp => {
            let (_, pubcomp) = parse_pubcomp(variable_header_and_payload)?;
            super::MqttPacket::PubComp(pubcomp)
        }
        super::PacketType::Subscribe => {
            let (_, subscribe) = parse_subscribe(variable_header_and_payload)?;
            super::MqttPacket::Subscribe(subscribe)
        }
        super::PacketType::SubAck => {
            let (_, suback) = parse_suback(variable_header_and_payload)?;
            super::MqttPacket::SubAck(suback)
        }
        super::PacketType::Unsubscribe => {
            let (_, unsubscribe) = parse_unsubscribe(variable_header_and_payload)?;
            super::MqttPacket::Unsubscribe(unsubscribe)
        }
        super::PacketType::UnsubAck => {
            let (_, unsuback) = parse_unsuback(variable_header_and_payload)?;
            super::MqttPacket::UnsubAck(unsuback)
        }
        super::PacketType::PingReq => {
            super::MqttPacket::PingReq
        }
        super::PacketType::PingResp => {
            super::MqttPacket::PingResp
        }
        super::PacketType::Disconnect => {
            super::MqttPacket::Disconnect
        }
    };
    
    Ok((input, packet))
}

/// 解析CONNECT数据包
pub fn parse_connect(input: &[u8]) -> IResult<&[u8], super::connect::ConnectPacket> {
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
    
    Ok((input, super::connect::ConnectPacket {
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

/// 解析CONNACK数据包
pub fn parse_connack(input: &[u8]) -> IResult<&[u8], super::connack::ConnAckPacket> {
    let (input, flags) = be_u8(input)?;
    let (input, return_code) = be_u8(input)?;
    
    let session_present = (flags & 0x01) != 0;
    let return_code = super::ConnectReturnCode::from_u8(return_code).ok_or(nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))?;
    
    Ok((input, super::connack::ConnAckPacket {
        session_present,
        return_code,
    }))
}

/// 解析PUBLISH数据包
pub fn parse_publish(input: &[u8], flags: u8) -> IResult<&[u8], super::publish::PublishPacket> {
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
    
    Ok((input, super::publish::PublishPacket {
        dup,
        qos,
        retain,
        topic_name,
        packet_id,
        payload: Bytes::copy_from_slice(payload),
    }))
}

/// 解析PUBACK数据包
pub fn parse_puback(input: &[u8]) -> IResult<&[u8], super::puback::PubAckPacket> {
    let (input, packet_id) = be_u16(input)?;
    Ok((input, super::puback::PubAckPacket { packet_id }))
}

/// 解析PUBREC数据包
pub fn parse_pubrec(input: &[u8]) -> IResult<&[u8], super::pubrec::PubRecPacket> {
    let (input, packet_id) = be_u16(input)?;
    Ok((input, super::pubrec::PubRecPacket { packet_id }))
}

/// 解析PUBREL数据包
pub fn parse_pubrel(input: &[u8]) -> IResult<&[u8], super::pubrel::PubRelPacket> {
    let (input, packet_id) = be_u16(input)?;
    Ok((input, super::pubrel::PubRelPacket { packet_id }))
}

/// 解析PUBCOMP数据包
pub fn parse_pubcomp(input: &[u8]) -> IResult<&[u8], super::pubcomp::PubCompPacket> {
    let (input, packet_id) = be_u16(input)?;
    Ok((input, super::pubcomp::PubCompPacket { packet_id }))
}

/// 解析SUBSCRIBE数据包
pub fn parse_subscribe(input: &[u8]) -> IResult<&[u8], super::subscribe::SubscribePacket> {
    let (input, packet_id) = be_u16(input)?;
    
    let mut topics = Vec::new();
    let mut input = input;
    
    while !input.is_empty() {
        let (rest, topic) = parse_mqtt_string(input)?;
        let (rest, qos) = be_u8(rest)?;
        topics.push((topic, qos));
        input = rest;
    }
    
    Ok((input, super::subscribe::SubscribePacket {
        packet_id,
        topics,
    }))
}

/// 解析SUBACK数据包
pub fn parse_suback(input: &[u8]) -> IResult<&[u8], super::suback::SubAckPacket> {
    let (input, packet_id) = be_u16(input)?;
    
    let mut return_codes = Vec::new();
    let mut input = input;
    
    while !input.is_empty() {
        let (rest, code) = be_u8(input)?;
        return_codes.push(code);
        input = rest;
    }
    
    Ok((input, super::suback::SubAckPacket {
        packet_id,
        return_codes,
    }))
}

/// 解析UNSUBSCRIBE数据包
pub fn parse_unsubscribe(input: &[u8]) -> IResult<&[u8], super::unsubscribe::UnsubscribePacket> {
    let (input, packet_id) = be_u16(input)?;
    
    let mut topics = Vec::new();
    let mut input = input;
    
    while !input.is_empty() {
        let (rest, topic) = parse_mqtt_string(input)?;
        topics.push(topic);
        input = rest;
    }
    
    Ok((input, super::unsubscribe::UnsubscribePacket {
        packet_id,
        topics,
    }))
}

/// 解析UNSUBACK数据包
pub fn parse_unsuback(input: &[u8]) -> IResult<&[u8], super::unsuback::UnsubAckPacket> {
    let (input, packet_id) = be_u16(input)?;
    Ok((input, super::unsuback::UnsubAckPacket { packet_id }))
}

/// 解析PINGREQ数据包
pub fn parse_ping_req(input: &[u8]) -> IResult<&[u8], ()> {
    Ok((input, ()))
}

/// 解析PINGRESP数据包
pub fn parse_ping_resp(input: &[u8]) -> IResult<&[u8], ()> {
    Ok((input, ()))
}

/// 解析DISCONNECT数据包
pub fn parse_disconnect(input: &[u8]) -> IResult<&[u8], ()> {
    Ok((input, ()))
}

/// 将nom错误转换为anyhow错误
fn nom_to_anyhow<T>(result: IResult<&[u8], T>, context: &str) -> Result<T> {
    match result {
        Ok((_, value)) => Ok(value),
        Err(nom::Err::Incomplete(_)) => Err(anyhow::anyhow!("{}: incomplete data", context)),
        Err(nom::Err::Error(e)) => Err(anyhow::anyhow!("{}: parsing error: {:?}", context, e)),
        Err(nom::Err::Failure(e)) => Err(anyhow::anyhow!("{}: parsing failure: {:?}", context, e)),
    }
}

/// 解析MQTT数据包（使用anyhow错误处理）
pub fn parse_packet_anyhow(input: &[u8]) -> Result<super::MqttPacket> {
    nom_to_anyhow(parse_packet(input), "parse MQTT packet")
}

/// 解析固定头（使用anyhow错误处理）
pub fn parse_fixed_header_anyhow(input: &[u8]) -> Result<super::FixedHeader> {
    nom_to_anyhow(parse_fixed_header(input), "parse fixed header")
}

/// 解析CONNECT数据包（使用anyhow错误处理）
pub fn parse_connect_anyhow(input: &[u8]) -> Result<super::connect::ConnectPacket> {
    nom_to_anyhow(parse_connect(input), "parse CONNECT packet")
}

/// 解析PUBLISH数据包（使用anyhow错误处理）
pub fn parse_publish_anyhow(input: &[u8], flags: u8) -> Result<super::publish::PublishPacket> {
    nom_to_anyhow(parse_publish(input, flags), "parse PUBLISH packet")
}

/// 解析SUBSCRIBE数据包（使用anyhow错误处理）
pub fn parse_subscribe_anyhow(input: &[u8]) -> Result<super::subscribe::SubscribePacket> {
    nom_to_anyhow(parse_subscribe(input), "parse SUBSCRIBE packet")
}
