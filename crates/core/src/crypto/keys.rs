use crate::crypto::friend_code;
use crate::error::Error;
use bip39::Mnemonic;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use zeroize::Zeroize;

pub const ACCOUNT_SEED_LEN: usize = 32;
pub const DEVICE_KEY_LEN: usize = 32;

/// Long-term account identity. The 32-byte seed is the BIP39-derived secret; it
/// is what the user backs up via the generated mnemonic and what the keystore
/// persists (encrypted). Losing it means losing the identity — by design there
/// is no server-side recovery.
#[derive(Clone)]
pub struct AccountKey {
    seed: [u8; ACCOUNT_SEED_LEN],
    signing: SigningKey,
}

impl AccountKey {
    /// Generate a new account identity together with its BIP39 recovery mnemonic
    /// (24 words / 256-bit entropy -> 32-byte seed).
    pub fn generate() -> (AccountKey, Mnemonic) {
        let mut entropy = [0u8; ACCOUNT_SEED_LEN];
        OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy).expect("valid entropy");
        let key = AccountKey::from_entropy(&entropy);
        entropy.zeroize();
        (key, mnemonic)
    }

    pub fn from_mnemonic(mnemonic: &Mnemonic) -> Result<AccountKey, Error> {
        let entropy = mnemonic.to_entropy();
        if entropy.len() != ACCOUNT_SEED_LEN {
            return Err(Error::Mnemonic(format!(
                "expected {} bytes of entropy, got {}",
                ACCOUNT_SEED_LEN,
                entropy.len()
            )));
        }
        let mut seed = [0u8; ACCOUNT_SEED_LEN];
        seed.copy_from_slice(&entropy);
        let key = AccountKey::from_entropy(&seed);
        seed.zeroize();
        Ok(key)
    }

    pub(crate) fn from_entropy(seed: &[u8; ACCOUNT_SEED_LEN]) -> AccountKey {
        AccountKey {
            seed: *seed,
            signing: SigningKey::from_bytes(seed),
        }
    }

    pub fn to_seed(&self) -> [u8; ACCOUNT_SEED_LEN] {
        self.seed
    }

    pub fn public_key(&self) -> [u8; 32] {
        self.signing.verifying_key().to_bytes()
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing.verifying_key()
    }

    /// Human-readable, encoded public-key handle shown to other users.
    pub fn friend_code(&self) -> String {
        friend_code::encode(&self.public_key())
    }

    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let sig: Signature = self.signing.sign(message);
        sig.to_bytes()
    }

    pub fn verify(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> Result<(), Error> {
        let vk = VerifyingKey::from_bytes(public_key).map_err(|_| Error::Signature)?;
        let sig = Signature::from_bytes(signature);
        vk.verify(message, &sig).map_err(|_| Error::Signature)
    }
}

/// Per-device X25519 keypair. Used for MLS group operations and peer routing.
/// A fresh device key is minted each time the app is installed; the account key
/// is the stable identity.
#[derive(Clone)]
pub struct DeviceKey {
    secret: StaticSecret,
    public: X25519PublicKey,
}

impl DeviceKey {
    pub fn generate() -> DeviceKey {
        let mut csprng = OsRng;
        let secret = StaticSecret::random_from_rng(&mut csprng);
        let public = X25519PublicKey::from(&secret);
        DeviceKey { secret, public }
    }

    pub fn from_private(private: &[u8; 32]) -> DeviceKey {
        let secret = StaticSecret::from(*private);
        let public = X25519PublicKey::from(&secret);
        DeviceKey { secret, public }
    }

    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.secret.to_bytes()
    }

    pub fn public_key(&self) -> [u8; 32] {
        self.public.to_bytes()
    }
}
