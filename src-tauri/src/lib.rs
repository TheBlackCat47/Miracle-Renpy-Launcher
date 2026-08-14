mod commands;
mod process;
mod saves;
mod storage;

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
            saves::backup_game_saves
        ])
        .run(tauri::generate_context!())
        .expect("error while running Miracle Ren'Py Launcher");
}
