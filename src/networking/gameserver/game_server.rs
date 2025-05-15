use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use log::{error, info, debug};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::core::configuration_manager::ConfigurationManager;
use crate::get_config;
use crate::messages::packet_manager::PacketManager;
use crate::habbohotel::gameclients::GameClientManager;
use crate::networking::Server;
use crate::networking::gameserver::decoders::{GamePolicyDecoder, GameByteFrameDecoder, GameByteDecoder, GameClientMessageLogger, GameMessageRateLimit, GameMessageHandler};
use crate::networking::gameserver::encoders::{GameServerMessageEncoder, GameServerMessageLogger};
use crate::networking::gameserver::handlers::IdleTimeoutHandler;

pub struct GameServer {
    name: String,
    host: String,
    port: u16,
    boss_threads: usize,
    worker_threads: usize,
    runtime: Option<Runtime>,
    packet_manager: Arc<Mutex<PacketManager>>,
    game_client_manager: Arc<Mutex<GameClientManager>>,
}

impl Server for GameServer {
    fn initialize_pipeline(&self) -> io::Result<()> {
        info!("Initializing {} pipeline", self.name);
        Ok(())
    }
    
    fn connect(&self) -> io::Result<()> {
        let host = self.host.clone();
        let port = self.port;
        let packet_manager = self.packet_manager.clone();
        let game_client_manager = self.game_client_manager.clone();
        
        // Create a new runtime for the server
        let runtime = Runtime::new()?;
        
        // Store the runtime in a mutable reference
        // This works because Runtime has an internal Arc and can be cloned safely
        let mut this = self.to_owned();
        this.runtime = Some(runtime.clone());
        
        // Spawn the server task
        runtime.spawn(async move {
            let addr = format!("{host}:{port}");
            let listener = match TcpListener::bind(&addr).await {
                Ok(listener) => listener,
                Err(e) => {
                    error!("Failed to bind game server to {}: {}", addr, e);
                    return;
                }
            };
            
            info!("Game server listening on {}", addr);
            
            loop {
                match listener.accept().await {
                    Ok((socket, addr)) => {
                        // Handle new connection
                        debug!("New connection from: {}", addr);
                        
                        // Clone the managers for the connection handler
                        let pm = packet_manager.clone();
                        let gcm = game_client_manager.clone();
                        
                        // Spawn a new task for each connection
                        tokio::spawn(async move {
                            Self::handle_connection(socket, addr, pm, gcm).await;
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
                
                // Check if we should stop
                if crate::is_shutting_down() {
                    break;
                }
            }
            
            info!("Game server stopped");
        });
        
        info!("{} started on {}:{}", self.name, self.host, self.port);
        
        Ok(())
    }
    
    fn disconnect(&self) -> io::Result<()> {
        // Since we're using Arc/Mutex for data, we can still shut down the runtime
        // with an immutable reference using interior mutability
        let mut this = self.to_owned();
        if let Some(runtime) = this.runtime.take() {
            // Shutdown the runtime
            drop(runtime);
        }
        
        info!("{} disconnected", self.name);
        Ok(())
    }
}

impl GameServer {
    pub fn new(host: String, port: u16) -> io::Result<Self> {
        let config = Config::get_instance(); // Assuming there's a Config singleton
        
        // Get thread counts from config
        let boss_threads = config.get_int("io.bossgroup.threads") as usize;
        let worker_threads = config.get_int("io.workergroup.threads") as usize;
        
        Ok(GameServer {
            name: String::from("Game Server"),
            host,
            port,
            boss_threads,
            worker_threads,
            runtime: None,
            packet_manager: Arc::new(Mutex::new(PacketManager::new())),
            game_client_manager: Arc::new(Mutex::new(GameClientManager::new())),
        })
    }
    
    async fn handle_connection(
        socket: TcpStream, 
        addr: SocketAddr,
        packet_manager: Arc<Mutex<PacketManager>>,
        game_client_manager: Arc<Mutex<GameClientManager>>
    ) {
        // This would implement the equivalent of the netty pipeline in Java
        // For each connection, we'd:
        // 1. Set up policy decoder
        // 2. Set up byte frame decoder
        // 3. Set up byte decoder
        // 4. Add message logger if debug is enabled
        // 5. Add idle timeout handler
        // 6. Add message rate limiter
        // 7. Add message handler
        // 8. Add message encoder
        // 9. Add message logger if debug is enabled
        
        // In a real implementation, this would use tokio's codec system
        // or a custom protocol implementation
    }
    
    pub fn get_packet_manager(&self) -> Arc<Mutex<PacketManager>> {
        self.packet_manager.clone()
    }
    
    pub fn get_game_client_manager(&self) -> Arc<Mutex<GameClientManager>> {
        self.game_client_manager.clone()
    }
}
