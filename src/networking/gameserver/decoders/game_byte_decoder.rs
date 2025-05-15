use bytes::{BytesMut, Buf};
use tokio_util::codec::Decoder;
use std::io;

use crate::messages::client_message::ClientMessage;

pub struct GameByteDecoder;

impl GameByteDecoder {
    pub fn new() -> Self {
        Self {}
    }
}

impl Decoder for GameByteDecoder {
    type Item = ClientMessage;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Need at least 2 bytes for the header
        if src.len() < 2 {
            return Ok(None);
        }

        // Read the header (message ID)
        let header = u16::from_be_bytes([src[0], src[1]]);
        
        // Remove header bytes from buffer
        src.advance(2);
        
        // Create a new client message with the remaining bytes
        let body = src.clone();
        src.clear(); // Clear the source buffer as we've consumed it
        
        Ok(Some(ClientMessage::new(header, body)))
    }
}