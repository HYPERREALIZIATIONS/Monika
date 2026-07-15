use crate::crypto::keys::{AccountKey, DeviceKey, ACCOUNT_SEED_LEN, DEVICE_KEY_LEN};
use crate::error::Error;
use argon2::Argon2;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

const MAGIC: &[u8; 4] = b"MNK1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

#[derive(Serialize, Deserialize, Zeroize)]
struct Stored {
    account_seed: [u8; ACCOUNT_SEED_LEN],
    device_private: [u8; DEVICE_KEY_LEN],
}

/// A device-local store of the user's account and device keys, encrypted at rest
/// with a user-chosen password (Argon2id KDF + ChaCha20-Poly1305 AEAD).
pub struct Keystore {
    pub account: AccountKey,
    pub device: DeviceKey,
}

impl Keystore {
    /// Generate a brand-new account + device identity.
    pub fn generate() -> Keystore {
        let (account, _mnemonic) = AccountKey::generate();
        let device = DeviceKey::generate();
        Keystore { account, device }
    }

    /// Rebuild an account identity from its recovery mnemonic, minting a fresh
    /// device key (devices are not backed up).
    pub fn from_mnemonic(mnemonic: &bip39::Mnemonic) -> Result<Keystore, Error> {
        let account = AccountKey::from_mnemonic(mnemonic)?;
        let device = DeviceKey::generate();
        Ok(Keystore { account, device })
    }

    /// Serialize and encrypt the keystore under `password`.
    pub fn to_encrypted(&self, password: &str) -> Result<Vec<u8>, Error> {
        let mut stored = Stored {
            account_seed: self.account.to_seed(),
            device_private: self.device.private_key_bytes(),
        };
        let plaintext = serde_cbor::to_vec(&stored).map_err(|e| Error::Serialization(e.to_string()))?;

        let mut salt = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt);
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), &salt, &mut key)
            .map_err(|e| Error::Crypto(e.to_string()))?;

        let cipher = ChaCha20Poly1305::new_from_slice(&key).map_err(|e| Error::Crypto(e.to_string()))?;
        let mut nonce = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_slice())
            .map_err(|_| Error::KeystoreAuth)?;

        let mut out = Vec::with_capacity(MAGIC.len() + SALT_LEN + NONCE_LEN + ciphertext.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&salt);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);

        stored.zeroize();
        key.zeroize();
        Ok(out)
    }

    /// Decrypt a keystore blob. Returns [`Error::KeystoreAuth`] on wrong password
    /// or tampered data.
    pub fn from_encrypted(blob: &[u8], password: &str) -> Result<Keystore, Error> {
        if blob.len() < MAGIC.len() + SALT_LEN + NONCE_LEN {
            return Err(Error::KeystoreAuth);
        }
        if &blob[..MAGIC.len()] != MAGIC {
            return Err(Error::KeystoreAuth);
        }
        let salt = &blob[MAGIC.len()..MAGIC.len() + SALT_LEN];
        let nonce = &blob[MAGIC.len() + SALT_LEN..MAGIC.len() + SALT_LEN + NONCE_LEN];
        let ciphertext = &blob[MAGIC.len() + SALT_LEN + NONCE_LEN..];

        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| Error::Crypto(e.to_string()))?;

        let cipher = ChaCha20Poly1305::new_from_slice(&key).map_err(|e| Error::Crypto(e.to_string()))?;
        let plaintext = cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|_| Error::KeystoreAuth)?;

        let stored: Stored =
            serde_cbor::from_slice(&plaintext).map_err(|_| Error::KeystoreAuth)?;

        let account = AccountKey::from_entropy(&stored.account_seed);
        let device = DeviceKey::from_private(&stored.device_private);

        key.zeroize();
        Ok(Keystore { account, device })
    }
}
