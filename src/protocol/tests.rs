use super::*;
use bytes::{Bytes, BytesMut};

#[test]
fn test_read_from_bytes_mut() {
    // 测试CONNECT数据包
    // 剩余长度计算：协议名称(6) + 协议级别(1) + 连接标志(1) + 保持连接(2) + 客户端ID(6) = 16 = 0x10
    let connect_packet = b"\x10\x10\x00\x04MQTT\x04\x02\x00\x3c\x00\x04test";
    let mut buf = BytesMut::from(&connect_packet[..]);
    
    let packet = read_from_bytes_mut(&mut buf).expect("Failed to read packet");
    assert_eq!(packet.len(), connect_packet.len());
    assert_eq!(packet, Bytes::from_static(connect_packet));
    assert!(buf.is_empty());
}

#[test]
fn test_read_from_bytes_mut_partial() {
    // 测试部分数据包
    // 剩余长度计算：协议名称(6) + 协议级别(1) + 连接标志(1) + 保持连接(2) + 客户端ID(6) = 16 = 0x10
    let connect_packet = b"\x10\x10\x00\x04MQTT\x04\x02\x00\x3c\x00\x04test";
    let mut buf = BytesMut::new();
    
    // 先写入部分数据
    buf.extend_from_slice(&connect_packet[..5]);
    assert!(read_from_bytes_mut(&mut buf).is_none());
    
    // 再写入剩余数据
    buf.extend_from_slice(&connect_packet[5..]);
    let packet = read_from_bytes_mut(&mut buf).expect("Failed to read packet");
    assert_eq!(packet.len(), connect_packet.len());
    assert_eq!(packet, Bytes::from_static(connect_packet));
}

#[test]
fn test_parse_fixed_header() {
    // 测试固定头解析
    let data = b"\x10\x13";
    let result = parse_fixed_header(data);
    assert!(result.is_ok());
    
    let (_, header) = result.unwrap();
    assert_eq!(header.packet_type, PacketType::Connect);
    assert_eq!(header.flags, 0x00);
    assert_eq!(header.remaining_length, 0x13);
}

#[test]
fn test_parse_connect() {
    // 测试CONNECT数据包解析
    let data = b"\x00\x04MQTT\x04\x02\x00\x3c\x00\x04test";
    let result = parse_connect(data);
    assert!(result.is_ok());
    
    let (_, packet) = result.unwrap();
    assert_eq!(packet.protocol_name, "MQTT");
    assert_eq!(packet.protocol_level, 0x04);
    assert_eq!(packet.connect_flags, 0x02);
    assert_eq!(packet.keep_alive, 0x003c);
    assert_eq!(packet.client_id, "test");
    assert!(packet.will_topic.is_none());
    assert!(packet.will_message.is_none());
    assert!(packet.username.is_none());
    assert!(packet.password.is_none());
}

#[test]
fn test_parse_publish() {
    // 测试PUBLISH数据包解析
    let data = b"\x00\x04testHello";
    let result = parse_publish(data, 0x00);
    assert!(result.is_ok());
    
    let (_, packet) = result.unwrap();
    assert_eq!(packet.dup, false);
    assert_eq!(packet.qos, 0);
    assert_eq!(packet.retain, false);
    assert_eq!(packet.topic_name, "test");
    assert!(packet.packet_id.is_none());
    assert_eq!(packet.payload, Bytes::from_static(b"Hello"));
}

#[test]
fn test_parse_publish_with_packet_id() {
    // 测试带packet_id的PUBLISH数据包解析
    let data = b"\x00\x04test\x00\x01Hello";
    let result = parse_publish(data, 0x02); // QoS 1
    assert!(result.is_ok());
    
    let (_, packet) = result.unwrap();
    assert_eq!(packet.dup, false);
    assert_eq!(packet.qos, 1);
    assert_eq!(packet.retain, false);
    assert_eq!(packet.topic_name, "test");
    assert_eq!(packet.packet_id, Some(1));
    assert_eq!(packet.payload, Bytes::from_static(b"Hello"));
}

#[test]
fn test_parse_puback() {
    // 测试PUBACK数据包解析
    let data = b"\x00\x01";
    let result = parse_puback(data);
    assert!(result.is_ok());
    
    let (_, packet) = result.unwrap();
    assert_eq!(packet.packet_id, 0x0001);
}

#[test]
fn test_parse_subscribe() {
    // 测试SUBSCRIBE数据包解析
    let data = b"\x00\x01\x00\x04test\x01";
    let result = parse_subscribe(data);
    assert!(result.is_ok());
    
    let (_, packet) = result.unwrap();
    assert_eq!(packet.packet_id, 0x0001);
    assert_eq!(packet.topics.len(), 1);
    assert_eq!(packet.topics[0].0, "test");
    assert_eq!(packet.topics[0].1, 0x01);
}

#[test]
fn test_parse_suback() {
    // 测试SUBACK数据包解析
    let data = b"\x00\x01\x00";
    let result = parse_suback(data);
    assert!(result.is_ok());
    
    let (_, packet) = result.unwrap();
    assert_eq!(packet.packet_id, 0x0001);
    assert_eq!(packet.return_codes.len(), 1);
    assert_eq!(packet.return_codes[0], 0x00);
}

#[test]
fn test_parse_unsubscribe() {
    // 测试UNSUBSCRIBE数据包解析
    let data = b"\x00\x01\x00\x04test";
    let result = parse_unsubscribe(data);
    assert!(result.is_ok());
    
    let (_, packet) = result.unwrap();
    assert_eq!(packet.packet_id, 0x0001);
    assert_eq!(packet.topics.len(), 1);
    assert_eq!(packet.topics[0], "test");
}

#[test]
fn test_parse_unsuback() {
    // 测试UNSUBACK数据包解析
    let data = b"\x00\x01";
    let result = parse_unsuback(data);
    assert!(result.is_ok());
    
    let (_, packet) = result.unwrap();
    assert_eq!(packet.packet_id, 0x0001);
}

#[test]
fn test_parse_ping_req() {
    // 测试PINGREQ数据包解析
    let data = b"";
    let result = parse_ping_req(data);
    assert!(result.is_ok());
}

#[test]
fn test_parse_ping_resp() {
    // 测试PINGRESP数据包解析
    let data = b"";
    let result = parse_ping_resp(data);
    assert!(result.is_ok());
}

#[test]
fn test_parse_disconnect() {
    // 测试DISCONNECT数据包解析
    let data = b"";
    let result = parse_disconnect(data);
    assert!(result.is_ok());
}

#[test]
fn test_serialize_packet() {
    // 测试序列化CONNECT数据包
    let connect_packet = ConnectPacket {
        protocol_name: "MQTT".to_string(),
        protocol_level: 4,
        connect_flags: 2,
        keep_alive: 60,
        client_id: "test".to_string(),
        will_topic: None,
        will_message: None,
        username: None,
        password: None,
    };
    
    let packet = MqttPacket::Connect(connect_packet);
    let serialized = serialize_packet(&packet);
    
    // 验证序列化结果
    assert!(!serialized.is_empty());
    assert_eq!(serialized[0], 0x10); // CONNECT类型
}
