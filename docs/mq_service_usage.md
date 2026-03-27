# MQ 服务使用指南

## 概述

MQ 服务已被重构为解耦的设计，生产者和消费者现在是独立的服务，可以根据需要单独使用。

## 服务架构

### 1. MqProducerService - 生产者服务
专门负责消息发送，提供简洁的消息发送接口。

### 2. MqConsumerManager - 消费者管理器
专门负责管理消费者服务，支持多个消费者实例。

### 3. MqRequestResponseService - 请求-响应服务（可选）
提供请求-响应模式支持，适用于需要同步响应的场景。

### 4. MqService - 传统服务（已弃用）
保留用于向后兼容，未来版本可能会被移除。

## 使用示例

### 仅使用生产者服务

```rust
use mqtt_adapt::mq::producer_service::MqProducerService;
use mqtt_adapt::mq::factory::MqFactory;

// 创建生产者服务
let producer = MqFactory::create_producer(config)?;
let producer_service = MqProducerService::new(producer).await?;

// 发送消息
let message = MqMessage::new("topic", payload, 1, false, "node_id");
producer_service.send_message(message).await?;

// 关闭服务
producer_service.close().await?;
```

### 仅使用消费者管理器

```rust
use mqtt_adapt::mq::consumer_manager::MqConsumerManager;
use mqtt_adapt::mq::factory::MqFactory;

// 创建消费者管理器
let consumer_manager = MqConsumerManager::new();

// 添加消费者
let consumer = MqFactory::create_consumer(config)?;
let consumer_service = consumer_manager.add_consumer(consumer).await?;

// 设置消息处理器
let mut service = consumer_service.lock().await;
service.set_message_handler(Box::new(|message| {
    // 处理消息
    Ok(())
}));

// 启动所有消费者
consumer_manager.start_all().await?;

// 关闭所有消费者
consumer_manager.close_all().await?;
```

### 使用请求-响应服务

```rust
use mqtt_adapt::mq::request_response_service::MqRequestResponseService;
use mqtt_adapt::mq::factory::MqFactory;

// 创建请求-响应服务
let producer = MqFactory::create_producer(config)?;
let rr_service = MqRequestResponseService::new(producer).await?;

// 启动响应处理
rr_service.start().await;

// 发送请求并注册回调
let message = MqMessage::new("request/topic", payload, 1, false, "node_id");
rr_service.send_request(
    message,
    "response/topic".to_string(),
    Box::new(|response| {
        // 处理响应
        Ok(())
    })
).await?;

// 处理接收到的响应
rr_service.handle_response(response_message);
```

### 同时使用生产者和消费者

```rust
use mqtt_adapt::mq::producer_service::MqProducerService;
use mqtt_adapt::mq::consumer_manager::MqConsumerManager;
use mqtt_adapt::mq::factory::MqFactory;

// 创建独立的生产者服务
let producer = MqFactory::create_producer(config)?;
let producer_service = MqProducerService::new(producer).await?;

// 创建独立的消费者管理器
let consumer_manager = MqConsumerManager::new();

// 添加消费者
let consumer = MqFactory::create_consumer(config)?;
let consumer_service = consumer_manager.add_consumer(consumer).await?;

// 设置消息处理器
let mut service = consumer_service.lock().await;
service.set_message_handler(Box::new(|message| {
    // 处理消息
    Ok(())
}));

// 启动消费者
consumer_manager.start_all().await?;

// 发送消息
let message = MqMessage::new("topic", payload, 1, false, "node_id");
producer_service.send_message(message).await?;

// 关闭服务
producer_service.close().await?;
consumer_manager.close_all().await?;
```

## 迁移指南

### 从旧的 MqService 迁移

如果你正在使用旧的 `MqService`，可以按照以下步骤迁移：

1. **仅发送消息**：使用 `MqProducerService` 替代
2. **仅接收消息**：使用 `MqConsumerManager` 替代
3. **请求-响应模式**：使用 `MqRequestResponseService` 替代
4. **同时发送和接收**：分别创建 `MqProducerService` 和 `MqConsumerManager`

### 优势

- **职责清晰**：每个服务只负责一个功能
- **独立使用**：可以根据需要单独使用生产者或消费者
- **更好的性能**：减少不必要的耦合和开销
- **易于测试**：每个服务可以独立测试
- **更好的扩展性**：可以灵活组合不同的服务

## 性能优化

新的解耦设计已经包含以下性能优化：

1. **异步锁**：使用 `tokio::sync::Mutex` 和 `tokio::sync::RwLock`
2. **任务池**：使用 `tokio::spawn` 管理消费者服务
3. **背压机制**：有界通道防止消息堆积
4. **读写分离**：使用 `RwLock` 减少锁竞争

## 注意事项

1. **生命周期管理**：确保在应用退出时正确关闭所有服务
2. **错误处理**：所有服务都返回 `Result`，需要正确处理错误
3. **并发安全**：所有服务都是线程安全的，可以在多线程环境中使用
4. **资源清理**：使用 `close()` 方法正确释放资源


如果河流有了生命，那汇入大海的行为是个什么行为？ （生命）
如果你的梦能影响现实，你觉得你会做什么梦？ （自己）