use std::sync::Arc;
use tokio::sync::Mutex;
use std::io;
use log::{debug, error, info, warn};

use crate::messages::client_message::ClientMessage;
use crate::networking::gameserver::game_server::GameServer;

pub struct ChannelReadHandler {
    channel: Arc<Mutex<tokio::net::TcpStream>>,
    message: ClientMessage,
    game_server: Arc<GameServer>,
}

impl ChannelReadHandler {
    pub fn new(
        channel: Arc<Mutex<tokio::net::TcpStream>>,
        message: ClientMessage,
        game_server: Arc<GameServer>,
    ) -> Self {
        Self {
            channel,
            message,
            game_server,
        }
    }

    pub async fn run(&self) -> io::Result<()> {
        // Get the packet from the message
        let header = self.message.get_header();
        
        // Log the packet being processed
        debug!("Processing packet: {}", header);

        // Handle the packet through the packet manager
        if let Some(packet_manager) = self.game_server.get_packet_manager() {
            match packet_manager.handle(&self.message, Arc::clone(&self.channel)).await {
                Ok(_) => {
                    debug!("Successfully handled packet: {}", header);
                    Ok(())
                }
                Err(e) => {
                    error!("Error handling packet {}: {:?}", header, e);
                    Err(e)
                }
            }
        } else {
            error!("No packet manager available to handle message: {}", header);
            Err(io::Error::new(io::ErrorKind::Other, "No packet manager available"))
        }
    }
}
