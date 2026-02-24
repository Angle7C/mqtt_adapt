# MQTT Adapt

一个基于 Rust 和 Tokio 实现的高性能 MQTT 代理（broker）。

## 功能特性

- ✅ 完整的 MQTT 3.1.1 协议支持（所有报文类型）
- ✅ 基于 Tokio 的异步运行时，高性能并发处理
- ✅ 事件驱动架构，使用 flume 通道进行消息传递
- ✅ 模块化设计，协议解析、网络层、路由层分离
- ✅ 用户认证（数据库集成）
- ✅ 遗嘱消息（LWT）支持
- ✅ 订阅/发布机制
- ✅ 消息路由
- ✅ QoS 0/1/2 级别完整支持
- ✅ 保留消息功能（支持数据库存储）
- ✅ 数据库集成（SQLite/PostgreSQL）
- ✅ 消息队列模块
- ✅ 性能优化（mimalloc 内存分配器）
- ✅ 完整的基准测试

## 项目结构

```
mqtt_adapt/
├── Cargo.toml                  # 项目配置和依赖
├── README.md                   # 项目说明
├── docs.md                     # 实现步骤文档
├── hotspot_analysis.md         # 热点代码分析文档
├── 1.svg                       # 性能火焰图
├── migrations/                 # 数据库迁移
│   └── 001_create_retained_messages_table.sql
├── benches/
│   └── performance.rs          # 基准测试
├── src/
│   ├── lib.rs                  # 库入口
│   ├── main.rs                 # 主程序
│   ├── server.rs               # MQTT 服务器
│   ├── client/                 # 客户端模块
│   │   ├── mod.rs
│   │   ├── client.rs           # 客户端结构体
│   │   ├── builder.rs          # 客户端构建器
│   │   ├── handler.rs          # 客户端事件处理
│   │   └── connection.rs       # 网络 I/O 操作
│   ├── protocol/               # 协议解析模块
│   │   ├── mod.rs
│   │   ├── connect.rs          # CONNECT 报文
│   │   ├── connack.rs          # CONNACK 报文
│   │   ├── publish.rs          # PUBLISH 报文
│   │   ├── puback.rs           # PUBACK 报文
│   │   ├── pubrec.rs           # PUBREC 报文
│   │   ├── pubrel.rs           # PUBREL 报文
│   │   ├── pubcomp.rs          # PUBCOMP 报文
│   │   ├── subscribe.rs        # SUBSCRIBE 报文
│   │   ├── suback.rs           # SUBACK 报文
│   │   ├── unsubscribe.rs      # UNSUBSCRIBE 报文
│   │   ├── unsuback.rs         # UNSUBACK 报文
│   │   ├── pingreq.rs          # PINGREQ 报文
│   │   ├── pingresp.rs         # PINGRESP 报文
│   │   └── disconnect.rs       # DISCONNECT 报文
│   ├── routing/                # 消息路由模块
│   │   ├── mod.rs
│   │   ├── router.rs           # 消息路由器
│   │   ├── qos.rs              # QoS 消息状态管理
│   │   ├── event.rs            # 事件定义
│   │   └── channel.rs          # 通道定义
│   ├── topic.rs                # 主题管理
│   ├── db/                     # 数据库模块
│   │   ├── mod.rs
│   │   ├── connection.rs       # 数据库连接
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── ag_user.rs      # 用户模型
│   │       └── retained_message.rs  # 保留消息模型
│   └── mq/                     # 消息队列模块
│       ├── mod.rs
│       ├── client.rs           # MQ 客户端
│       ├── consumer.rs         # 消费者
│       ├── producer.rs         # 生产者
│       ├── service.rs          # MQ 服务
│       ├── message.rs          # 消息定义
│       ├── factory.rs          # 工厂模式
│       ├── device_data.rs      # 设备数据
│       ├── thread_pool.rs      # 线程池
│       └── topic_resolver.rs   # 主题解析器
└── tests/
    ├── topic_manager.rs        # 主题管理测试
    └── message_router.rs       # 消息路由测试
```

## 安装

### 前提条件

- Rust 1.75+
- Cargo

### 安装步骤

1. 克隆仓库

```bash
git clone git@github.com:Angle7C/mqtt_adapt.git
cd mqtt_adapt
```

2. 构建项目

```bash
cargo build --release
```

3. 运行测试

```bash
cargo test
```

4. 运行基准测试

```bash
cargo bench
```

## 依赖

### 核心依赖
- `tokio` - 异步运行时
- `bytes` - 高效字节处理
- `nom` - 解析器组合库
- `anyhow` - 错误处理
- `flume` - 异步通道
- `mimalloc` - 高性能内存分配器

### 数据库
- `sqlx` - 异步数据库访问
- `chrono` - 日期时间处理

### 日志和监控
- `log` - 日志库
- `env_logger` - 环境变量配置的日志实现
- `tracing` - 应用程序追踪

### 其他
- `async-trait` - 异步 trait
- `uuid` - UUID 生成
- `regex` - 正则表达式
- `serde` - 序列化/反序列化
- `serde_json` - JSON 处理

### 开发依赖
- `criterion` - 基准测试框架
- `rumqttc` - MQTT 客户端（用于测试）
- `rand` - 随机数生成

## 网络层架构

### 核心组件

1. **服务器** (`src/server.rs`)
   - 使用 `tokio::net::TcpListener` 监听 TCP 连接
   - 每个连接独立 `tokio::spawn` 任务处理
   - 禁用 Nagle 算法优化延迟

2. **客户端** (`src/client/`)
   - `Client` 结构体管理单个连接状态
   - `tokio::select!` 多路复用网络事件
   - 支持保活超时检测

3. **协议解析** (`src/protocol/`)
   - 零拷贝解析（使用 `BytesMut`）
   - 完整支持所有 MQTT 3.1.1 报文类型
   - 模块化设计，易于扩展

### 数据流向

```
客户端 → TCP → TcpListener → Client → 协议解析 → MessageRouter → 订阅者
```

## 实现进度

根据 `docs.md` 中的实现步骤：

1. ✅ 解析 MQTT 协议
2. ✅ 实现网络层
3. ⏳ 实现会话管理
4. ✅ 实现订阅/发布机制
5. ✅ 实现 QoS 级别
6. ✅ 实现保留消息
7. ✅ 实现遗嘱消息
8. ✅ 实现安全认证
9. ✅ 性能优化

## 性能优化

### 已实现的优化

1. **mimalloc 内存分配器**
   - 替代默认的系统分配器
   - 减少内存碎片，提升分配速度

2. **异步 I/O**
   - Tokio 异步运行时
   - 非阻塞网络操作
   - 连接级别的并发处理

3. **缓冲区优化**
   - 预分配 `BytesMut` 缓冲区（10KB）
   - 零拷贝协议解析

### 性能基准测试

```
rumqttc_connect_publish_disconnect
                        time:   [618.15 µs 621.02 µs 624.03 µs]
```

详细的性能分析请参考 [hotspot_analysis.md](./hotspot_analysis.md)。

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT
