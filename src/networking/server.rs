use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

// This is a trait that defines the common functionality for all servers
pub trait Server {
    fn initialize_pipeline(&self) -> io::Result<()>;
    fn connect(&self) -> io::Result<()>;
    fn disconnect(&self) -> io::Result<()>;
}

// This would be a base implementation for servers
pub struct BaseServer {
    host: String,
    port: u16,
}

impl BaseServer {
    pub fn new(host: String, port: u16) -> Self {
        BaseServer { host, port }
    }
    
    pub fn get_host(&self) -> &str {
        &self.host
    }
    
    pub fn get_port(&self) -> u16 {
        self.port
    }
}