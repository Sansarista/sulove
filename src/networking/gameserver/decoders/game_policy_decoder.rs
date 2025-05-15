use bytes::{BytesMut, Buf, BufMut};
use tokio_util::codec::Decoder;
use std::io;

pub struct GamePolicyDecoder {
    policy_sent: bool,
}

impl GamePolicyDecoder {
    pub fn new() -> Self {
        Self {
            policy_sent: false,
        }
    }
    
    // The policy XML string to be sent when requested
    fn get_policy_string() -> String {
        String::from("<?xml version=\"1.0\"?>\n\
          <!DOCTYPE cross-domain-policy SYSTEM \"/xml/dtds/cross-domain-policy.dtd\">\n\
          <cross-domain-policy>\n\
          <allow-access-from domain=\"*\" to-ports=\"1-31111\" />\n\
          </cross-domain-policy>\u{0}")
    }
    
    // Function to check if a buffer starts with a policy request
    fn is_policy_request(buffer: &BytesMut) -> bool {
        if buffer.is_empty() {
            return false;
        }
        
        // Policy requests start with '<'
        buffer[0] == b'<'
    }
}

impl Decoder for GamePolicyDecoder {
    type Item = (BytesMut, bool); // (Data, is_policy)
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        
        // Check if this is a policy request
        if !self.policy_sent && Self::is_policy_request(src) {
            // Mark that we've handled a policy request
            self.policy_sent = true;
            
            // Clear the buffer as we've processed the policy request
            src.clear();
            
            // Create a buffer with the policy response
            let mut policy_buffer = BytesMut::new();
            policy_buffer.put_slice(Self::get_policy_string().as_bytes());
            
            // Return the policy response with a flag indicating it's a policy response
            return Ok(Some((policy_buffer, true)));
        }
        
        // Not a policy request, continue normal processing
        // Clone the buffer and clear the source
        let data = src.clone();
        src.clear();
        
        // Return the data with a flag indicating it's not a policy response
        Ok(Some((data, false)))
    }
}