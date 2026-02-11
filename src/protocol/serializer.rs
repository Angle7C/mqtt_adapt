use bytes::Bytes;

/// 序列化剩余长度
pub fn serialize_remaining_length(length: usize) -> Vec<u8> {
    let mut result = Vec::new();
    let mut value = length;
    
    loop {
        let mut byte = (value % 128) as u8;
        value /= 128;
        
        if value > 0 {
            byte |= 0x80;
        }
        
        result.push(byte);
        
        if value == 0 {
            break;
        }
    }
    
    result
}

/// 序列化MQTT字符串
pub fn serialize_mqtt_string(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let length = s.len() as u16;
    result.extend_from_slice(&length.to_be_bytes());
    result.extend_from_slice(s.as_bytes());
    result
}

/// 序列化MQTT二进制数据
pub fn serialize_mqtt_binary(data: &Bytes) -> Vec<u8> {
    let mut result = Vec::new();
    let length = data.len() as u16;
    result.extend_from_slice(&length.to_be_bytes());
    result.extend_from_slice(data);
    result
}

/// 序列化固定头
pub fn serialize_fixed_header(packet_type: super::PacketType, flags: u8, remaining_length: usize) -> Vec<u8> {
    let mut result = Vec::new();
    let first_byte = ((packet_type as u8) << 4) | (flags & 0x0F);
    result.push(first_byte);
    result.extend_from_slice(&serialize_remaining_length(remaining_length));
    result
}

