use flume::{Receiver, Sender};

use crate::routing::event::Event;
#[derive(Debug, Clone)]
pub struct RouterChannel {
    /// 集群发送通道
    // tx: Sender<Event>,
    /// 接收通道
    pub rx: Receiver<Event>,
    pub tx: Sender<Event>,
}
