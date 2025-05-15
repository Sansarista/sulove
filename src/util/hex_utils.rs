use rand::{thread_rng, Rng};

pub struct HexUtils;

impl HexUtils {
    const HEX_CHARS: &'static [u8] = b"0123456789ABCDEF";
    
    pub fn to_hex(bytes: &[u8]) -> String {
        let mut hex = String::with_capacity(bytes.len() * 2);
        
        for &b in bytes {
            hex.push(char::from(Self::HEX_CHARS[(b >> 4) as usize]));
            hex.push(char::from(Self::HEX_CHARS[(b & 0x0F) as usize]));
        }
        
        hex
    }
    
    pub fn to_bytes(hex_string: &str) -> Result<Vec<u8>, String> {
        if hex_string.len() % 2 != 0 {
            return Err("Hex string must have an even length".to_string());
        }
        
        let mut bytes = Vec::with_capacity(hex_string.len() / 2);
        
        for i in (0..hex_string.len()).step_by(2) {
            let high = hex_string.chars().nth(i)
                .ok_or_else(|| format!("Invalid hex character at position {}", i))?;
            let low = hex_string.chars().nth(i + 1)
                .ok_or_else(|| format!("Invalid hex character at position {}", i + 1))?;
            
            let high_digit = char::to_digit(high, 16)
                .ok_or_else(|| format!("Invalid hex character: {}", high))?;
            let low_digit = char::to_digit(low, 16)
                .ok_or_else(|| format!("Invalid hex character: {}", low))?;
            
            bytes.push(((high_digit << 4) | low_digit) as u8);
        }
        
        Ok(bytes)
    }
    
    pub fn get_random(length: usize) -> String {
        let mut rng = thread_rng();
        let mut result = String::with_capacity(length);
        
        while result.len() < length {
            result.push_str(&format!("{:x}", rng.gen::<u32>()));
        }
        
        result[0..length].to_string()
    }
}