use mqtt_adapt::routing::{router::MessageRouter, event::Event}; 
use mqtt_adapt::protocol::{MqttPacket, SubscribePacket, UnsubscribePacket, PublishPacket};
use flume::{unbounded};
use bytes::Bytes;

#[tokio::test]
async fn test_message_router_basic() {
    // 创建新的消息路由器
    let router = MessageRouter::new();
    
    // 验证路由器创建成功
    let _sender = router.get_sender();
}

#[tokio::test]
async fn test_client_registration() {
    let router = MessageRouter::new();
    let client_id = "test_client_1".to_string();
    let (tx, _rx) = unbounded();
    
    // 注册客户端
    let result = router.register_client(&client_id.clone(), tx).await;
    assert!(result.is_ok());
    
    // 移除客户端
    router.remove_client(&client_id).await;
}

#[tokio::test]
async fn test_client_connected_event() {
    let router = MessageRouter::new();
    let client_id = "test_client_2".to_string();
    let (tx, rx) = unbounded();
    
    // 注册客户端
    router.register_client(&client_id.clone(), tx).await.unwrap();
    
    // 发送客户端连接事件
    let event = Event::ClientConnected(client_id.clone());
    router.handle_event(event).await;
    
    // 检查是否收到CONNACK消息
    if let Ok(Event::MessageSent(recv_client_id, packet)) = rx.try_recv() {
        assert_eq!(recv_client_id, client_id);
        match packet {
            MqttPacket::ConnAck(_) => {
                // 验证是CONNACK数据包
            }
            _ => {
                panic!("Expected CONNACK packet, got {:?}", packet);
            }
        }
    } else {
        panic!("Expected MessageSent event");
    }
}

#[tokio::test]
async fn test_subscribe_event() {
    let router = MessageRouter::new();
    let client_id = "test_client_3".to_string();
    let (tx, rx) = unbounded();
    
    // 注册客户端
    router.register_client(&client_id.clone(), tx).await.unwrap();
    
    // 创建订阅数据包
    let topics = vec![("test/topic".to_string(), 0)];
    let subscribe_packet = SubscribePacket {
        packet_id: 1,
        topics,
    };
    
    // 发送订阅事件
    let event = Event::MessageReceived(client_id.clone(), MqttPacket::Subscribe(subscribe_packet));
    router.handle_event(event).await;
    
    // 检查是否收到SUBACK消息
    if let Ok(Event::MessageSent(recv_client_id, packet)) = rx.try_recv() {
        assert_eq!(recv_client_id, client_id);
        match packet {
            MqttPacket::SubAck(_) => {
                // 验证是SUBACK数据包
            }
            _ => {
                panic!("Expected SUBACK packet, got {:?}", packet);
            }
        }
    } else {
        panic!("Expected MessageSent event");
    }
}

#[tokio::test]
async fn test_unsubscribe_event() {
    let router = MessageRouter::new();
    let client_id = "test_client_4".to_string();
    let (tx, rx) = unbounded();
    
    // 注册客户端
    router.register_client(&client_id.clone(), tx).await.unwrap();
    
    // 创建取消订阅数据包
    let topics = vec!["test/topic".to_string()];
    let unsubscribe_packet = UnsubscribePacket {
        packet_id: 1,
        topics,
    };
    
    // 发送取消订阅事件
    let event = Event::MessageReceived(client_id.clone(), MqttPacket::Unsubscribe(unsubscribe_packet));
    router.handle_event(event).await;
    
    // 检查是否收到UNSUBACK消息
    if let Ok(Event::MessageSent(recv_client_id, packet)) = rx.try_recv() {
        assert_eq!(recv_client_id, client_id);
        match packet {
            MqttPacket::UnsubAck(_) => {
                // 验证是UNSUBACK数据包
            }
            _ => {
                panic!("Expected UNSUBACK packet, got {:?}", packet);
            }
        }
    } else {
        panic!("Expected MessageSent event");
    }
}

#[tokio::test]
async fn test_publish_event() {
    let router = MessageRouter::new();
    
    // 注册发布者客户端
    let publisher_id = "publisher".to_string();
    let (publisher_tx, _publisher_rx) = unbounded();
    router.register_client(&publisher_id.clone(), publisher_tx).await.unwrap();
    
    // 注册订阅者客户端
    let subscriber_id = "subscriber".to_string();
    let (subscriber_tx, subscriber_rx) = unbounded();
    router.register_client(&subscriber_id.clone(), subscriber_tx).await.unwrap();
    
    // 首先让订阅者订阅主题
    let subscribe_topics = vec![("test/topic".to_string(), 0)];
    let subscribe_packet = SubscribePacket {
        packet_id: 1,
        topics: subscribe_topics,
    };
    let subscribe_event = Event::MessageReceived(subscriber_id.clone(), MqttPacket::Subscribe(subscribe_packet));
    router.handle_event(subscribe_event).await;
    
    // 清除SUBACK消息
    let _ = subscriber_rx.try_recv();
    
    // 创建发布数据包
    let publish_packet = PublishPacket {
        dup: false,
        qos: 0,
        retain: false,
        topic_name: "test/topic".to_string(),
        packet_id: None,
        payload: Bytes::from(vec![1, 2, 3, 4, 5]),
    };
    
    // 发送发布事件
    let publish_event = Event::MessageReceived(publisher_id.clone(), MqttPacket::Publish(publish_packet));
    router.handle_event(publish_event).await;
    
    // 检查订阅者是否收到消息
    if let Ok(Event::MessageSent(recv_client_id, packet)) = subscriber_rx.try_recv() {
        assert_eq!(recv_client_id, subscriber_id);
        match packet {
            MqttPacket::Publish(_) => {
                // 验证是PUBLISH数据包
            }
            _ => {
                panic!("Expected PUBLISH packet, got {:?}", packet);
            }
        }
    } else {
        panic!("Expected MessageSent event for subscriber");
    }
}

#[tokio::test]
async fn test_client_disconnected_event() {
    let router = MessageRouter::new();
    let client_id = "test_client_5".to_string();
    let (tx, _rx) = unbounded();
    
    // 注册客户端
    router.register_client(&client_id.clone(), tx).await.unwrap();
    
    // 发送客户端断开连接事件
    let event = Event::ClientDisconnected(client_id.clone());
    router.handle_event(event).await;
    
    // 验证客户端已被移除
    // 由于remove_client是内部方法，我们无法直接验证
    // 但可以通过尝试发送消息来间接验证
    let (new_tx, _new_rx) = unbounded();
    let result = router.register_client(&client_id.clone(), new_tx).await;
    assert!(result.is_ok());
}
