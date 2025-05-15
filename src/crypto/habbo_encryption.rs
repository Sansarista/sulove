use crate::crypto::habbo_rsa_crypto::HabboRSACrypto;
use crate::crypto::habbo_diffie_hellman::HabboDiffieHellman;
use std::sync::Arc;

pub struct HabboEncryption {
    crypto: Arc<HabboRSACrypto>,
    diffie: HabboDiffieHellman,
}

impl HabboEncryption {
    pub fn new(e: &str, n: &str, d: &str) -> Self {
        let crypto = Arc::new(HabboRSACrypto::new_with_private_key(e, n, d));
        let diffie = HabboDiffieHellman::new(Arc::clone(&crypto));
        
        HabboEncryption {
            crypto,
            diffie,
        }
    }

    pub fn get_crypto(&self) -> &Arc<HabboRSACrypto> {
        &self.crypto
    }

    pub fn get_diffie(&self) -> &HabboDiffieHellman {
        &self.diffie
    }
}