use crate::crypto::keys::AccountKey;

/// A user-facing identity: a chosen display name plus the derived, shareable
/// "friend code" handle. The friend code is what others add to connect.
#[derive(Clone, Debug)]
pub struct Profile {
    pub username: String,
    pub friend_code: String,
    pub public_key: [u8; 32],
}

impl Profile {
    pub fn from_account(username: &str, account: &AccountKey) -> Profile {
        Profile {
            username: username.to_string(),
            friend_code: account.friend_code(),
            public_key: account.public_key(),
        }
    }
}
