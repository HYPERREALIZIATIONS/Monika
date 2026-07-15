use monika_core::crypto::friend_code;
use monika_core::crypto::invite::Invite;
use monika_core::crypto::keys::AccountKey;
use monika_core::crypto::keystore::Keystore;
use monika_core::identity::Profile;

#[test]
fn account_key_and_friend_code_roundtrip() {
    let (account, _mnemonic) = AccountKey::generate();
    let code = account.friend_code();
    let pk = friend_code::decode(&code).expect("friend code should decode");
    assert_eq!(pk, account.public_key());

    // case-insensitive, dash-tolerant decoding
    let lower = code.to_lowercase();
    assert_eq!(friend_code::decode(&lower).unwrap(), account.public_key());
}

#[test]
fn mnemonic_backup_and_restore_is_deterministic() {
    let (account, mnemonic) = AccountKey::generate();
    let pk = account.public_key();

    let restored = AccountKey::from_mnemonic(&mnemonic).expect("restore from mnemonic");
    assert_eq!(restored.public_key(), pk);

    // phrase <-> mnemonic round trip
    let phrase = mnemonic.to_string();
    let m2: bip39::Mnemonic = phrase.parse().unwrap();
    let restored2 = AccountKey::from_mnemonic(&m2).unwrap();
    assert_eq!(restored2.public_key(), pk);
}

#[test]
fn keystore_encrypt_decrypt_and_wrong_password() {
    let ks = Keystore::generate();
    let pw = "correct horse battery staple";
    let blob = ks.to_encrypted(pw).unwrap();

    let restored = Keystore::from_encrypted(&blob, pw).unwrap();
    assert_eq!(restored.account.public_key(), ks.account.public_key());
    assert_eq!(restored.device.public_key(), ks.device.public_key());

    // different password must fail auth, not panic
    assert!(Keystore::from_encrypted(&blob, "wrong-password").is_err());
    // tampered blob must fail auth
    let mut tampered = blob.clone();
    if let Some(b) = tampered.last_mut() {
        *b ^= 0xff;
    }
    assert!(Keystore::from_encrypted(&tampered, pw).is_err());
}

#[test]
fn keystore_restore_from_mnemonic_preserves_account() {
    let (account, mnemonic) = AccountKey::generate();
    let pk = account.public_key();

    let ks = Keystore::from_mnemonic(&mnemonic).unwrap();
    assert_eq!(ks.account.public_key(), pk);
}

#[test]
fn invite_sign_verify_and_encode_roundtrip() {
    let (inviter, _m) = AccountKey::generate();
    let invite = Invite::new(
        "team-abc",
        vec!["peer1.onion".into(), "peer2.onion".into()],
        vec![1, 2, 3, 4],
        &inviter,
    );
    invite.verify().expect("freshly built invite verifies");

    let code = invite.encode();
    let decoded = Invite::decode(&code).expect("invite decodes");
    decoded.verify().expect("decoded invite verifies");
    assert_eq!(decoded.community_id, "team-abc");
    assert_eq!(decoded.bootstrap_peers.len(), 2);
}

#[test]
fn profile_derives_friend_code() {
    let (account, _m) = AccountKey::generate();
    let profile = Profile::from_account("alice", &account);
    assert_eq!(profile.username, "alice");
    assert!(profile.friend_code.starts_with("MONIKA-"));
    assert_eq!(profile.public_key, account.public_key());
}
