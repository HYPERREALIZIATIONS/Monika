use crate::error::Error;
use data_encoding::BASE32_NOPAD;
use sha2::{Digest, Sha256};

const PREFIX: &str = "MONIKA";

/// Encode an Ed25519 public key into a human-friendly, checksummed "friend code"
/// such as `MONIKA-ABCD-EFGH-IJKL-MNOP-QRST-UVWX-YZ23`.
pub fn encode(public_key: &[u8; 32]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(public_key);
    let checksum = hasher.finalize();

    let mut payload = Vec::with_capacity(36);
    payload.extend_from_slice(public_key);
    payload.extend_from_slice(&checksum[..4]);

    let encoded = BASE32_NOPAD.encode(&payload);
    let grouped: String = encoded
        .as_bytes()
        .chunks(4)
        .map(|c| std::str::from_utf8(c).expect("ascii base32"))
        .collect::<Vec<_>>()
        .join("-");

    format!("{}-{}", PREFIX, grouped)
}

/// Decode a friend code back into the 32-byte public key, verifying the checksum.
pub fn decode(code: &str) -> Result<[u8; 32], Error> {
    let normalized = code.trim().to_uppercase();
    let without_prefix = normalized
        .strip_prefix("MONIKA-")
        .ok_or_else(|| Error::InvalidFriendCode("missing MONIKA- prefix".into()))?;
    let compact: String = without_prefix.chars().filter(|c| *c != '-').collect();

    let decoded = BASE32_NOPAD
        .decode(compact.as_bytes())
        .map_err(|_| Error::InvalidFriendCode("invalid base32".into()))?;
    if decoded.len() != 36 {
        return Err(Error::InvalidFriendCode("unexpected length".into()));
    }

    let mut pk = [0u8; 32];
    pk.copy_from_slice(&decoded[..32]);

    let mut hasher = Sha256::new();
    hasher.update(&pk);
    let checksum = hasher.finalize();
    if &decoded[32..36] != &checksum[..4] {
        return Err(Error::InvalidFriendCode("checksum mismatch".into()));
    }

    Ok(pk)
}
