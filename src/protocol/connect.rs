use super::Packet;
use super::parse_mqtt_bytes;
use super::parse_mqtt_string;
use super::write_mqtt_bytes;
use super::write_mqtt_string;
use super::write_remaining_length;
use bytes::{Buf, BufMut, Bytes, BytesMut};

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




impl Packet for ConnectPacket {
    /// 将CONNECT数据包序列化为字节并写入缓冲区
    fn write(&self, buf: &mut BytesMut) {
        // 计算可变头和载荷长度
        let mut variable_header_length = 0;
        
        // 协议名称长度
        variable_header_length += 2 + self.protocol_name.len();
        // 协议级别长度
        variable_header_length += 1;
        // 连接标志长度
        variable_header_length += 1;
        // 保活时间长度
        variable_header_length += 2;
        
        // 载荷长度
        let mut payload_length = 0;
        
        // 客户端标识符长度
        payload_length += 2 + self.client_id.len();
        
        // 遗嘱主题和遗嘱消息长度（如果有）
        if let Some(topic) = &self.will_topic {
            payload_length += 2 + topic.len();
        }
        if let Some(message) = &self.will_message {
            payload_length += 2 + message.len();
        }
        
        // 用户名长度（如果有）
        if let Some(username) = &self.username {
            payload_length += 2 + username.len();
        }
        
        // 密码长度（如果有）
        if let Some(password) = &self.password {
            payload_length += 2 + password.len();
        }
        
        // 总剩余长度
        let remaining_length = variable_header_length + payload_length;
        
        // 写入固定头
        let packet_type = 1; // CONNECT
        let flags = 0x00; // CONNECT固定标志位为0x00
        let first_byte = (packet_type << 4) | flags;
        buf.put_u8(first_byte);
        
        // 写入剩余长度
        write_remaining_length(buf, remaining_length);
        
        // 写入可变头
        // 协议名称
        write_mqtt_string(buf, &self.protocol_name);
        // 协议级别
        buf.put_u8(self.protocol_level);
        // 连接标志
        buf.put_u8(self.connect_flags);
        // 保活时间
        buf.put_u16(self.keep_alive);
        
        // 写入载荷
        // 客户端标识符
        write_mqtt_string(buf, &self.client_id);
        
        // 遗嘱主题和遗嘱消息（如果有）
        if let Some(topic) = &self.will_topic {
            write_mqtt_string(buf, topic);
        }
        if let Some(message) = &self.will_message {
            write_mqtt_bytes(buf, message);
        }
        
        // 用户名（如果有）
        if let Some(username) = &self.username {
            write_mqtt_string(buf, username);
        }
        
        // 密码（如果有）
        if let Some(password) = &self.password {
            write_mqtt_bytes(buf, password);
        }
    }
    
    /// 从BytesMut解析CONNECT数据包
    fn parse(input: &mut BytesMut, _flags: Option<u8>) -> Result<Self, String> {
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
}


