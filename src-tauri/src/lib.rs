mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_system_status,
            commands::inspect_game_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running Miracle Ren'Py Launcher");
}
