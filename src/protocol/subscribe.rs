use bytes::{Buf, BytesMut};

/// SUBSCRIBE数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribePacket {
    pub packet_id: u16,
    pub topics: Vec<(String, u8)>,
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

/// 解析SUBSCRIBE数据包
pub fn parse_subscribe(input: &mut BytesMut) -> Result<SubscribePacket, String> {
    if input.len() < 2 {
        return Err("Insufficient data for SUBSCRIBE packet".to_string());
    }
    
    let packet_id = input.get_u16();
    
    let mut topics = Vec::new();
    
    while !input.is_empty() {
        let topic = parse_mqtt_string(input)?;
        
        if input.is_empty() {
            return Err("Insufficient data for QoS level".to_string());
        }
        
        let qos = input.get_u8();
        topics.push((topic, qos));
    }
    
    Ok(SubscribePacket {
        packet_id,
        topics,
    })
}
