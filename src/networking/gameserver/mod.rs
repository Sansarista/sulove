use std::io;

// Re-export main structs
pub mod game_server;
pub mod game_server_attributes;

// These modules should exist if they're needed
pub mod decoders;
pub mod encoders;
pub mod handlers;

// Public re-exports
pub use game_server::GameServer;
// Also re-export the attributes types needed by code that integrates with this module
pub use game_server_attributes::{GameServerAttributes, GameClientAttribute, CryptoAttribute};

// This trait defines the server interface that would be implemented by the GameServer
pub trait Server {
    fn initialize_pipeline(&self) -> io::Result<()>;
    fn connect(&self) -> io::Result<()>;
    fn disconnect(&self) -> io::Result<()>;
}