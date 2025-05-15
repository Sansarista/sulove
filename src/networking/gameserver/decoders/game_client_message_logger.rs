use log::{debug, error, info, trace, warn};
use std::sync::Arc;
use tokio_util::codec::Decoder;
use bytes::BytesMut;

use crate::messages::client_message::ClientMessage;
use crate::networking::gameserver::game_server::GameServer;

pub struct GameClientMessageLogger {
    packet_manager: Arc<GameServer>,
}

impl GameClientMessageLogger {
    pub fn new(packet_manager: Arc<GameServer>) -> Self {
        Self {
            packet_manager,
        }
    }
}

impl Decoder for GameClientMessageLogger {
    type Item = ClientMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        // Clone the source and pass it through
        let message = match src.clone().try_into() {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to convert bytes to ClientMessage: {:?}", e);
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid message format"));
            }
        };

        // Log the message with its ID and name
        let message_id = message.get_message_id();
        let message_name = self.packet_manager
            .get_packet_manager()
            .get_names()
            .get_incoming_name(message_id)
            .unwrap_or_else(|| "Unknown".to_string());

        debug!("[CLIENT][{:4}][{:41}] => {:?}", 
            message_id, 
            message_name,
            message.get_message_body());

        // Return the message for further processing
        Ok(Some(message))
    }
}