use std::collections::HashMap;
use std::sync::Arc;

use crate::messages::incoming::message_handler::MessageHandler;
use crate::messages::incoming::incoming::Incoming;
use crate::messages::outgoing::outgoing::Outgoing;

/// The PacketManager handles the registration and management of packet handlers
/// throughout the server application. It maps incoming packet IDs to their 
/// respective handlers and provides utility functions for packet processing.
pub struct PacketManager {
    /// Map of packet IDs to their corresponding handlers
    handlers: HashMap<i32, Arc<dyn MessageHandler + Send + Sync>>,
    /// Mapping of incoming packet IDs to their string names for debugging
    incoming_names: HashMap<i32, String>,
    /// Mapping of outgoing packet IDs to their string names for debugging
    outgoing_names: HashMap<i32, String>,
}

impl PacketManager {
    /// Creates a new PacketManager instance with empty mappings
    pub fn new() -> Self {
        PacketManager {
            handlers: HashMap::new(),
            incoming_names: HashMap::new(),
            outgoing_names: HashMap::new(),
        }
    }
    
    /// Registers a message handler for a specific incoming packet ID
    pub fn register<H>(&mut self, header: Incoming, handler: H) 
    where 
        H: MessageHandler + Send + Sync + 'static 
    {
        let header_id = header as i32;
        self.handlers.insert(header_id, Arc::new(handler));
    }
    
    /// Gets a handler for a specific packet ID, if registered
    pub fn get_handler(&self, header: i32) -> Option<Arc<dyn MessageHandler + Send + Sync>> {
        self.handlers.get(&header).cloned()
    }
    
    /// Initializes the packet name mappings for debugging purposes
    pub fn initialize_packet_names(&mut self) {
        // To populate packet name mappings, you'll need to manually map
        // the enum variants to their names. In a real implementation,
        // you could use a macro to generate this code automatically.
        
        // Example implementation - you'll need to adapt this to your actual enums
        // For Incoming packets
        self.register_incoming_names();
        
        // For Outgoing packets
        self.register_outgoing_names();
    }
    
    /// Helper method to register incoming packet names
    fn register_incoming_names(&mut self) {
        // This is a placeholder - you'll need to populate with your actual enum variants
        // Example: 
        // self.incoming_names.insert(Incoming::Login as i32, "Login".to_string());
        // self.incoming_names.insert(Incoming::GetRooms as i32, "GetRooms".to_string());
    }
    
    /// Helper method to register outgoing packet names
    fn register_outgoing_names(&mut self) {
        // This is a placeholder - you'll need to populate with your actual enum variants
        // Example:
        // self.outgoing_names.insert(Outgoing::AuthOk as i32, "AuthOk".to_string());
        // self.outgoing_names.insert(Outgoing::RoomList as i32, "RoomList".to_string());
    }
    
    /// Gets the name of an incoming packet by ID
    pub fn get_incoming_packet_name(&self, header: i32) -> Option<&String> {
        self.incoming_names.get(&header)
    }
    
    /// Gets the name of an outgoing packet by ID
    pub fn get_outgoing_packet_name(&self, header: i32) -> Option<&String> {
        self.outgoing_names.get(&header)
    }
}
