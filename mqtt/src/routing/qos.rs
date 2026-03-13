use crate::protocol::PublishPacket;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct QoSManager {
    outgoing_messages: HashMap<u16, PublishPacket>,
    incoming_qos2: HashMap<u16, PublishPacket>,
    next_packet_id: u16,
}

impl QoSManager {
    pub fn new() -> Self {
        Self {
            outgoing_messages: HashMap::new(),
            incoming_qos2: HashMap::new(),
            next_packet_id: 1,
        }
    }

    pub fn next_packet_id(&mut self) -> u16 {
        let id = self.next_packet_id;
        self.next_packet_id = self.next_packet_id.wrapping_add(1);
        if self.next_packet_id == 0 {
            self.next_packet_id = 1;
        }
        id
    }

    pub fn store_outgoing(&mut self, packet_id: u16, packet: PublishPacket) {
        self.outgoing_messages.insert(packet_id, packet);
    }

    pub fn remove_outgoing(&mut self, packet_id: u16) -> Option<PublishPacket> {
        self.outgoing_messages.remove(&packet_id)
    }

    pub fn store_incoming_qos2(&mut self, packet_id: u16, packet: PublishPacket) -> bool {
        if self.incoming_qos2.contains_key(&packet_id) {
            false
        } else {
            self.incoming_qos2.insert(packet_id, packet);
            true
        }
    }

    pub fn remove_incoming_qos2(&mut self, packet_id: u16) -> Option<PublishPacket> {
        self.incoming_qos2.remove(&packet_id)
    }
}

impl Default for QoSManager {
    fn default() -> Self {
        Self::new()
    }
}
