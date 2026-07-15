use crate::crypto::keys::AccountKey;
use crate::error::Error;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};

const PREFIX: &str = "MONIKA-INVITE-";

/// A shareable community invitation. Encodes everything a new device needs to
/// discover the DHT, present a join authorization, and receive the MLS welcome.
///
/// `welcome` is an opaque MLS Welcome blob today (filled in once the MLS layer
/// lands); it is integrity-protected by `join_token`, which the inviter signs
/// over `(community_id || welcome)` with their account key.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Invite {
    pub community_id: String,
    pub bootstrap_peers: Vec<String>,
    pub welcome: Vec<u8>,
    pub join_token: Vec<u8>,
    pub inviter_public_key: [u8; 32],
}

impl Invite {
    pub fn new(
        community_id: &str,
        bootstrap_peers: Vec<String>,
        welcome: Vec<u8>,
        inviter: &AccountKey,
    ) -> Invite {
        let mut payload = Vec::new();
        payload.extend_from_slice(community_id.as_bytes());
        payload.extend_from_slice(&welcome);
        let join_token = inviter.sign(&payload).to_vec();
        Invite {
            community_id: community_id.to_string(),
            bootstrap_peers,
            welcome,
            join_token,
            inviter_public_key: inviter.public_key(),
        }
    }

    /// Verify the inviter's signature over the community id + welcome blob.
    pub fn verify(&self) -> Result<(), Error> {
        let mut payload = Vec::new();
        payload.extend_from_slice(self.community_id.as_bytes());
        payload.extend_from_slice(&self.welcome);
        let token: [u8; 64] = self
            .join_token
            .as_slice()
            .try_into()
            .map_err(|_| Error::InvalidInvite("join token has wrong length".into()))?;
        AccountKey::verify(&self.inviter_public_key, &payload, &token)
    }

    pub fn encode(&self) -> String {
        let cbor = serde_cbor::to_vec(self).expect("invite is serializable");
        format!("{}{}", PREFIX, URL_SAFE_NO_PAD.encode(cbor))
    }

    pub fn decode(code: &str) -> Result<Invite, Error> {
        let b64 = code
            .trim()
            .strip_prefix(PREFIX)
            .ok_or_else(|| Error::InvalidInvite("missing MONIKA-INVITE- prefix".into()))?;
        let cbor = URL_SAFE_NO_PAD
            .decode(b64)
            .map_err(|_| Error::InvalidInvite("invalid base64".into()))?;
        let invite: Invite =
            serde_cbor::from_slice(&cbor).map_err(|e| Error::InvalidInvite(e.to_string()))?;
        Ok(invite)
    }
}
