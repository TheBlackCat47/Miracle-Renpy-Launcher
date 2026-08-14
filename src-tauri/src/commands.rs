use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub app_name: &'static str,
    pub version: &'static str,
    pub platform: &'static str,
    pub data_directory: String,
}

#[tauri::command]
pub fn get_system_status() -> Result<SystemStatus, String> {
    let data_directory = dirs_next::data_local_dir()
        .ok_or_else(|| "Impossible de déterminer le dossier de données local.".to_string())?
        .join("MiracleRenpyLauncher")
        .display()
        .to_string();

    Ok(SystemStatus {
        app_name: "Miracle Ren'Py Launcher",
        version: env!("CARGO_PKG_VERSION"),
        platform: std::env::consts::OS,
        data_directory,
    })
}
