use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use log::{error, info, debug};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use bytes::BytesMut;

use crate::core::configuration_manager::ConfigurationManager;
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
        
        // Store the runtime in the struct
        // We need interior mutability since we have an immutable reference to self
        let this = unsafe { &*(self as *const _ as *mut GameServer) };
        this.runtime = Some(runtime);
        
        // Get a reference to the runtime we just stored
        let runtime = self.runtime.as_ref().unwrap();
        
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
        // Initialize configuration manager with the proper config path
        let config = ConfigurationManager::new("config.ini").expect("Failed to load configuration");
        
        // Get thread counts from config
        let boss_threads = (config.get_int("io.bossgroup.threads").unwrap_or(1)) as usize;
        let worker_threads = (config.get_int("io.workergroup.threads").unwrap_or(4)) as usize;
        
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
        // Wrap the socket in an Arc<Mutex> for shared access
        let socket = Arc::new(Mutex::new(socket));
        
        // Get configuration for debug settings
        let config = ConfigurationManager::new("config.ini").expect("Failed to load configuration");
        let debug_enabled = config.get_bool("debug.mode").unwrap_or(false);
        
        // Create a reference to self for the handlers
        let game_server_ref = Arc::new(Self {
            name: String::from("Game Server"),
            host: String::new(),
            port: 0,
            boss_threads: 0,
            worker_threads: 0,
            runtime: None,
            packet_manager: packet_manager.clone(),
            game_client_manager: game_client_manager.clone(),
        });
        
        // Create the handler chain
        let policy_decoder = GamePolicyDecoder::new();
        let frame_decoder = GameByteFrameDecoder::new();
        let byte_decoder = GameByteDecoder::new();
        
        // Optional message logger for debug
        let message_logger = if debug_enabled {
            Some(GameClientMessageLogger::new(game_server_ref.clone()))
        } else {
            None
        };
        
        // Rate limiter and message handler
        let rate_limiter = GameMessageRateLimit::new();
        let message_handler = GameMessageHandler::new(game_server_ref.clone());
        
        // Register the client channel
        if let Err(e) = message_handler.channel_registered(socket.clone()).await {
            error!("Failed to register client: {}", e);
            return;
        }
        
        // Set up async processing loop for the connection
        loop {
            // Read data from the socket
            let mut socket_guard = match socket.lock().await {
                Ok(guard) => guard,
                Err(_) => {
                    error!("Failed to lock socket mutex");
                    break;
                }
            };
            
            // Create a buffer to read into
            let mut buf = BytesMut::with_capacity(4096);
            
            // Read from socket
            match socket_guard.read_buf(&mut buf).await {
                Ok(0) => {
                    // Connection closed
                    debug!("Connection closed by client: {}", addr);
                    break;
                },
                Ok(_) => {
                    // Process the data through the decoder chain
                    // First, check for policy request
                    match policy_decoder.decode(&mut buf) {
                        Ok(Some((policy_data, is_policy))) => {
                            if is_policy {
                                // Send policy response and close connection
                                if let Err(e) = socket_guard.write_all(&policy_data).await {
                                    error!("Failed to send policy response: {}", e);
                                }
                                break; // Close connection after policy response
                            } else {
                                // Continue with frame decoding
                                let mut framed_data = policy_data;
                                
                                // Apply frame decoder
                                if let Ok(Some(frame)) = frame_decoder.decode(&mut framed_data) {
                                    // Apply byte decoder
                                    if let Ok(Some(message)) = byte_decoder.decode(&mut frame) {
                                        // Apply message logger if enabled
                                        let logged_message = if let Some(ref mut logger) = message_logger {
                                            match logger.decode(&mut BytesMut::new()) {
                                                Ok(Some(m)) => m,
                                                _ => message,
                                            }
                                        } else {
                                            message
                                        };
                                        
                                        // Apply rate limiter
                                        if let Ok(Some(rate_limited_message)) = rate_limiter.decode(&mut BytesMut::new(), Some(logged_message)) {
                                            // Handle the message
                                            if let Err(e) = message_handler.handle_message(socket.clone(), rate_limited_message).await {
                                                error!("Error handling message: {}", e);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            error!("Error in policy decoder: {}", e);
                            break;
                        },
                        _ => {} // No data to process
                    }
                },
                Err(e) => {
                    // Handle error
                    if let Err(handle_err) = message_handler.exception_caught(socket.clone(), e).await {
                        error!("Error handling exception: {}", handle_err);
                    }
                    break;
                },
            }
            
            // Check if we should stop
            if crate::is_shutting_down() {
                break;
            }
        }
        
        // Connection closed, unregister the channel
        if let Err(e) = message_handler.channel_unregistered(socket.clone()).await {
            error!("Failed to unregister client: {}", e);
        }
    }
    
    pub fn get_packet_manager(&self) -> Arc<Mutex<PacketManager>> {
        self.packet_manager.clone()
    }
    
    pub fn get_game_client_manager(&self) -> Arc<Mutex<GameClientManager>> {
        self.game_client_manager.clone()
    }
}
