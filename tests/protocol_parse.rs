use mqtt_adapt::protocol::{MqttPacket, Packet, FixedHeader, PacketType};
use mqtt_adapt::protocol::{ConnectPacket, ConnAckPacket, PublishPacket, PubAckPacket, PubRecPacket, PubRelPacket, PubCompPacket, SubscribePacket, SubAckPacket, UnsubscribePacket, UnsubAckPacket, PingReqPacket, PingRespPacket, DisconnectPacket};
use bytes::{BytesMut, Bytes, BufMut};

// 测试固定头解析
#[test]
fn test_fixed_header_parse(){
    let mut buffer = BytesMut::new();
    // 写入CONNECT数据包的固定头: 0x10 (类型) + 0x04 (剩余长度)
    buffer.put_u8(0x10);
    buffer.put_u8(0x04);
    
    let result = FixedHeader::parse(&mut buffer);
    assert!(result.is_ok());
    let header = result.unwrap();
    assert_eq!(header.packet_type, PacketType::Connect);
    assert_eq!(header.flags, 0x00);
    assert_eq!(header.remaining_length, 4);
}

// 测试CONNECT数据包解析
#[test]
fn test_connect_packet_parse() {
    let mut buffer = BytesMut::new();
    // 写入MQTT协议名称和版本
    buffer.put_u16(4); // 协议名称长度
    buffer.put_slice(b"MQTT"); // 协议名称
    buffer.put_u8(4); // 协议级别
    buffer.put_u8(0x02); // 连接标志
    buffer.put_u16(60); // 保持连接时间
    buffer.put_u16(11); // 客户端ID长度
    buffer.put_slice(b"test_client"); // 客户端ID
    
    let result = ConnectPacket::parse(&mut buffer, None);
    assert!(result.is_ok());
    let packet = result.unwrap();
    assert_eq!(packet.protocol_name, "MQTT");
    assert_eq!(packet.protocol_level, 4);
    assert_eq!(packet.client_id, "test_client");
}

// 测试CONNACK数据包解析
#[test]
fn test_connack_packet_parse() {
    let mut buffer = BytesMut::new();
    // 写入CONNACK标志和返回码
    buffer.put_u8(0x00); // 会话存在标志
    buffer.put_u8(0x00); // 返回码: 接受
    
    let result = ConnAckPacket::parse(&mut buffer, None);
    assert!(result.is_ok());
    let packet = result.unwrap();
    assert!(!packet.session_present);
    assert_eq!(packet.return_code as u8, 0x00);
}

// 测试PUBLISH数据包解析
#[test]
fn test_publish_packet_parse() {
    let mut buffer = BytesMut::new();
    // 写入主题名和载荷
    buffer.put_u16(10); // 主题名长度
    buffer.put_slice(b"test/topic"); // 主题名
    buffer.put_slice(b"test payload"); // 载荷
    
    let result = PublishPacket::parse(&mut buffer, Some(0x00)); // QoS 0
    assert!(result.is_ok());
    let packet = result.unwrap();
    assert_eq!(packet.topic_name, "test/topic");
    assert_eq!(packet.payload, Bytes::from_static(b"test payload"));
    assert_eq!(packet.qos, 0);
}

// 测试PUBACK数据包解析
#[test]
fn test_puback_packet_parse() {
    let mut buffer = BytesMut::new();
    // 写入数据包ID
    buffer.put_u16(1234);
    
    let result = PubAckPacket::parse(&mut buffer, None);
    assert!(result.is_ok());
    let packet = result.unwrap();
    assert_eq!(packet.packet_id, 1234);
}

// 测试PUBREC数据包解析
#[test]
fn test_pubrec_packet_parse() {
    let mut buffer = BytesMut::new();
    // 写入数据包ID
    buffer.put_u16(5678);
    
    let result = PubRecPacket::parse(&mut buffer, None);
    assert!(result.is_ok());
    let packet = result.unwrap();
    assert_eq!(packet.packet_id, 5678);
}

// 测试PUBREL数据包解析
#[test]
fn test_pubrel_packet_parse() {
    let mut buffer = BytesMut::new();
    // 写入数据包ID
    buffer.put_u16(9012);
    
    let result = PubRelPacket::parse(&mut buffer, None);
    assert!(result.is_ok());
    let packet = result.unwrap();
    assert_eq!(packet.packet_id, 9012);
}

// 测试PUBCOMP数据包解析
#[test]
fn test_pubcomp_packet_parse() {
    let mut buffer = BytesMut::new();
    // 写入数据包ID
    buffer.put_u16(3456);
    
    let result = PubCompPacket::parse(&mut buffer, None);
    assert!(result.is_ok());
    let packet = result.unwrap();
    assert_eq!(packet.packet_id, 3456);
}

