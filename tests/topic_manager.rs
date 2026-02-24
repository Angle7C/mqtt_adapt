use mqtt_adapt::topic::TopicManager;
use bytes::{Bytes};

#[tokio::test]
async fn test_topic_manager_basic() {
    let mut topic_manager = TopicManager::new();
    
    // Test 1: Add subscriptions
    for i in 0..10 {
        let client_id = format!("client_{}", i);
        let topic = format!("sensor.{}.data", i % 5);
        topic_manager.add_subscription(client_id, topic, 1).await;
    }
    
    // Test 2: Find subscribers
    for i in 0..5 {
        let topic = format!("sensor.{}.data", i);
        let subscribers = topic_manager.find_subscribers(&topic).await;
        assert!(!subscribers.is_empty());
    }
    
    // Test 3: Remove subscriptions
    for i in 0..10 {
        let client_id = format!("client_{}", i);
        let topic = format!("sensor.{}.data", i % 5);
        topic_manager.remove_subscription(client_id, topic).await;
    }
    
    // Test 4: Verify all subscriptions are removed
    for i in 0..5 {
        let topic = format!("sensor.{}.data", i);
        let subscribers = topic_manager.find_subscribers(&topic).await;
        assert!(subscribers.is_empty());
    }
}

// 测试主题通配符匹配
#[tokio::test]
async fn test_topic_wildcard_matching() {
    let mut topic_manager = TopicManager::new();
    
    // 添加带有通配符的订阅
    topic_manager.add_subscription("client1".to_string(), "test/+/topic".to_string(), 0).await;
    topic_manager.add_subscription("client2".to_string(), "test/#".to_string(), 1).await;
    topic_manager.add_subscription("client3".to_string(), "+/specific/topic".to_string(), 2).await;
    
    // 测试单个层级通配符 +
    let subscribers1 = topic_manager.find_subscribers("test/123/topic").await;
    assert_eq!(subscribers1.len(), 2); // client1 和 client2 应该匹配
    
    // 测试多层级通配符 #
    let subscribers2 = topic_manager.find_subscribers("test/foo/bar/baz").await;
    assert_eq!(subscribers2.len(), 1); // 只有 client2 应该匹配
    
    // 测试开头通配符 +
    let subscribers3 = topic_manager.find_subscribers("any/specific/topic").await;
    assert_eq!(subscribers3.len(), 1); // 只有 client3 应该匹配
    
    // 测试不匹配的主题
    let subscribers4 = topic_manager.find_subscribers("other/topic").await;
    assert!(subscribers4.is_empty()); // 没有订阅者应该匹配
}

// 测试保留消息
#[tokio::test]
async fn test_retained_messages() {
    let mut topic_manager = TopicManager::new();
    
    // 存储保留消息
    let payload1 = Bytes::from_static(b"Hello, world!");
    topic_manager.store_retained_message("test/topic".to_string(), payload1.clone(), 1).await;
    
    // 获取保留消息
    let retained_messages1 = topic_manager.get_retained_messages("test/topic").await;
    assert_eq!(retained_messages1.len(), 1);
    assert_eq!(retained_messages1[0].0, "test/topic");
    assert_eq!(retained_messages1[0].1.payload, payload1);
    assert_eq!(retained_messages1[0].1.qos, 1);
    
    // 存储空负载的保留消息（删除保留消息）
    let empty_payload = Bytes::new();
    topic_manager.store_retained_message("test/topic".to_string(), empty_payload, 0).await;
    
    // 验证保留消息已被删除
    let retained_messages2 = topic_manager.get_retained_messages("test/topic").await;
    assert!(retained_messages2.is_empty());
    
    // 存储多个保留消息
    let payload2 = Bytes::from_static(b"Message 2");
    let payload3 = Bytes::from_static(b"Message 3");
    topic_manager.store_retained_message("sensor/1/data".to_string(), payload2.clone(), 0).await;
    topic_manager.store_retained_message("sensor/2/data".to_string(), payload3.clone(), 1).await;
    
    // 获取多个保留消息
    let retained_messages3 = topic_manager.get_retained_messages("sensor/1/data").await;
    assert_eq!(retained_messages3.len(), 1);
    assert_eq!(retained_messages3[0].1.payload, payload2);
    
    let retained_messages4 = topic_manager.get_retained_messages("sensor/2/data").await;
    assert_eq!(retained_messages4.len(), 1);
    assert_eq!(retained_messages4[0].1.payload, payload3);
}

