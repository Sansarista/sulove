use num_bigint::BigInt;

/// Utility functions for working with BigInt values
pub struct BigIntegerUtils;

impl BigIntegerUtils {
    /// Convert a BigInt to an unsigned byte array
    /// 
    /// This removes any leading zero byte that might be present
    /// in the BigInt's two's complement representation
    pub fn to_unsigned_byte_array(big_integer: &BigInt) -> Vec<u8> {
        let (_, bytes) = big_integer.to_bytes_be();
        
        // Check if the first byte is zero (sign byte) and remove it
        if !bytes.is_empty() && bytes[0] == 0 {
            bytes[1..].to_vec()
        } else {
            bytes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_to_unsigned_byte_array() {
        // Test with a positive BigInt that needs leading zero removed
        let big_int = BigInt::parse_bytes(b"128", 10).unwrap();
        let bytes = BigIntegerUtils::to_unsigned_byte_array(&big_int);
        assert_eq!(bytes, vec![0x80]); // 128 as a single byte
        
        // Test with a BigInt that doesn't need leading zero removed
        let big_int = BigInt::parse_bytes(b"127", 10).unwrap();
        let bytes = BigIntegerUtils::to_unsigned_byte_array(&big_int);
        assert_eq!(bytes, vec![0x7F]); // 127 as a single byte
    }
}
