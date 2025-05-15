use crate::crypto::habbo_rsa_crypto::HabboRSACrypto;
use num_bigint::{BigInt, RandBigInt};
use rand::{thread_rng, Rng};
use std::cmp::Ordering;
use std::sync::Arc;

pub struct HabboDiffieHellman {
    crypto: Arc<HabboRSACrypto>,
    dh_prime: BigInt,
    dh_generator: BigInt,
    dh_private: BigInt,
    dh_public: BigInt,
}

const DH_PRIMES_BIT_SIZE: u64 = 128;
const DH_KEY_BIT_SIZE: u64 = 128;

#[derive(Debug)]
pub enum HabboCryptoError {
    CryptoOperationFailed(String),
    InvalidInput(String),
}

impl HabboDiffieHellman {
    pub fn new(crypto: Arc<HabboRSACrypto>) -> Self {
        let mut instance = Self {
            crypto,
            dh_prime: BigInt::from(0),
            dh_generator: BigInt::from(0),
            dh_private: BigInt::from(0),
            dh_public: BigInt::from(0),
        };
        
        instance.generate_dh_primes();
        instance.generate_dh_keys();
        instance
    }

    pub fn get_dh_prime(&self) -> &BigInt {
        &self.dh_prime
    }

    pub fn get_dh_generator(&self) -> &BigInt {
        &self.dh_generator
    }

    fn generate_dh_primes(&mut self) {
        let mut rng = thread_rng();
        
        self.dh_prime = rng.gen_bigint(DH_PRIMES_BIT_SIZE);
        self.dh_generator = rng.gen_bigint(DH_PRIMES_BIT_SIZE);

        if self.dh_generator > self.dh_prime {
            std::mem::swap(&mut self.dh_prime, &mut self.dh_generator);
        }
    }

    fn generate_dh_keys(&mut self) {
        let mut rng = thread_rng();
        
        self.dh_private = rng.gen_bigint(DH_KEY_BIT_SIZE);
        // g^a mod p
        self.dh_public = self.dh_generator.modpow(&self.dh_private, &self.dh_prime);
    }

    fn encrypt_big_integer(&self, integer: &BigInt) -> Result<String, HabboCryptoError> {
        let str = integer.to_string();
        let bytes = str.as_bytes();
        
        match self.crypto.sign(bytes) {
            Ok(encrypted) => {
                let hex = hex::encode(&encrypted);
                Ok(hex)
            },
            Err(e) => Err(HabboCryptoError::CryptoOperationFailed(e.to_string())),
        }
    }

    fn decrypt_big_integer(&self, hex_str: &str) -> Result<BigInt, HabboCryptoError> {
        let bytes = match hex::decode(hex_str) {
            Ok(b) => b,
            Err(_) => return Err(HabboCryptoError::InvalidInput("Invalid hex string".to_string())),
        };
        
        match self.crypto.decrypt(&bytes) {
            Ok(decrypted) => {
                let int_str = match std::str::from_utf8(&decrypted) {
                    Ok(s) => s,
                    Err(_) => return Err(HabboCryptoError::InvalidInput("Invalid UTF-8".to_string())),
                };
                
                match int_str.parse::<BigInt>() {
                    Ok(num) => Ok(num),
                    Err(_) => Err(HabboCryptoError::InvalidInput("Failed to parse BigInt".to_string())),
                }
            },
            Err(e) => Err(HabboCryptoError::CryptoOperationFailed(e.to_string())),
        }
    }

    pub fn get_public_key(&self) -> Result<String, HabboCryptoError> {
        self.encrypt_big_integer(&self.dh_public)
    }

    pub fn get_signed_prime(&self) -> Result<String, HabboCryptoError> {
        self.encrypt_big_integer(&self.dh_prime)
    }

    pub fn get_signed_generator(&self) -> Result<String, HabboCryptoError> {
        self.encrypt_big_integer(&self.dh_generator)
    }

    pub fn do_handshake(&mut self, signed_prime: &str, signed_generator: &str) -> Result<(), HabboCryptoError> {
        self.dh_prime = self.decrypt_big_integer(signed_prime)?;
        self.dh_generator = self.decrypt_big_integer(signed_generator)?;

        let two = BigInt::from(2);
        
        if self.dh_prime.cmp(&two) != Ordering::Greater {
            return Err(HabboCryptoError::InvalidInput(format!(
                "Prime cannot be <= 2!\nPrime: {}", self.dh_prime
            )));
        }

        if self.dh_generator.cmp(&self.dh_prime) != Ordering::Less {
            return Err(HabboCryptoError::InvalidInput(format!(
                "Generator cannot be >= Prime!\nPrime: {}\nGenerator: {}",
                self.dh_prime, self.dh_generator
            )));
        }

        self.generate_dh_keys();
        Ok(())
    }

    pub fn get_shared_key(&self, public_key_str: &str) -> Result<Vec<u8>, HabboCryptoError> {
        let public_key = self.decrypt_big_integer(public_key_str)?;
        let shared_key = public_key.modpow(&self.dh_private, &self.dh_prime);
        
        // Convert BigInt to unsigned byte array
        let bytes = shared_key.to_bytes_be().1;
        Ok(bytes)
    }
}