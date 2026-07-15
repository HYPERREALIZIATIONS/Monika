use tauri::Manager;

/// In-memory handle to the unlocked local keystore. In a full build this is
/// persisted on disk (encrypted, see `monika_core::Keystore`) and reloaded on
/// launch after the user enters their password.
struct AppState {
    keystore: std::sync::Mutex<Option<monika_core::Keystore>>,
}

#[tauri::command]
fn generate_friend_code() -> String {
    let (account, _mnemonic) = monika_core::AccountKey::generate();
    account.friend_code()
}

#[tauri::command]
fn make_invite(community_id: String, bootstrap_peers: Vec<String>) -> String {
    let (inviter, _mnemonic) = monika_core::AccountKey::generate();
    let invite = monika_core::Invite::new(&community_id, bootstrap_peers, Vec::new(), &inviter);
    invite.encode()
}

#[tauri::command]
fn verify_invite(code: String) -> Result<bool, String> {
    let invite = monika_core::Invite::decode(&code).map_err(|e| e.to_string())?;
    Ok(invite.verify().is_ok())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::Builder::new().build())
        .setup(|app| {
            app.manage(AppState {
                keystore: std::sync::Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            generate_friend_code,
            make_invite,
            verify_invite
        ])
        .run(tauri::generate_context!())
        .expect("error while running Monika");
}
