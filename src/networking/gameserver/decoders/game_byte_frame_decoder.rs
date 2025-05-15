use bytes::{BytesMut, Buf, BufMut};
use tokio_util::codec::Decoder;
use std::io;

pub struct GameByteFrameDecoder {
    // Constants for frame decoding
    max_packet_length: usize,
    length_field_offset: usize,
    length_field_length: usize,
    length_field_adjustment: usize,
    initial_bytes_to_strip: usize,
    // Buffer for accumulating data
    cumulation: Option<BytesMut>,
}

impl GameByteFrameDecoder {
    pub fn new() -> Self {
        // These values match the Java implementation
        // MAX_PACKET_LENGTH is based on the maximum camera PNG size.
        // Maximum camera packet is 320 * 320 Pixel * 4 Bytes per Pixel = 409600.
        // Adding some for overhead 409600 + 8192 = 417792
        let max_packet_length = 417792;
        let length_field_offset = 0;
        let length_field_length = 4;
        let length_field_adjustment = 0;
        let initial_bytes_to_strip = 4;

        Self {
            max_packet_length,
            length_field_offset,
            length_field_length,
            length_field_adjustment,
            initial_bytes_to_strip,
            cumulation: None,
        }
    }

    fn extract_frame(&mut self, buffer: &mut BytesMut, index: usize) -> Option<BytesMut> {
        let frame_length = self.get_frame_length(buffer, index);
        
        // Frame length must be valid
        if frame_length < 0 || frame_length > self.max_packet_length as i32 {
            return None;
        }
        
        // Calculate the total length of the frame including the header
        let frame_length = frame_length as usize;
        let total_length = self.length_field_offset + self.length_field_length + frame_length;
        
        // Check if we have enough bytes
        if buffer.len() < total_length {
            return None;
        }
        
        // Extract the frame and strip initial bytes if required
        let mut frame = buffer.split_to(total_length);
        if self.initial_bytes_to_strip > 0 {
            frame.advance(self.initial_bytes_to_strip);
        }
        
        Some(frame)
    }
    
    fn get_frame_length(&self, buffer: &BytesMut, index: usize) -> i32 {
        if self.length_field_length == 4 {
            // Read a 4-byte integer
            let offset = index + self.length_field_offset;
            let bytes = [buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]];
            let length = i32::from_be_bytes(bytes);
            length + self.length_field_adjustment as i32
        } else {
            // For other field lengths, provide implementation as needed
            // This example only handles 4-byte length fields
            panic!("Unsupported length field length: {}", self.length_field_length);
        }
    }
}

impl Decoder for GameByteFrameDecoder {
    type Item = BytesMut;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // If src is empty, return None
        if src.is_empty() {
            return Ok(None);
        }
        
        // Accumulate the data
        let mut buffer = match self.cumulation.take() {
            Some(mut buf) => {
                buf.put_slice(src.as_ref());
                src.clear();
                buf
            }
            None => {
                let buf = src.clone();
                src.clear();
                buf
            }
        };
        
        // Check if we have enough bytes for the length field
        if buffer.len() < self.length_field_offset + self.length_field_length {
            self.cumulation = Some(buffer);
            return Ok(None);
        }
        
        // Extract the frame
        match self.extract_frame(&mut buffer, 0) {
            Some(frame) => {
                // Keep any remaining bytes in the cumulation buffer
                if !buffer.is_empty() {
                    self.cumulation = Some(buffer);
                }
                Ok(Some(frame))
            }
            None => {
                // Not enough data yet, keep accumulating
                self.cumulation = Some(buffer);
                Ok(None)
            }
        }
    }
}