use mqtt_adapt::topic::TopicManager;
use std::sync::Arc;
use std::thread;

#[tokio::test]
async fn test_lock_free_topic_manager() {
    let topic_manager = TopicManager::new();
    
    // Test 1: Add subscriptions
    topic_manager.add_subscription("sensor.1.data", "client_1".to_string(), 1).await;
    topic_manager.add_subscription("sensor.2.data", "client_2".to_string(), 1).await;
    topic_manager.add_subscription("sensor.+.data", "client_3".to_string(), 2).await;
    
    // Test 2: Find subscribers
    let subscribers = topic_manager.find_subscribers("sensor.1.data").await;
    assert_eq!(subscribers.len(), 2); // client_1 and client_3
    
    // Test 3: Remove subscription
    topic_manager.remove_subscription("sensor.1.data", "client_1").await;
    let subscribers_after_remove = topic_manager.find_subscribers("sensor.1.data").await;
    assert_eq!(subscribers_after_remove.len(), 1); // only client_3
    
    // Test 4: Add another subscription
    topic_manager.add_subscription("sensor.0.data", "client_4".to_string(), 1).await;
    
    // Verify subscription was added
    let subscribers = topic_manager.find_subscribers("sensor.0.data").await;
    assert!(!subscribers.is_empty());
}

#[tokio::test]
async fn test_add_topic() {
    let topic_manager = TopicManager::new();
    
    // Test 1: Add topics
    topic_manager.add_topic("sensor.1.data").await;
    topic_manager.add_topic("sensor.2.data").await;
    topic_manager.add_topic("sensor.+.data").await;
    
    // Test 2: Add subscription to existing topic
    topic_manager.add_subscription("sensor.1.data", "client_1".to_string(), 1).await;
    
    // Test 3: Find subscribers
    let subscribers = topic_manager.find_subscribers("sensor.1.data").await;
    assert_eq!(subscribers.len(), 1); // only client_1
    
    // Test 4: Add subscription to wildcard topic
    topic_manager.add_subscription("sensor.+.data", "client_2".to_string(), 2).await;
    
    // Test 5: Find subscribers with wildcard
    let subscribers_wildcard = topic_manager.find_subscribers("sensor.1.data").await;
    assert_eq!(subscribers_wildcard.len(), 2); // client_1 and client_2
}

#[tokio::test]
async fn test_remove_empty_topic() {
    let topic_manager = TopicManager::new();
    
    // Test 1: Add subscription
    topic_manager.add_subscription("sensor.1.data", "client_1".to_string(), 1).await;
    
    // Test 2: Verify subscription exists
    let subscribers = topic_manager.find_subscribers("sensor.1.data").await;
    assert_eq!(subscribers.len(), 1); // client_1
    
    // Test 3: Remove subscription
    topic_manager.remove_subscription("sensor.1.data", "client_1").await;
    
    // Test 4: Verify subscription is removed
    let subscribers_after_remove = topic_manager.find_subscribers("sensor.1.data").await;
    assert_eq!(subscribers_after_remove.len(), 0); // no subscribers
    
    // Test 5: Add another subscription to the same topic
    topic_manager.add_subscription("sensor.1.data", "client_2".to_string(), 1).await;
    
    // Test 6: Verify new subscription exists
    let subscribers_new = topic_manager.find_subscribers("sensor.1.data").await;
    assert_eq!(subscribers_new.len(), 1); // client_2
}