// 测试SUBSCRIBE数据包解析
#[test]
fn test_subscribe_packet_parse() {
    let mut buffer = BytesMut::new();
    // 写入数据包ID
    buffer.put_u16(7890);
    // 写入主题过滤器和QoS
    buffer.put_u16(10); // 主题长度
    buffer.put_slice(b"test/topic"); // 主题
    buffer.put_u8(0x01); // QoS级别
    
    let result = SubscribePacket::parse(&mut buffer, None);
    assert!(result.is_ok());
    let packet = result.unwrap();
    assert_eq!(packet.packet_id, 7890);
    assert_eq!(packet.topics.len(), 1);
    assert_eq!(packet.topics[0].0, "test/topic");
    assert_eq!(packet.topics[0].1, 1);
}

// 测试SUBACK数据包解析
#[test]
fn test_suback_packet_parse() {
    let mut buffer = BytesMut::new();
    // 写入数据包ID和返回码
    buffer.put_u16(1111);
    buffer.put_u8(0x01); // QoS级别
    
    let result = SubAckPacket::parse(&mut buffer, None);
    assert!(result.is_ok());
    let packet = result.unwrap();
    assert_eq!(packet.packet_id, 1111);
    assert_eq!(packet.return_codes, 1);
}

// 测试UNSUBSCRIBE数据包解析
#[test]
fn test_unsubscribe_packet_parse() {
    let mut buffer = BytesMut::new();
    // 写入数据包ID
    buffer.put_u16(2222);
    // 写入主题过滤器
    buffer.put_u16(10); // 主题长度
    buffer.put_slice(b"test/topic"); // 主题
    
    let result = UnsubscribePacket::parse(&mut buffer, None);
    assert!(result.is_ok());
    let packet = result.unwrap();
    assert_eq!(packet.packet_id, 2222);
    assert_eq!(packet.topics.len(), 1);
    assert_eq!(packet.topics[0], "test/topic");
}

// 测试UNSUBACK数据包解析
#[test]
fn test_unsuback_packet_parse() {
    let mut buffer = BytesMut::new();
    // 写入数据包ID
    buffer.put_u16(3333);
    
    let result = UnsubAckPacket::parse(&mut buffer, None);
    assert!(result.is_ok());
    let packet = result.unwrap();
    assert_eq!(packet.packet_id, 3333);
}

// 测试PINGREQ数据包解析
#[test]
fn test_pingreq_packet_parse() {
    let mut buffer = BytesMut::new();
    // PINGREQ没有可变头和载荷
    
    let result = PingReqPacket::parse(&mut buffer, None);
    assert!(result.is_ok());
}

// 测试PINGRESP数据包解析
#[test]
fn test_pingresp_packet_parse() {
    let mut buffer = BytesMut::new();
    // PINGRESP没有可变头和载荷
    
    let result = PingRespPacket::parse(&mut buffer, None);
    assert!(result.is_ok());
}

// 测试DISCONNECT数据包解析
#[test]
fn test_disconnect_packet_parse() {
    let mut buffer = BytesMut::new();
    // DISCONNECT没有可变头和载荷
    
    let result = DisconnectPacket::parse(&mut buffer, None);
    assert!(result.is_ok());
}

// 测试数据包序列化和反序列化
#[test]
fn test_packet_serialization() {
    // 测试CONNECT数据包序列化
    let connect_packet = ConnectPacket {
        protocol_name: "MQTT".to_string(),
        protocol_level: 4,
        connect_flags: 0x02,
        keep_alive: 60,
        client_id: "test_client".to_string(),
        will_topic: None,
        will_message: None,
        username: None,
        password: None,
    };
    
    let mut buffer = BytesMut::new();
    connect_packet.write(&mut buffer);
    assert!(!buffer.is_empty());
    
    // 测试CONNACK数据包序列化
    let connack_packet = ConnAckPacket {
        session_present: false,
        return_code: mqtt_adapt::protocol::ConnectReturnCode::Accepted,
    };
    
    let mut buffer = BytesMut::new();
    connack_packet.write(&mut buffer);
    assert!(!buffer.is_empty());
    
    // 测试PUBLISH数据包序列化
    let publish_packet = PublishPacket {
        dup: false,
        qos: 0,
        retain: false,
        topic_name: "test/topic".to_string(),
        packet_id: None,
        payload: Bytes::from_static(b"test payload"),
    };
    
    let mut buffer = BytesMut::new();
    publish_packet.write(&mut buffer);
    assert!(!buffer.is_empty());
}

// 测试MqttPacket::read方法
#[test]
fn test_mqtt_packet_read() {
    let mut buffer = BytesMut::new();
    // 写入CONNECT数据包
    buffer.put_u8(0x10); // 固定头: CONNECT类型
    buffer.put_u8(0x17); // 剩余长度 (23字节)
    buffer.put_u16(4); // 协议名称长度
    buffer.put_slice(b"MQTT"); // 协议名称
    buffer.put_u8(4); // 协议级别
    buffer.put_u8(0x02); // 连接标志
    buffer.put_u16(60); // 保持连接时间
    buffer.put_u16(11); // 客户端ID长度
    buffer.put_slice(b"test_client"); // 客户端ID
    
    let result = MqttPacket::read(&mut buffer);
    assert!(result.is_ok());
    match result.unwrap() {
        MqttPacket::Connect(_) => {
            // 验证是CONNECT数据包
        }
        _ => {
            panic!("Expected CONNECT packet");
        }
    }
}
