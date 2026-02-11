use bytes::{Buf, BytesMut};

/// UNSUBSCRIBE数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsubscribePacket {
    pub packet_id: u16,
    pub topics: Vec<String>,
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

/// 解析UNSUBSCRIBE数据包
pub fn parse_unsubscribe(input: &mut BytesMut) -> Result<UnsubscribePacket, String> {
    if input.len() < 2 {
        return Err("Insufficient data for UNSUBSCRIBE packet".to_string());
    }
    
    let packet_id = input.get_u16();
    
    let mut topics = Vec::new();
    
    while !input.is_empty() {
        let topic = parse_mqtt_string(input)?;
        topics.push(topic);
    }
    
    Ok(UnsubscribePacket {
        packet_id,
        topics,
    })
}
