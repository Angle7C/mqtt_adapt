use bytes::{Buf, Bytes, BytesMut};
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

/// 解析MQTT二进制数据
fn parse_mqtt_bytes(input: &mut BytesMut) -> Result<Bytes, String> {
    if input.len() < 2 {
        return Err("Insufficient data for MQTT bytes length".to_string());
    }
    
    let length = input.get_u16() as usize;
    
    if input.len() < length {
        return Err("Insufficient data for MQTT bytes content".to_string());
    }
    
    let bytes = input.split_to(length);
    Ok(bytes.freeze())
}

/// 解析CONNECT数据包
pub fn parse_connect(input: &mut BytesMut) -> Result<ConnectPacket, String> {
    // 解析协议名称
    let protocol_name = parse_mqtt_string(input)?;
    
    // 解析协议级别
    if input.is_empty() {
        return Err("Insufficient data for protocol level".to_string());
    }
    let protocol_level = input.get_u8();
    
    // 解析连接标志
    if input.is_empty() {
        return Err("Insufficient data for connect flags".to_string());
    }
    let connect_flags = input.get_u8();
    
    // 解析保活时间
    if input.len() < 2 {
        return Err("Insufficient data for keep alive".to_string());
    }
    let keep_alive = input.get_u16();
    
    // 解析客户端标识符
    let client_id = parse_mqtt_string(input)?;
    
    // 解析可选字段
    let mut will_topic = None;
    let mut will_message = None;
    let mut username = None;
    let mut password = None;
    
    // 检查遗嘱标志
    if (connect_flags & 0x04) != 0 {
        // 解析遗嘱主题
        let topic = parse_mqtt_string(input)?;
        will_topic = Some(topic);
        
        // 解析遗嘱消息
        let message = parse_mqtt_bytes(input)?;
        will_message = Some(message);
    }
    
    // 检查用户名标志
    if (connect_flags & 0x80) != 0 {
        let user = parse_mqtt_string(input)?;
        username = Some(user);
    }
    
    // 检查密码标志
    if (connect_flags & 0x40) != 0 {
        let pass = parse_mqtt_bytes(input)?;
        password = Some(pass);
    }
    
    Ok(ConnectPacket {
        protocol_name,
        protocol_level,
        connect_flags,
        keep_alive,
        client_id,
        will_topic,
        will_message,
        username,
        password,
    })
}

impl ConnectPacket {
    /// 从BytesMut读取并解析为ConnectPacket
    pub fn from_bytes(bytes: &mut std::slice::Iter<u8>) -> Result<Self, String> {
        // 注意：此方法已被废弃，建议使用parse_connect函数
        Err("from_bytes method is deprecated, use parse_connect instead".to_string())
    }
}
