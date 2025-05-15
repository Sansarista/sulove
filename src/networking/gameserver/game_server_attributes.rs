use crate::habbohotel::gameclients::GameClient;
use crate::crypto::HabboRC4;

/// A Rust equivalent of Java's AttributeKey pattern
pub struct GameServerAttributes;

impl GameServerAttributes {
    /// Constants that serve as keys for the attribute system
    pub const CLIENT: &'static str = "GameClient";
    pub const CRYPTO_CLIENT: &'static str = "CryptoClient";
    pub const CRYPTO_SERVER: &'static str = "CryptoServer";
}

/// Manages client objects, designed to be stored in a connection's attributes
pub struct GameClientAttribute {
    /// Connection ID
    pub connection_id: u64,
    /// Client reference
    pub client: GameClient,
}

/// Manages crypto objects, designed to be stored in a connection's attributes
pub struct CryptoAttribute {
    /// Connection ID
    pub connection_id: u64,
    /// Crypto reference
    pub crypto: HabboRC4,
}
