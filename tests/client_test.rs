use mqtt_adapt::client::{Client, ClientState}; 
use mqtt_adapt::protocol::{MqttPacket, PublishPacket}; 
use mqtt_adapt::routing::event::Event; 
use flume::{unbounded}; 
use bytes::{Bytes}; 
use tokio::net::{TcpListener, TcpStream}; 

// 测试Client结构体的基本功能
#[tokio::test]
async fn test_client_basic_functions() {
    // 创建测试TCP连接
    let listener = TcpListener::bind("127.0.0.1:3333").await.unwrap();

    let addr = listener.local_addr().unwrap();
    
    let socket = TcpStream::connect(addr).await.unwrap();
    
    // 创建事件通道
    let (_tx, rx) = unbounded();
    let (router_tx, _router_rx) = unbounded();
    
    // 创建客户端
    let mut client = Client::new(socket, addr, rx, router_tx, "test_client".to_string());
    
    // 测试设置客户端ID
    client.set_client_id("new_client_id".to_string());
    assert_eq!(client.client_id(), "new_client_id");
    
    // 测试设置保活时间
    client.set_keepalive(120);
    assert_eq!(client.keepalive(), 120);
    
    // 测试获取客户端状态
    assert_eq!(client.state(), &ClientState::Connected);
}

// 测试Client的事件处理
#[tokio::test]
async fn test_client_event_handling() {
    // 创建测试TCP连接
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    let socket = TcpStream::connect(addr).await.unwrap();
    
    // 创建事件通道
    let (_tx, rx) = unbounded();
    let (router_tx, router_rx) = unbounded();
    
    // 创建客户端
    let client = Client::new(socket, addr, rx, router_tx, "test_client".to_string());
    
    // 测试发送事件
    let publish_packet = PublishPacket {
        dup: false,
        qos: 0,
        retain: false,
        topic_name: "test/topic".to_string(),
        packet_id: None,
        payload: Bytes::from_static(b"test payload"),
    };
    
    let event = Event::MessageReceived("test_client".to_string(), MqttPacket::Publish(publish_packet));
    let send_result = client.send_event(event);
    assert!(send_result.is_ok());
    
    // 检查路由器是否收到事件
    if let Ok(received_event) = router_rx.try_recv() {
        match received_event {
            Event::MessageReceived(client_id, packet) => {
                assert_eq!(client_id, "test_client");
                match packet {
                    MqttPacket::Publish(_) => {
                        // 验证是PUBLISH数据包
                    }
                    _ => {
                        panic!("Expected PUBLISH packet, got {:?}", packet);
                    }
                }
            }
            _ => {
                panic!("Expected MessageReceived event, got {:?}", received_event);
            }
        }
    } else {
        panic!("Expected event in router channel");
    }
}

// 测试保活时间设置
#[tokio::test]
async fn test_client_keepalive() {
    // 创建测试TCP连接
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    let socket = TcpStream::connect(addr).await.unwrap();
    
    // 创建事件通道
    let (_tx, rx) = unbounded();
    let (router_tx, _router_rx) = unbounded();
    
    // 创建客户端
    let mut client = Client::new(socket, addr, rx, router_tx, "test_client".to_string());
    
    // 默认保活时间应该是60秒
    assert_eq!(client.keepalive(), 60);
    
    // 设置保活时间为120秒
    client.set_keepalive(120);
    assert_eq!(client.keepalive(), 120);
    
    // 设置保活时间为0秒（禁用保活）
    client.set_keepalive(0);
    assert_eq!(client.keepalive(), 0);
}

// 测试客户端ID设置
#[tokio::test]
async fn test_client_id_management() {
    // 创建测试TCP连接
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    let socket = TcpStream::connect(addr).await.unwrap();
    
    // 创建事件通道
    let (_tx, rx) = unbounded();
    let (router_tx, _router_rx) = unbounded();
    
    // 创建客户端
    let mut client = Client::new(socket, addr, rx, router_tx, "initial_client".to_string());
    
    // 初始客户端ID
    assert_eq!(client.client_id(), "initial_client");
    
    // 修改客户端ID
    client.set_client_id("updated_client".to_string());
    assert_eq!(client.client_id(), "updated_client");
    
    // 再次修改客户端ID
    client.set_client_id("final_client".to_string());
    assert_eq!(client.client_id(), "final_client");
}

// 测试客户端状态
#[tokio::test]
async fn test_client_state_management() {
    // 创建测试TCP连接
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    let socket = TcpStream::connect(addr).await.unwrap();
    
    // 创建事件通道
    let (_tx, rx) = unbounded();
    let (router_tx, _router_rx) = unbounded();
    
    // 创建客户端
    let client = Client::new(socket, addr, rx, router_tx, "test_client".to_string());
    
    // 初始状态应该是Connected
    assert_eq!(client.state(), &ClientState::Connected);
}
