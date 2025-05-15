use std::io::{self, BufRead};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;

use log::{debug, info, warn};
use tokio::runtime::Runtime;
use num_cpus;
use chrono::Local;
use once_cell::sync::OnceCell;

// Module imports
mod core;
mod crypto;
mod database;
mod habbohotel;
mod messages;
mod networking; 
mod threading;
mod util;

// Constants
const PREVIEW: &str = "";
const VERSION: &str = "Sulove Rust Emulator";
//rahmed is best rust dev

// Logo
const LOGO: &str = r#"
███████╗██╗   ██╗██╗      ██████╗ ██╗   ██╗███████╗
██╔════╝██║   ██║██║     ██╔═══██╗██║   ██║██╔════╝
███████╗██║   ██║██║     ██║   ██║██║   ██║█████╗  
╚════██║██║   ██║██║     ██║   ██║╚██╗ ██╔╝██╔══╝  
███████║╚██████╔╝███████╗╚██████╔╝ ╚████╔╝ ███████╗
╚══════╝ ╚═════╝ ╚══════╝ ╚═════╝   ╚═══╝  ╚══════╝
"#;

// Global statj
static CONFIG_MANAGER: OnceCell<Arc<core::configuration_manager::ConfigurationManager>> = OnceCell::new();
static DATABASE: OnceCell<Arc<database::database::Database>> = OnceCell::new();
static GAME_ENVIRONMENT: OnceCell<Arc<habbohotel::game_enviroment::GameEnvironment>> = OnceCell::new();
static IS_READY: AtomicBool = AtomicBool::new(false);
static IS_SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);
static TIME_STARTED: OnceCell<u64> = OnceCell::new();

pub fn get_config() -> Arc<core::configuration_manager::ConfigurationManager> {
    CONFIG_MANAGER.get().expect("ConfigurationManager not initialized").clone()
}

pub fn get_database() -> Arc<database::database::Database> {
    DATABASE.get().expect("Database not initialized").clone()
}

pub fn get_game_environment() -> Arc<habbohotel::game_enviroment::GameEnvironment> {
    GAME_ENVIRONMENT.get().expect("GameEnvironment not initialized").clone()
}

pub fn is_ready() -> bool {
    IS_READY.load(Ordering::SeqCst)
}

pub fn is_shutting_down() -> bool {
    IS_SHUTTING_DOWN.load(Ordering::SeqCst)
}

pub fn get_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Print logo and version info
    println!("{}", LOGO);
    info!("Version: {}", VERSION);
    info!("This project is for educational purposes only.");
    info!("Follow our development at https://github.com/sulove-rust/sulove");
    
    let start_time = SystemTime::now();
    
    // Initialize configuration
    let config = Arc::new(core::configuration_manager::ConfigurationManager::new("config.ini")?);
    CONFIG_MANAGER.set(config.clone()).expect("Failed to set ConfigurationManager");
    
    // Initialize database
    let database = Arc::new(database::database::Database::new(config.clone())?);
    DATABASE.set(database.clone()).expect("Failed to set Database");
    
    // Load configuration from database
    config.load_from_database()?;
    
    // Initialize thread pool
    let thread_count = config.get_int("runtime.threads").unwrap_or_else(|_| num_cpus::get() as i32 * 2);
    let threading = Arc::new(threading::thread_polling::ThreadPooling::new(thread_count as usize));
    
    // Initialize game server
    let game_host = config.get_string("game.host").unwrap_or_else(|_| "127.0.0.1".to_string());
    let game_port = config.get_int("game.port").unwrap_or_else(|_| 30000);
    let game_server = Arc::new(networking::gameserver::GameServer::new(game_host, game_port as u16));
    
    // Initialize RCON server
    let rcon_host = config.get_string("rcon.host").unwrap_or_else(|_| "127.0.0.1".to_string());
    let rcon_port = config.get_int("rcon.port").unwrap_or_else(|_| 30001);
    let rcon_server = Arc::new(networking::rconserver::RCONServer::new(rcon_host, rcon_port as u16));
    
    // Initialize game environment
    let game_environment = Arc::new(habbohotel::game_enviroment::GameEnvironment::new());
    GAME_ENVIRONMENT.set(game_environment.clone()).expect("Failed to set GameEnvironment");
    
    // Load game environment
    game_environment.load()?;
    
    // Connect servers
    game_server.initialize_pipeline()?;
    game_server.connect()?;
    
    rcon_server.initialize_pipeline()?;
    rcon_server.connect()?;
    
    // Set up cleaner thread
    let _cleaner = core::cleaner_thread::CleanerThread::new();
    
    // Calculate startup time
    let elapsed = start_time.elapsed()?.as_millis();
    
    info!("Sulove has successfully loaded.");
    info!("System launched in: {}ms. Using {} threads!", elapsed, thread_count);
    
    // Set debugging mode
    let debugging = config.get_bool("debug.mode").unwrap_or(false);
    if debugging {
        debug!("Debugging enabled.");
    }
    
    // Set ready state and record start time
    IS_READY.store(true, Ordering::SeqCst);
    TIME_STARTED.set(get_unix_timestamp()).expect("Failed to set start time");
    
    // Check if console mode is enabled
    if config.get_bool("console.mode").unwrap_or(true) {
        let stdin = io::stdin();
        let mut reader = stdin.lock().lines();
        
        println!("Waiting for command: ");
        
        while !IS_SHUTTING_DOWN.load(Ordering::SeqCst) && IS_READY.load(Ordering::SeqCst) {
            if let Some(Ok(line)) = reader.next() {
                // Handle console commands
                // TODO: Implement console command handling
                println!("Command received: {}", line);
                println!("Waiting for command: ");
            }
        }
    } else {
        // If console mode is disabled, just keep the main thread alive
        while !IS_SHUTTING_DOWN.load(Ordering::SeqCst) && IS_READY.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_secs(1));
        }
    }
    
    Ok(())
}

// Shutdown function
pub fn dispose() {
    if IS_SHUTTING_DOWN.load(Ordering::SeqCst) {
        return;
    }
    
    IS_SHUTTING_DOWN.store(true, Ordering::SeqCst);
    info!("Shutting down Sulove...");
    
    // TODO: Implement proper shutdown sequence
    // - Stop accepting new connections
    // - Save user data
    // - Close database connections
    // - Shutdown thread pools
    
    info!("Sulove has been shut down.");
}