use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::sleep;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use log::{debug, warn};

use crate::messages::client_message::ClientMessage;

/// The header value for Pong packets. Replace with the real value.
const PONG_EVENT_HEADER: u16 = 1234; // TODO: Replace with actual value

/// Sends a ping to the client. Replace with actual implementation.
pub async fn send_ping(_stream: &Arc<AsyncMutex<TcpStream>>) {
    // TODO: Implement actual ping sending logic
    debug!("Sending ping to client");
}

/// IdleTimeoutHandler for managing ping/pong and idle timeouts.
pub struct IdleTimeoutHandler {
    /// How often to send a ping (nanoseconds)
    ping_schedule: Duration,
    /// How long to wait for a pong (nanoseconds)
    pong_timeout: Duration,
    /// Last time a pong was received
    last_pong: Arc<Mutex<Instant>>,
    /// Handle to the ping task
    ping_task: Option<JoinHandle<()>>,
    /// State: 0 = none, 1 = initialized, 2 = destroyed
    state: Arc<Mutex<u8>>,
}

impl IdleTimeoutHandler {
    pub fn new(ping_schedule_secs: u64, pong_timeout_secs: u64) -> Self {
        let min_timeout = Duration::from_millis(1);
        Self {
            ping_schedule: std::cmp::max(min_timeout, Duration::from_secs(ping_schedule_secs)),
            pong_timeout: std::cmp::max(min_timeout, Duration::from_secs(pong_timeout_secs)),
            last_pong: Arc::new(Mutex::new(Instant::now())),
            ping_task: None,
            state: Arc::new(Mutex::new(0)),
        }
    }

    pub fn initialize(&mut self, stream: Arc<AsyncMutex<TcpStream>>) {
        let mut state = self.state.lock().unwrap();
        if *state == 1 || *state == 2 {
            return;
        }
        *state = 1;
        drop(state);
        {
            let mut last_pong = self.last_pong.lock().unwrap();
            *last_pong = Instant::now();
        }
        let ping_schedule = self.ping_schedule;
        let pong_timeout = self.pong_timeout;
        let last_pong = Arc::clone(&self.last_pong);
        let state = Arc::clone(&self.state);
        // Spawn ping task
        self.ping_task = Some(tokio::spawn(async move {
            loop {
                sleep(ping_schedule).await;
                if *state.lock().unwrap() != 1 {
                    break;
                }
                let now = Instant::now();
                let last = *last_pong.lock().unwrap();
                if now.duration_since(last) > pong_timeout {
                    warn!("Pong timeout, closing connection");
                    // Close the connection
                    let _ = stream.lock().await.shutdown().await;
                    break;
                }
                send_ping(&stream).await;
            }
        }));
    }

    pub fn destroy(&mut self) {
        let mut state = self.state.lock().unwrap();
        *state = 2;
        drop(state);
        if let Some(handle) = self.ping_task.take() {
            handle.abort();
        }
    }

    /// Call this when a message is received from the client.
    pub fn on_client_message(&self, msg: &ClientMessage) {
        if msg.get_header() == PONG_EVENT_HEADER {
            let mut last_pong = self.last_pong.lock().unwrap();
            *last_pong = Instant::now();
        }
    }
}

// Usage example (integration required):
// let mut handler = IdleTimeoutHandler::new(30, 60);
// handler.initialize(client_stream.clone());
// ... on message: handler.on_client_message(&msg);
// ... on disconnect: handler.destroy();
