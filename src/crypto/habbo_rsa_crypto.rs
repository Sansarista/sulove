use crate::crypto::exceptions::HabboCryptoException;
use num_bigint::BigInt;
use rand::{thread_rng, Rng};
use std::io::Cursor;
use std::io::Write;

pub struct HabboRSACrypto {
    e: BigInt,
    n: BigInt,
    d: Option<BigInt>,
    block_size: usize,
}

impl HabboRSACrypto {
    pub fn new(e: &str, n: &str) -> Self {
        let e_value = BigInt::parse_bytes(e.as_bytes(), 16).unwrap();
        let n_value = BigInt::parse_bytes(n.as_bytes(), 16).unwrap();
        let block_size = (calculate_bigint_bits(&n_value) + 7) / 8;

        HabboRSACrypto {
            e: e_value,
            n: n_value,
            d: None,
            block_size,
        }
    }

    pub fn new_with_private_key(e: &str, n: &str, d: &str) -> Self {
        let e_value = BigInt::parse_bytes(e.as_bytes(), 16).unwrap();
        let n_value = BigInt::parse_bytes(n.as_bytes(), 16).unwrap();
        let d_value = BigInt::parse_bytes(d.as_bytes(), 16).unwrap();
        let block_size = (calculate_bigint_bits(&n_value) + 7) / 8;

        HabboRSACrypto {
            e: e_value,
            n: n_value,
            d: Some(d_value),
            block_size,
        }
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, HabboCryptoException> {
        self.do_encrypt(data, true, 2)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, HabboCryptoException> {
        self.do_decrypt(data, false, 2)
    }

    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, HabboCryptoException> {
        self.do_encrypt(data, false, 1)
    }

    pub fn verify(&self, data: &[u8]) -> Result<Vec<u8>, HabboCryptoException> {
        self.do_decrypt(data, true, 1)
    }

    fn do_public(&self, x: &BigInt) -> BigInt {
        x.modpow(&self.e, &self.n)
    }

    fn do_private(&self, x: &BigInt) -> Result<BigInt, HabboCryptoException> {
        match &self.d {
            Some(d) => Ok(x.modpow(d, &self.n)),
            None => Err(HabboCryptoException::new("Private key not available")),
        }
    }

    fn do_encrypt(&self, data: &[u8], is_public: bool, pad_type: u8) -> Result<Vec<u8>, HabboCryptoException> {
        let mut dst = Vec::new();
        let bl = self.block_size;
        let end = data.len();
        let mut pos = 0;

        while pos < end {
            let padded = Self::pkcs1_pad(data, &mut pos, end, bl, pad_type);
            let block = BigInt::from_bytes_be(num_bigint::Sign::Plus, &padded);
            let chunk = if is_public {
                self.do_public(&block)
            } else {
                self.do_private(&block)?
            };

            // Calculate number of leading zeros needed
            let chunk_bytes = chunk.to_bytes_be().1;
            let leading_zeros = bl.saturating_sub(chunk_bytes.len());
            
            // Write leading zeros
            for _ in 0..leading_zeros {
                dst.push(0x00);
            }
            
            // Write chunk bytes
            dst.extend_from_slice(&chunk_bytes);
        }

        Ok(dst)
    }

    fn do_decrypt(&self, data: &[u8], is_public: bool, pad_type: u8) -> Result<Vec<u8>, HabboCryptoException> {
        if data.len() % self.block_size != 0 {
            return Err(HabboCryptoException::new(&format!(
                "Decryption data was not in blocks of {} bytes, total {}.",
                self.block_size, data.len()
            )));
        }

        let mut dst = Vec::new();
        let end = data.len();
        let mut pos = 0;

        while pos < end {
            let mut block_data = vec![0; self.block_size];
            block_data.copy_from_slice(&data[pos..pos + self.block_size]);

            let block = BigInt::from_bytes_be(num_bigint::Sign::Plus, &block_data);
            let chunk = if is_public {
                self.do_public(&block)
            } else {
                self.do_private(&block)?
            };

            let unpadded = Self::pkcs1_unpad(&chunk.to_bytes_be().1, self.block_size, pad_type)?;
            pos += self.block_size;
            dst.extend_from_slice(&unpadded);
        }

        Ok(dst)
    }

    fn pkcs1_pad(src: &[u8], pos: &mut usize, end: usize, n: usize, pad_type: u8) -> Vec<u8> {
        let mut result = vec![0; n];
        let p = *pos;
        let bounded_end = std::cmp::min(end, std::cmp::min(src.len(), p + n - 11));
        *pos = bounded_end;
        let mut i = bounded_end - 1;
        let mut n_index = n;

        while i >= p && n_index > 11 {
            n_index -= 1;
            result[n_index] = src[i];
            i -= 1;
        }

        n_index -= 1;
        result[n_index] = 0;

        if pad_type == 2 {
            let mut rng = thread_rng();
            while n_index > 2 {
                n_index -= 1;
                result[n_index] = rng.gen_range(1..=255) as u8;
            }
        } else {
            while n_index > 2 {
                n_index -= 1;
                result[n_index] = 0xFF;
            }
        }

        n_index -= 1;
        result[n_index] = pad_type;
        n_index -= 1;
        result[n_index] = 0;

        result
    }

    fn pkcs1_unpad(b: &[u8], n: usize, pad_type: u8) -> Result<Vec<u8>, HabboCryptoException> {
        let mut result = vec![0; n];
        let mut result_pos = 0;
        let mut i = 0;

        while i < b.len() && b[i] == 0 {
            i += 1;
        }

        if b.len() - i != n - 1 || b[i] != pad_type {
            return Err(HabboCryptoException::new(&format!(
                "PKCS#1 unpad: i={}, expected b[i]=={}, got b[i]={}",
                i, pad_type, b[i]
            )));
        }

        i += 1;

        while i < b.len() && b[i] != 0 {
            i += 1;
            if i >= b.len() {
                return Err(HabboCryptoException::new(&format!(
                    "PKCS#1 unpad: i={}, b[i-1]!=0 (={})",
                    i,
                    b[i - 1]
                )));
            }
        }

        while i + 1 < b.len() {
            i += 1;
            result[result_pos] = b[i];
            result_pos += 1;
        }

        let mut result_copy = vec![0; result_pos];
        result_copy.copy_from_slice(&result[0..result_pos]);

        Ok(result_copy)
    }
}

/// Helper function to calculate the number of bits in a BigInt
fn calculate_bigint_bits(big_int: &BigInt) -> usize {
    let (_, bytes) = big_int.to_bytes_be();
    let mut bits = bytes.len() * 8;
    
    if let Some(&first_byte) = bytes.first() {
        let mut mask = 0x80;
        while mask > 0 && first_byte & mask == 0 {
            bits -= 1;
            mask >>= 1;
        }
    }
    
    bits
}