# MQTT Adapt

一个基于Rust和Tokio实现的MQTT代理（broker）。

## 功能特性

- ✅ MQTT协议解析（支持所有MQTT报文类型）
- ✅ 基于Tokio的异步运行时
- ✅ 模块化设计，每种报文类型独立文件
- ✅ 完整的单元测试

## 项目结构

```
mqtt_adapt/
├── Cargo.toml          # 项目配置和依赖
├── README.md           # 项目说明
├── docs.md             # 实现步骤文档
├── src/
│   ├── lib.rs          # 库入口
│   ├── main.rs         # 主程序
│   └── protocol/       # 协议解析模块
│       ├── mod.rs      # 模块导出
│       ├── types.rs    # 通用类型定义
│       ├── parser.rs   # 解析器
│       ├── serializer.rs # 序列化器
│       ├── connect.rs  # CONNECT报文
│       ├── connack.rs  # CONNACK报文
│       ├── publish.rs  # PUBLISH报文
│       ├── puback.rs   # PUBACK报文
│       ├── pubrec.rs   # PUBREC报文
│       ├── pubrel.rs   # PUBREL报文
│       ├── pubcomp.rs  # PUBCOMP报文
│       ├── subscribe.rs # SUBSCRIBE报文
│       ├── suback.rs   # SUBACK报文
│       ├── unsubscribe.rs # UNSUBSCRIBE报文
│       ├── unsuback.rs # UNSUBACK报文
│       ├── ping.rs     # PINGREQ/PINGRESP报文
│       ├── disconnect.rs # DISCONNECT报文
│       └── tests.rs    # 单元测试
```

## 安装

### 前提条件

- Rust 1.70+
- Cargo

### 安装步骤

1. 克隆仓库

```bash
git clone git@github.com:Angle7C/mqtt_adapt.git
cd mqtt_adapt
```

2. 构建项目

```bash
cargo build
```

3. 运行测试

```bash
cargo test
```

## 依赖

- `tokio` - 异步运行时
- `bytes` - 高效字节处理
- `nom` - 解析器组合库
- `log` - 日志库
- `env_logger` - 环境变量配置的日志实现

## 实现进度

根据 `docs.md` 中的实现步骤：

1. ✅ 解析MQTT协议
2. ⏳ 实现网络层
3. ⏳ 实现会话管理
4. ⏳ 实现订阅/发布机制
5. ⏳ 实现QoS级别
6. ⏳ 实现保留消息
7. ⏳ 实现遗嘱消息
8. ⏳ 实现安全认证
9. ⏳ 性能优化

## 贡献

欢迎提交Issue和Pull Request！

## 许可证

MIT
