//! Module for handling incoming messages from clients

// Re-export all submodules
pub mod achievements;
pub mod ambassadors;
pub mod camera;
pub mod catalog;
pub mod crafting;
pub mod events;
pub mod floorplaneditor;
pub mod friends;
pub mod gamecenter;
pub mod guardians;
pub mod guides;
pub mod guilds;
pub mod handshake;
pub mod helper;
pub mod hotelview;
pub mod inventory;
pub mod modtool;
pub mod navigator;
pub mod polls;
pub mod rooms;
pub mod trading;
pub mod unknown;
pub mod users;
pub mod wired;

// Re-export the incoming.rs file
mod message_handler;

// Export any important items from this module
// pub use self::message_handler::*;