// 测试多个订阅者订阅同一个主题
#[tokio::test]
async fn test_multiple_subscribers() {
    let mut topic_manager = TopicManager::new();
    
    // 添加多个订阅者到同一个主题
    topic_manager.add_subscription("client1".to_string(), "test/topic".to_string(), 0).await;
    topic_manager.add_subscription("client2".to_string(), "test/topic".to_string(), 1).await;
    topic_manager.add_subscription("client3".to_string(), "test/topic".to_string(), 2).await;
    
    // 查找订阅者
    let subscribers = topic_manager.find_subscribers("test/topic").await;
    assert_eq!(subscribers.len(), 3);
    
    // 验证所有订阅者都在列表中
    let client_ids: Vec<String> = subscribers.iter().map(|s| s.client_id.clone()).collect();
    assert!(client_ids.contains(&"client1".to_string()));
    assert!(client_ids.contains(&"client2".to_string()));
    assert!(client_ids.contains(&"client3".to_string()));
    
    // 验证QoS级别
    for subscriber in &subscribers {
        match subscriber.client_id.as_str() {
            "client1" => assert_eq!(subscriber.qos, 0),
            "client2" => assert_eq!(subscriber.qos, 1),
            "client3" => assert_eq!(subscriber.qos, 2),
            _ => panic!("Unexpected client ID: {}", subscriber.client_id),
        }
    }
    
    // 移除一个订阅者
    topic_manager.remove_subscription("client2".to_string(), "test/topic".to_string()).await;
    
    // 验证订阅者已被移除
    let subscribers_after_remove = topic_manager.find_subscribers("test/topic").await;
    assert_eq!(subscribers_after_remove.len(), 2);
    let client_ids_after_remove: Vec<String> = subscribers_after_remove.iter().map(|s| s.client_id.clone()).collect();
    assert!(!client_ids_after_remove.contains(&"client2".to_string()));
}

// 测试不同QoS级别的订阅
#[tokio::test]
async fn test_different_qos_levels() {
    let mut topic_manager = TopicManager::new();
    
    // 添加不同QoS级别的订阅
    topic_manager.add_subscription("client_qos0".to_string(), "test/qos".to_string(), 0).await;
    topic_manager.add_subscription("client_qos1".to_string(), "test/qos".to_string(), 1).await;
    topic_manager.add_subscription("client_qos2".to_string(), "test/qos".to_string(), 2).await;
    
    // 查找订阅者
    let subscribers = topic_manager.find_subscribers("test/qos").await;
    assert_eq!(subscribers.len(), 3);
    
    // 验证每个订阅者的QoS级别
    for subscriber in &subscribers {
        match subscriber.client_id.as_str() {
            "client_qos0" => assert_eq!(subscriber.qos, 0),
            "client_qos1" => assert_eq!(subscriber.qos, 1),
            "client_qos2" => assert_eq!(subscriber.qos, 2),
            _ => panic!("Unexpected client ID: {}", subscriber.client_id),
        }
    }
}

// 测试订阅管理的边界情况
#[tokio::test]
async fn test_subscription_edge_cases() {
    let mut topic_manager = TopicManager::new();
    
    // 测试空主题
    topic_manager.add_subscription("client1".to_string(), "".to_string(), 0).await;
    let subscribers1 = topic_manager.find_subscribers("").await;
    assert_eq!(subscribers1.len(), 1);
    
    // 测试深层嵌套主题
    let deep_topic = "a/b/c/d/e/f/g/h/i/j".to_string();
    topic_manager.add_subscription("client2".to_string(), deep_topic.clone(), 1).await;
    let subscribers2 = topic_manager.find_subscribers(&deep_topic).await;
    assert_eq!(subscribers2.len(), 1);
    
    // 测试移除不存在的订阅
    topic_manager.remove_subscription("non_existent_client".to_string(), "test/topic".to_string()).await;
    // 应该不会崩溃
    
    // 测试移除不存在的主题的订阅
    topic_manager.remove_subscription("client1".to_string(), "non_existent_topic".to_string()).await;
    // 应该不会崩溃
}
