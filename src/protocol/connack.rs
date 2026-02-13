use super::ConnectReturnCode;
use super::Packet;
use super::write_remaining_length;
use bytes::{Buf, BufMut, BytesMut};
use anyhow::Result;



/// CONNACK数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnAckPacket {
    pub session_present: bool,
    pub return_code: ConnectReturnCode,
}


impl Packet for ConnAckPacket {
    /// 将CONNACK数据包序列化为字节并写入缓冲区
    fn write(&self, buf: &mut BytesMut) {
        // 计算可变头长度
        let variable_header_length = 2; // session present flag (1 byte) + return code (1 byte)
        
        // 写入固定头
        let packet_type = 2; // CONNACK
        let flags = if self.session_present { 1 } else { 0 };
        let first_byte = (packet_type << 4) | flags;
        buf.put_u8(first_byte);
        
        // 写入剩余长度
        write_remaining_length(buf, variable_header_length);
        
        // 写入可变头
        buf.put_u8(flags); // session present flag
        buf.put_u8(self.return_code as u8); // return code
        
    }
    
    /// 从BytesMut解析CONNACK数据包
    fn parse(input: &mut BytesMut, _flags: Option<u8>) -> Result<Self> {
        if input.len() < 2 {
            return Err(anyhow::format_err!("Insufficient data for CONNACK packet"));
        }
        
        let flags = input.get_u8();
        let return_code_value = input.get_u8();
        
        let session_present = (flags & 0x01) != 0;
        let return_code = ConnectReturnCode::from_u8(return_code_value).ok_or(anyhow::format_err!("Invalid return code"))?; 
        
        Ok(ConnAckPacket {
            session_present,
            return_code,
        })
    }
}
