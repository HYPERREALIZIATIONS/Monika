//! Monika core — the security and identity foundation for a decentralized,
//! serverless, end-to-end encrypted team chat.
//!
//! This crate currently provides the building blocks that do not require a live
//! network: key generation, the human-readable "friend code" identity handle,
//! BIP39 mnemonic backup, a password-encrypted local keystore, and invite-code
//! creation/verification. The networking (Tor + DHT) and MLS group layers are
//! intentionally left for subsequent tasks and will build on these primitives.

pub mod crypto;
pub mod error;
pub mod identity;

pub use crypto::friend_code;
pub use crypto::invite::Invite;
pub use crypto::keys::{AccountKey, DeviceKey};
pub use crypto::keystore::Keystore;
pub use error::Error;
pub use identity::Profile;
