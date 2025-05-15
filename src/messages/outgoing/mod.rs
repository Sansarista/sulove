//! Module for handling outgoing messages to clients

// Re-export all submodules
pub mod achievements;
pub mod camera;
pub mod catalog;
pub mod crafting;
pub mod events;
pub mod floorplaneditor;
pub mod friends;
pub mod gamecenter;
pub mod generic;
pub mod guardians;
pub mod guides;
pub mod guilds;
pub mod habboway;
pub mod handshake;
pub mod hotelview;
pub mod inventory;
pub mod modtool;
pub mod mysterybox;
pub mod navigator;
pub mod polls;
pub mod quests;
pub mod rooms;
pub mod trading;
pub mod unknown;
pub mod users;
pub mod wired;

// Re-export the outgoing.rs file
mod message_composer;

// Export any important items from this module
// pub use self::message_composer::*;