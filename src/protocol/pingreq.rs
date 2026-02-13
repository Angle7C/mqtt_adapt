use bytes::{BufMut, BytesMut};
use super::Packet;
use anyhow::Result;
/// PINGREQ数据包
/// MQTT PINGREQ数据包
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingReqPacket;

impl Packet for PingReqPacket {
    /// 将PINGREQ数据包序列化为字节并写入缓冲区
    fn write(&self, buf: &mut BytesMut) {
        // PINGREQ数据包只包含固定头
        let packet_type = super::PacketType::PingReq as u8;
        let flags = 0x00;
        let first_byte = (packet_type << 4) | flags;
        buf.put_u8(first_byte);
        // 剩余长度为0
        buf.put_u8(0x00);
    }
    
    /// 从BytesMut解析PINGREQ数据包
    fn parse(_input: &mut BytesMut, _flags: Option<u8>) -> Result<Self> {
        // PINGREQ数据包没有可变头部和负载
        Ok(PingReqPacket)
    }
}
