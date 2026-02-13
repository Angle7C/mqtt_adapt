use anyhow::Result;
use bytes::BytesMut;
use flume::{Receiver, Sender};
use tokio::{io::BufReader, net::TcpStream};

use crate::routing::event::Event;

/// 客户端连接状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientState {
    /// 未连接
    Disconnected,
    /// 已连接
    Connected,
}

/// 客户端结构体
#[derive(Debug)]
pub struct Client {
    /// 底层TCP连接
    pub(super) socket: BufReader<TcpStream>,
    /// 连接状态
    pub(super) state: ClientState,
    /// 客户端ID
    pub(super) client_id: String,
    /// 保活时间（秒）
    pub(super) keepalive: u16,
    /// 读取缓冲区
    pub(super) read_buf: BytesMut,
    /// 写入缓冲区
    pub(super) write_buf: BytesMut,
    /// 消息接收通道
    pub(super) event_receiver: Receiver<Event>,
    /// 消息发送通道（用于向路由器发送事件）
    pub(super) router_send: Sender<Event>,
    /// 客户端地址
    pub(super) addr: std::net::SocketAddr,
}

impl Client {
    /// 创建新的客户端
    pub fn new(socket: TcpStream, addr: std::net::SocketAddr, rx: Receiver<Event>, tx: Sender<Event>, client_id: String) -> Self {
        Self {
            socket: BufReader::new(socket),
            state: ClientState::Connected,
            addr,
            client_id,
            keepalive: 60, // 默认保活时间为60秒
            read_buf: BytesMut::with_capacity(1024 * 10),
            write_buf: BytesMut::with_capacity(1024 * 10),
            event_receiver:rx,
            router_send: tx,
        }
    }



    /// 发送事件到路由器
    pub fn send_event(&self, event: Event) -> Result<()> {
        self.router_send.send(event)?;
        Ok(())
    }

    /// 设置客户端ID
    pub fn set_client_id(&mut self, client_id: String) {
        self.client_id = client_id;
    }

    /// 获取客户端ID
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// 获取客户端状态
    pub fn state(&self) -> &ClientState {
        &self.state
    }

    /// 设置保活时间
    pub fn set_keepalive(&mut self, keepalive: u16) {
        self.keepalive = keepalive;
    }

    /// 获取保活时间
    pub fn keepalive(&self) -> u16 {
        self.keepalive
    }
}
