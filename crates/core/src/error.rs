use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("cryptographic operation failed: {0}")]
    Crypto(String),

    #[error("invalid friend code: {0}")]
    InvalidFriendCode(String),

    #[error("keystore decryption failed (wrong password or corrupt data)")]
    KeystoreAuth,

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("invalid invite code: {0}")]
    InvalidInvite(String),

    #[error("mnemonic error: {0}")]
    Mnemonic(String),

    #[error("signature verification failed")]
    Signature,
}
