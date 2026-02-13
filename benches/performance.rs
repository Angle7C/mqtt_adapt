use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mqtt_adapt::topic::TopicManager;
use mqtt_adapt::routing::{router::MessageRouter, event::Event};
use mqtt_adapt::protocol::{MqttPacket, SubscribePacket, PublishPacket, Packet};
use bytes::{BytesMut, Bytes, BufMut};
use flume::{unbounded};
use tokio::runtime::Runtime;

// 测试主题管理器性能
fn bench_topic_manager(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("topic_manager_add_subscription", |b| {
        b.iter(|| {
            let topic_manager = TopicManager::new();
            rt.block_on(async {
                for i in 0..100 {
                    let client_id = format!("client_{}", i);
                    let topic = format!("sensor.{}.data", i % 10);
                    topic_manager.add_subscription(&topic, client_id, 1).await;
                }
            });
        });
    });
    
    c.bench_function("topic_manager_find_subscribers", |b| {
        let topic_manager = TopicManager::new();
        // 先添加一些订阅
        rt.block_on(async {
            for i in 0..100 {
                let client_id = format!("client_{}", i);
                let topic = format!("sensor.{}.data", i % 10);
                topic_manager.add_subscription(&topic, client_id, 1).await;
            }
        });
        
        b.iter(|| {
            rt.block_on(async {
                for i in 0..10 {
                    let topic = format!("sensor.{}.data", i);
                    topic_manager.find_subscribers(&topic).await;
                }
            });
        });
    });
    
    c.bench_function("topic_manager_remove_subscription", |b| {
        let topic_manager = TopicManager::new();
        // 先添加一些订阅
        rt.block_on(async {
            for i in 0..100 {
                let client_id = format!("client_{}", i);
                let topic = format!("sensor.{}.data", i % 10);
                topic_manager.add_subscription(&topic, client_id.clone(), 1).await;
            }
        });
        
        b.iter(|| {
            rt.block_on(async {
                for i in 0..100 {
                    let client_id = format!("client_{}", i);
                    let topic = format!("sensor.{}.data", i % 10);
                    topic_manager.remove_subscription(&topic, &client_id).await;
                }
            });
        });
    });
}

// 测试消息路由器性能
fn bench_message_router(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("message_router_handle_event", |b| {
        let router = MessageRouter::new();
        let client_id = "test_client".to_string();
        let (tx, _rx) = unbounded();
        
        rt.block_on(async {
            router.register_client(client_id.clone(), tx).await.unwrap();
        });
        
        b.iter(|| {
            rt.block_on(async {
                // 每次迭代创建新的数据包和事件
                let subscribe_packet = SubscribePacket {
                    packet_id: 1,
                    topics: vec![("test/topic".to_string(), 1)],
                };
                let event = Event::MessageReceived(client_id.clone(), MqttPacket::Subscribe(subscribe_packet));
                router.handle_event(black_box(event)).await;
            });
        });
    });
    
    c.bench_function("message_router_publish", |b| {
        let router = MessageRouter::new();
        let publisher_id = "publisher".to_string();
        let subscriber_id = "subscriber".to_string();
        let (publisher_tx, _publisher_rx) = unbounded();
        let (subscriber_tx, _subscriber_rx) = unbounded();
        
        rt.block_on(async {
            router.register_client(publisher_id.clone(), publisher_tx).await.unwrap();
            router.register_client(subscriber_id.clone(), subscriber_tx).await.unwrap();
            
            // 先让订阅者订阅主题
            let subscribe_packet = SubscribePacket {
                packet_id: 1,
                topics: vec![("test/topic".to_string(), 1)],
            };
            let subscribe_event = Event::MessageReceived(subscriber_id.clone(), MqttPacket::Subscribe(subscribe_packet));
            router.handle_event(subscribe_event).await;
        });
        
        b.iter(|| {
            rt.block_on(async {
                // 每次迭代创建新的数据包和事件
                let publish_packet = PublishPacket {
                    dup: false,
                    qos: 0,
                    retain: false,
                    topic_name: "test/topic".to_string(),
                    packet_id: None,
                    payload: Bytes::from_static(b"test payload"),
                };
                let publish_event = Event::MessageReceived(publisher_id.clone(), MqttPacket::Publish(publish_packet));
                router.handle_event(black_box(publish_event)).await;
            });
        });
    });
}

// 测试协议解析性能
fn bench_protocol_parse(c: &mut Criterion) {
    c.bench_function("protocol_parse_connect", |b| {
        let mut buffer = BytesMut::new();
        buffer.put_u16(4); // 协议名称长度
        buffer.put_slice(b"MQTT"); // 协议名称
        buffer.put_u8(4); // 协议级别
        buffer.put_u8(0x02); // 连接标志
        buffer.put_u16(60); // 保持连接时间
        buffer.put_u16(11); // 客户端ID长度
        buffer.put_slice(b"test_client"); // 客户端ID
        
        b.iter(|| {
            let mut buf = buffer.clone();
            let _ = mqtt_adapt::protocol::ConnectPacket::parse(&mut buf, None);
        });
    });
    
    c.bench_function("protocol_parse_publish", |b| {
        let mut buffer = BytesMut::new();
        buffer.put_u16(10); // 主题名长度
        buffer.put_slice(b"test/topic"); // 主题名
        buffer.put_slice(b"test payload"); // 载荷
        
        b.iter(|| {
            let mut buf = buffer.clone();
            let _ = mqtt_adapt::protocol::PublishPacket::parse(&mut buf, Some(0x00));
        });
    });
    
    c.bench_function("protocol_serialize_publish", |b| {
        let publish_packet = mqtt_adapt::protocol::PublishPacket {
            dup: false,
            qos: 0,
            retain: false,
            topic_name: "test/topic".to_string(),
            packet_id: None,
            payload: Bytes::from_static(b"test payload"),
        };
        
        b.iter(|| {
            let mut buffer = BytesMut::new();
            publish_packet.write(&mut buffer);
        });
    });
}

criterion_group!(benches, bench_topic_manager, bench_message_router, bench_protocol_parse);
criterion_main!(benches);
