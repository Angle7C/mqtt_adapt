use crate::ClinetId;
use crate::routing::event::Event;
use crate::routing::qos::QoSManager;
use crate::topic::TopicManager;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex};
use flume::{Receiver, Sender, unbounded};
use anyhow::Result;
use crate::protocol::{ConnAckPacket, ConnectReturnCode, MqttPacket, PublishPacket, PubAckPacket, PubRecPacket, PubRelPacket, PubCompPacket};

#[derive(Debug, Clone)]
pub struct MessageRouter {
    topic_manager: Arc<Mutex<TopicManager>>,
    qos_manager: Arc<Mutex<QoSManager>>,
    sender: Arc<Mutex<HashMap<ClinetId, Sender<Event>>>>,
    event_sender: Sender<Event>,
    event_receiver: Receiver<Event>,
}

impl MessageRouter {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        
        Self {
            topic_manager: Arc::new(Mutex::new(TopicManager::new())),
            qos_manager: Arc::new(Mutex::new(QoSManager::new())),
            sender: Arc::new(Mutex::new(HashMap::new())),
            event_sender: tx,
            event_receiver: rx,
        }
    }
    
    pub fn get_sender(&self) -> Sender<Event> {
        self.event_sender.clone()
    }
    
    pub async fn register_client(&self, client_id: &str, sender: Sender<Event>) -> Result<()> {
        let mut senders = self.sender.lock().await;
        senders.insert(client_id.to_string(), sender);
        Ok(())
    }

    pub async fn remove_client(&self, client_id: &str) {
        let mut senders = self.sender.lock().await;
        senders.remove(client_id);
    }

    pub async fn handle_event(&self, event: Event) {
        match event {
            Event::ClientConnected(client_id) => {
                let connack_packet = ConnAckPacket {
                    session_present: false,
                    return_code: ConnectReturnCode::Accepted,
                };
                
                let mqtt_packet = MqttPacket::ConnAck(connack_packet);
                
                let senders = self.sender.lock().await;
                if let Some(tx) = senders.get(&client_id) {
                    let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                    if let Err(e) = tx.try_send(event) {
                        error!("Error sending CONNACK to {}: {:?}", client_id, e);
                    }
                }
            }
            Event::ClientDisconnected(client_id) => {
                self.remove_client(&client_id).await;
            }
            Event::MessageReceived(client_id, packet) => {
                match packet {
                    MqttPacket::Subscribe(subscribe_packet) => {
                        self.handle_subscribe(client_id, subscribe_packet).await;
                    }
                    MqttPacket::Unsubscribe(unsubscribe_packet) => {
                        self.handle_unsubscribe(client_id, unsubscribe_packet).await;
                    }
                    MqttPacket::Publish(publish_packet) => {
                        self.handle_publish(client_id, publish_packet).await;
                    }
                    MqttPacket::PubAck(puback_packet) => {
                        self.handle_puback(client_id, puback_packet).await;
                    }
                    MqttPacket::PubRec(pubrec_packet) => {
                        self.handle_pubrec(client_id, pubrec_packet).await;
                    }
                    MqttPacket::PubRel(pubrel_packet) => {
                        self.handle_pubrel(client_id, pubrel_packet).await;
                    }
                    MqttPacket::PubComp(pubcomp_packet) => {
                        self.handle_pubcomp(client_id, pubcomp_packet).await;
                    }
                    _ => {
                        info!("Other packet type: {:?}", packet);
                    }
                }
            }
            Event::MessageSent(_client_id, _packet) => {
            }
            Event::BroadcastMessage(_packet) => {
            }
        }
    }

    pub async fn start(self) {
        while let Ok(event) = self.event_receiver.recv_async().await {
            self.handle_event(event).await;
        }
    }
    
    async fn handle_subscribe(&self, client_id: ClinetId, subscribe_packet: crate::protocol::SubscribePacket) {
        let mut code = 0x80;
        let mut topic_manager = self.topic_manager.lock().await;
        
        for (topic_filter, qos) in &subscribe_packet.topics {
            topic_manager.add_subscription(client_id.clone(), topic_filter.to_string(), *qos).await;
            code = *qos;
        }
        
        let suback_packet = crate::protocol::SubAckPacket {
            packet_id: subscribe_packet.packet_id,
            return_codes: code,
        };
        
        let mqtt_packet = MqttPacket::SubAck(suback_packet);
        let senders = self.sender.lock().await;
        if let Some(tx) = senders.get(&client_id) {
            let event = Event::MessageSent(client_id.clone(), mqtt_packet);
            if let Err(e) = tx.try_send(event) {
                error!("Error sending SUBACK to {}: {:?}", client_id, e);
            }
        }
        
        drop(senders);
        drop(topic_manager);
        
        for (topic_filter, qos) in &subscribe_packet.topics {
            self.send_retained_messages(client_id.clone(), topic_filter, *qos).await;
        }
    }
    
    async fn send_retained_messages(&self, client_id: ClinetId, topic_filter: &str, qos: u8) {
        let retained_messages = {
            let topic_manager = self.topic_manager.lock().await;
            topic_manager.get_retained_messages(topic_filter).await
        };
        
        if retained_messages.is_empty() {
            return;
        }
        
        let senders = self.sender.lock().await;
        if let Some(tx) = senders.get(&client_id) {
            for (topic, retained) in retained_messages {
                let publish_packet = PublishPacket {
                    dup: false,
                    qos: std::cmp::min(qos, retained.qos),
                    retain: true,
                    topic_name: topic,
                    packet_id: None,
                    payload: retained.payload,
                };
                
                let mqtt_packet = MqttPacket::Publish(publish_packet);
                let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                if let Err(e) = tx.try_send(event) {
                    error!("Error sending retained message to {}: {:?}", client_id, e);
                }
            }
        }
    }
    
    async fn handle_unsubscribe(&self, client_id: ClinetId, unsubscribe_packet: crate::protocol::UnsubscribePacket) {
        let mut topic_manager = self.topic_manager.lock().await;
        
        for topic_filter in &unsubscribe_packet.topics {
            topic_manager.remove_subscription(client_id.clone(), topic_filter.to_string()).await;
        }
        
        let unsuback_packet = crate::protocol::UnsubAckPacket {
            packet_id: unsubscribe_packet.packet_id,
        };
        
        let mqtt_packet = MqttPacket::UnsubAck(unsuback_packet);
        let senders = self.sender.lock().await;
        if let Some(tx) = senders.get(&client_id) {
            let event = Event::MessageSent(client_id.clone(), mqtt_packet);
            if let Err(e) = tx.try_send(event) {
                error!("Error sending UNSUBACK to {}: {:?}", client_id, e);
            }
        }
    }
    
    async fn handle_publish(&self, client_id: ClinetId, mut publish_packet: crate::protocol::PublishPacket) {
        let topic = publish_packet.topic_name.clone();
        let retain = publish_packet.retain;
        let qos = publish_packet.qos;
        let payload = publish_packet.payload.clone();
        
        if retain {
            let mut topic_manager = self.topic_manager.lock().await;
            topic_manager.store_retained_message(topic.clone(), payload, qos).await;
        }
        
        let subscribers = {
            let topic_manager = self.topic_manager.lock().await;
            topic_manager.find_subscribers(&topic).await
        };
        
        if subscribers.is_empty() {
            return;
        }
        
        let senders = self.sender.lock().await;
        for subscriber in subscribers {
            if let Some(tx) = senders.get(&subscriber.client_id) {
                let mut msg_packet = publish_packet.clone();
                msg_packet.qos = subscriber.qos;
                msg_packet.retain = false;
                
                if subscriber.qos > 0 && msg_packet.packet_id.is_none() {
                    let mut qos_manager = self.qos_manager.lock().await;
                    msg_packet.packet_id = Some(qos_manager.next_packet_id());
                }
                
                let mqtt_packet = MqttPacket::Publish(msg_packet.clone());
                let event = Event::MessageSent(subscriber.client_id.clone(), mqtt_packet);
                if let Err(_e) = tx.try_send(event) {
                }
                
                if subscriber.qos == 1 && msg_packet.packet_id.is_some() {
                    let mut qos_manager = self.qos_manager.lock().await;
                    qos_manager.store_outgoing(msg_packet.packet_id.unwrap(), msg_packet);
                }
            }
        }
        
        drop(senders);
        
        if qos == 1 && publish_packet.packet_id.is_some() {
            let puback_packet = PubAckPacket {
                packet_id: publish_packet.packet_id.unwrap(),
            };
            let mqtt_packet = MqttPacket::PubAck(puback_packet);
            let senders = self.sender.lock().await;
            if let Some(tx) = senders.get(&client_id) {
                let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                if let Err(e) = tx.try_send(event) {
                    error!("Error sending PUBACK to {}: {:?}", client_id, e);
                }
            }
        } else if qos == 2 && publish_packet.packet_id.is_some() {
            let packet_id = publish_packet.packet_id.unwrap();
            let mut qos_manager = self.qos_manager.lock().await;
            if qos_manager.store_incoming_qos2(packet_id, publish_packet) {
                drop(qos_manager);
                let pubrec_packet = PubRecPacket {
                    packet_id,
                };
                let mqtt_packet = MqttPacket::PubRec(pubrec_packet);
                let senders = self.sender.lock().await;
                if let Some(tx) = senders.get(&client_id) {
                    let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                    if let Err(e) = tx.try_send(event) {
                        error!("Error sending PUBREC to {}: {:?}", client_id, e);
                    }
                }
            }
        }
    }
    
    async fn handle_puback(&self, _client_id: ClinetId, puback_packet: PubAckPacket) {
        let mut qos_manager = self.qos_manager.lock().await;
        qos_manager.remove_outgoing(puback_packet.packet_id);
    }
    
    async fn handle_pubrec(&self, client_id: ClinetId, pubrec_packet: PubRecPacket) {
        let mut qos_manager = self.qos_manager.lock().await;
        if qos_manager.remove_outgoing(pubrec_packet.packet_id).is_some() {
            drop(qos_manager);
            let pubrel_packet = PubRelPacket {
                packet_id: pubrec_packet.packet_id,
            };
            let mqtt_packet = MqttPacket::PubRel(pubrel_packet);
            let senders = self.sender.lock().await;
            if let Some(tx) = senders.get(&client_id) {
                let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                if let Err(e) = tx.try_send(event) {
                    error!("Error sending PUBREL to {}: {:?}", client_id, e);
                }
            }
        }
    }
    
    async fn handle_pubrel(&self, client_id: ClinetId, pubrel_packet: PubRelPacket) {
        let mut qos_manager = self.qos_manager.lock().await;
        if qos_manager.remove_incoming_qos2(pubrel_packet.packet_id).is_some() {
            drop(qos_manager);
            let pubcomp_packet = PubCompPacket {
                packet_id: pubrel_packet.packet_id,
            };
            let mqtt_packet = MqttPacket::PubComp(pubcomp_packet);
            let senders = self.sender.lock().await;
            if let Some(tx) = senders.get(&client_id) {
                let event = Event::MessageSent(client_id.clone(), mqtt_packet);
                if let Err(e) = tx.try_send(event) {
                    error!("Error sending PUBCOMP to {}: {:?}", client_id, e);
                }
            }
        }
    }
    
    async fn handle_pubcomp(&self, _client_id: ClinetId, pubcomp_packet: PubCompPacket) {
        let mut qos_manager = self.qos_manager.lock().await;
        qos_manager.remove_outgoing(pubcomp_packet.packet_id);
    }
    
}