/// 序列化MQTT数据包
pub fn serialize_packet(packet: &super::MqttPacket) -> Bytes {
    let mut result = Vec::new();
    
    match packet {
        super::MqttPacket::Connect(connect) => {
            let mut variable_header = Vec::new();
            variable_header.extend_from_slice(&serialize_mqtt_string(&connect.protocol_name));
            variable_header.push(connect.protocol_level);
            variable_header.push(connect.connect_flags);
            variable_header.extend_from_slice(&connect.keep_alive.to_be_bytes());
            variable_header.extend_from_slice(&serialize_mqtt_string(&connect.client_id));
            
            if let Some(will_topic) = &connect.will_topic {
                variable_header.extend_from_slice(&serialize_mqtt_string(will_topic));
            }
            
            if let Some(will_message) = &connect.will_message {
                variable_header.extend_from_slice(&serialize_mqtt_binary(will_message));
            }
            
            if let Some(username) = &connect.username {
                variable_header.extend_from_slice(&serialize_mqtt_string(username));
            }
            
            if let Some(password) = &connect.password {
                variable_header.extend_from_slice(&serialize_mqtt_binary(password));
            }
            
            let fixed_header = serialize_fixed_header(super::PacketType::Connect, 0, variable_header.len());
            result.extend_from_slice(&fixed_header);
            result.extend_from_slice(&variable_header);
        }
        
        super::MqttPacket::ConnAck(connack) => {
            let mut variable_header = Vec::new();
            variable_header.push(if connack.session_present { 1 } else { 0 });
            variable_header.push(connack.return_code as u8);
            
            let fixed_header = serialize_fixed_header(super::PacketType::ConnAck, 0, variable_header.len());
            result.extend_from_slice(&fixed_header);
            result.extend_from_slice(&variable_header);
        }
        
        super::MqttPacket::Publish(publish) => {
            let mut variable_header = Vec::new();
            variable_header.extend_from_slice(&serialize_mqtt_string(&publish.topic_name));
            
            if let Some(packet_id) = publish.packet_id {
                variable_header.extend_from_slice(&packet_id.to_be_bytes());
            }
            
            let payload = publish.payload.to_vec();
            let total_length = variable_header.len() + payload.len();
            
            let mut flags = 0;
            if publish.dup {
                flags |= 0x08;
            }
            flags |= (publish.qos << 1) & 0x06;
            if publish.retain {
                flags |= 0x01;
            }
            
            let fixed_header = serialize_fixed_header(super::PacketType::Publish, flags, total_length);
            result.extend_from_slice(&fixed_header);
            result.extend_from_slice(&variable_header);
            result.extend_from_slice(&payload);
        }
        
        super::MqttPacket::PubAck(puback) => {
            let mut variable_header = Vec::new();
            variable_header.extend_from_slice(&puback.packet_id.to_be_bytes());
            
            let fixed_header = serialize_fixed_header(super::PacketType::PubAck, 0, variable_header.len());
            result.extend_from_slice(&fixed_header);
            result.extend_from_slice(&variable_header);
        }
        
        super::MqttPacket::PubRec(pubrec) => {
            let mut variable_header = Vec::new();
            variable_header.extend_from_slice(&pubrec.packet_id.to_be_bytes());
            
            let fixed_header = serialize_fixed_header(super::PacketType::PubRec, 0, variable_header.len());
            result.extend_from_slice(&fixed_header);
            result.extend_from_slice(&variable_header);
        }
        
        super::MqttPacket::PubRel(pubrel) => {
            let mut variable_header = Vec::new();
            variable_header.extend_from_slice(&pubrel.packet_id.to_be_bytes());
            
            let fixed_header = serialize_fixed_header(super::PacketType::PubRel, 0x02, variable_header.len());
            result.extend_from_slice(&fixed_header);
            result.extend_from_slice(&variable_header);
        }
        
        super::MqttPacket::PubComp(pubcomp) => {
            let mut variable_header = Vec::new();
            variable_header.extend_from_slice(&pubcomp.packet_id.to_be_bytes());
            
            let fixed_header = serialize_fixed_header(super::PacketType::PubComp, 0, variable_header.len());
            result.extend_from_slice(&fixed_header);
            result.extend_from_slice(&variable_header);
        }
        
        super::MqttPacket::Subscribe(subscribe) => {
            let mut variable_header = Vec::new();
            variable_header.extend_from_slice(&subscribe.packet_id.to_be_bytes());
            
            for (topic, qos) in &subscribe.topics {
                variable_header.extend_from_slice(&serialize_mqtt_string(topic));
                variable_header.push(*qos);
            }
            
            let fixed_header = serialize_fixed_header(super::PacketType::Subscribe, 0x02, variable_header.len());
            result.extend_from_slice(&fixed_header);
            result.extend_from_slice(&variable_header);
        }
        
        super::MqttPacket::SubAck(suback) => {
            let mut variable_header = Vec::new();
            variable_header.extend_from_slice(&suback.packet_id.to_be_bytes());
            variable_header.extend_from_slice(&suback.return_codes);
            
            let fixed_header = serialize_fixed_header(super::PacketType::SubAck, 0, variable_header.len());
            result.extend_from_slice(&fixed_header);
            result.extend_from_slice(&variable_header);
        }
        
        super::MqttPacket::Unsubscribe(unsubscribe) => {
            let mut variable_header = Vec::new();
            variable_header.extend_from_slice(&unsubscribe.packet_id.to_be_bytes());
            
            for topic in &unsubscribe.topics {
                variable_header.extend_from_slice(&serialize_mqtt_string(topic));
            }
            
            let fixed_header = serialize_fixed_header(super::PacketType::Unsubscribe, 0x02, variable_header.len());
            result.extend_from_slice(&fixed_header);
            result.extend_from_slice(&variable_header);
        }
        
        super::MqttPacket::UnsubAck(unsuback) => {
            let mut variable_header = Vec::new();
            variable_header.extend_from_slice(&unsuback.packet_id.to_be_bytes());
            
            let fixed_header = serialize_fixed_header(super::PacketType::UnsubAck, 0, variable_header.len());
            result.extend_from_slice(&fixed_header);
            result.extend_from_slice(&variable_header);
        }
        
        super::MqttPacket::PingReq => {
            let fixed_header = serialize_fixed_header(super::PacketType::PingReq, 0, 0);
            result.extend_from_slice(&fixed_header);
        }
        
        super::MqttPacket::PingResp => {
            let fixed_header = serialize_fixed_header(super::PacketType::PingResp, 0, 0);
            result.extend_from_slice(&fixed_header);
        }
        
        super::MqttPacket::Disconnect => {
            let fixed_header = serialize_fixed_header(super::PacketType::Disconnect, 0, 0);
            result.extend_from_slice(&fixed_header);
        }
    }
    
    Bytes::from(result)
}
