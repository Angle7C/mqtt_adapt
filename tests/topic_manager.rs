use mqtt_adapt::topic::TopicManager;

#[tokio::test]
async fn test_topic_manager() {
    let topic_manager = TopicManager::new();
    
    // Test 1: Add subscriptions
    for i in 0..10 {
        let client_id = format!("client_{}", i);
        let topic = format!("sensor.{}.data", i % 5);
        topic_manager.add_subscription(&topic, client_id, 1).await;
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
        topic_manager.remove_subscription(&topic, &client_id).await;
    }
    
    // Test 4: Verify all subscriptions are removed
    for i in 0..5 {
        let topic = format!("sensor.{}.data", i);
        let subscribers = topic_manager.find_subscribers(&topic).await;
        assert!(subscribers.is_empty());
    }
}
