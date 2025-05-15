use std::sync::Arc;
use tokio::sync::Mutex;
use log::{debug, error, info, warn};
use std::io;

use crate::messages::client_message::ClientMessage;
use crate::networking::gameserver::game_server::GameServer;
use crate::threading::channel_read_handler::ChannelReadHandler;

pub struct GameMessageHandler {
    game_server: Arc<GameServer>,
}

impl GameMessageHandler {
    pub fn new(game_server: Arc<GameServer>) -> Self {
        Self {
            game_server,
        }
    }

    // Handle channel registration
    pub async fn channel_registered(&self, channel: Arc<Mutex<tokio::net::TcpStream>>) -> io::Result<()> {
        // Add client to the game client manager
        if !self.game_server.get_game_client_manager().add_client(channel).await {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to add client"));
        }
        
        Ok(())
    }

    // Handle channel unregistration
    pub async fn channel_unregistered(&self, channel: Arc<Mutex<tokio::net::TcpStream>>) -> io::Result<()> {
        // Remove client from the game client manager
        // In a real implementation, this would close the channel
        Ok(())
    }

    // Handle incoming messages
    pub async fn handle_message(&self, channel: Arc<Mutex<tokio::net::TcpStream>>, message: ClientMessage) -> io::Result<()> {
        let multi_threaded = self.game_server.get_config().get_bool("packet_handling.multi_threaded");
        
        let handler = ChannelReadHandler::new(channel, message, Arc::clone(&self.game_server));
        
        if multi_threaded {
            // Spawn a new task to handle the message
            tokio::spawn(async move {
                if let Err(e) = handler.run().await {
                    error!("Error handling message: {:?}", e);
                }
            });
        } else {
            // Handle the message in the current task
            if let Err(e) = handler.run().await {
                error!("Error handling message: {:?}", e);
                return Err(e);
            }
        }
        
        Ok(())
    }

    // Handle channel inactive
    pub async fn channel_inactive(&self, channel: Arc<Mutex<tokio::net::TcpStream>>) -> io::Result<()> {
        // In a real implementation, this would close the channel
        Ok(())
    }

    // Handle exceptions
    pub async fn exception_caught(&self, channel: Arc<Mutex<tokio::net::TcpStream>>, error: io::Error) -> io::Result<()> {
        // Check if it's an IO error
        if error.kind() == io::ErrorKind::ConnectionReset || 
           error.kind() == io::ErrorKind::BrokenPipe ||
           error.kind() == io::ErrorKind::ConnectionAborted {
            // Close the channel
            return Ok(());
        }

        // Log the error if in debug mode
        if self.game_server.get_config().get_bool("debug.mode") {
            if error.kind() == io::ErrorKind::InvalidData {
                error!("Disconnecting client, reason: \"{}\".", error);
            } else {
                error!("Disconnecting client, exception in GameMessageHandler: {:?}", error);
            }
        }

        // Close the channel
        Ok(())
    }
}