pub struct PacketUtils;

impl PacketUtils {
    // Format a packet buffer for readable output
    pub fn format_packet(buffer: &[u8]) -> String {
        let mut result = String::from_utf8_lossy(buffer).to_string();
        
        // Replace control characters with their numeric representation
        for i in 0..32 {
            let control_char = char::from(i);
            result = result.replace(control_char, &format!("[{}]", i));
        }
        
        result
    }
}