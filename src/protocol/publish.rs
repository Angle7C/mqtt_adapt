use bytes::{Buf, Bytes, BytesMut};

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
fn parse_mqtt_string(input: &mut BytesMut) -> Result<String, String> {
    if input.len() < 2 {
        return Err("Insufficient data for MQTT string length".to_string());
    }
    
    let length = input.get_u16() as usize;
    
    if input.len() < length {
        return Err("Insufficient data for MQTT string content".to_string());
    }
    
    let bytes = input.split_to(length);
    let string = String::from_utf8_lossy(&bytes).to_string();
    Ok(string)
}

/// 解析PUBLISH数据包
pub fn parse_publish(input: &mut BytesMut, flags: u8) -> Result<PublishPacket, String> {
    let dup = (flags & 0x08) != 0;
    let qos = (flags & 0x06) >> 1;
    let retain = (flags & 0x01) != 0;
    
    let topic_name = parse_mqtt_string(input)?; 
    
    let mut packet_id = None;
    
    if qos > 0 {
        if input.len() < 2 {
            return Err("Insufficient data for packet ID".to_string());
        }
        
        let id = input.get_u16();
        packet_id = Some(id);
    }
    
    // 剩余的都是payload
    let payload = input.split().freeze();
    
    Ok(PublishPacket {
        dup,
        qos,
        retain,
        topic_name,
        packet_id,
        payload,
    })
}
