use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::client::client::Client;

impl Client {
    /// 写入数据到客户端
    /// 
    /// 将写入缓冲区中的数据发送到客户端，并在发送完成后清空缓冲区
    pub async fn write(&mut self) -> Result<()> {
        self.socket.write(&self.write_buf).await?;
        self.write_buf.clear();
        Ok(())
    }

    /// 从客户端读取数据
    /// 
    /// 从TCP连接中读取数据到读取缓冲区，并返回读取的字节数
    pub async fn read(&mut self) -> Result<usize> {
        self.read_buf.clear();
        
        let n = self.socket.read_buf(&mut self.read_buf).await?;
        Ok(n)
    }
}
