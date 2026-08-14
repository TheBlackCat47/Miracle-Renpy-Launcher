mod cloud;
mod commands;
mod process;
mod saves;
mod storage;
mod sync;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_system_status,
            commands::inspect_game_directory,
            storage::list_games,
            storage::register_game,
            process::launch_game,
            process::get_running_games,
            saves::scan_game_saves,
            saves::backup_game_saves,
            saves::list_backups,
            saves::restore_backup,
            cloud::get_cloud_status,
            cloud::save_google_client_id,
            cloud::start_google_auth,
            cloud::disconnect_google,
            cloud::verify_google_drive,
            sync::sync_game_to_drive,
            sync::sync_game_from_drive,
            sync::resolve_sync_conflict
        ])
        .run(tauri::generate_context!())
        .expect("error while running Miracle Ren'Py Launcher");
}
