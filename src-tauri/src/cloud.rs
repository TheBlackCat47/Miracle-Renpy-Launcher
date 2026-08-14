use crate::storage::{get_setting, set_setting};
use serde::Serialize;

pub trait CloudProvider {
    fn name(&self) -> &'static str;
}

pub struct GoogleDriveProvider;

impl CloudProvider for GoogleDriveProvider {
    fn name(&self) -> &'static str {
        "Google Drive"
    }
}

#[derive(Debug, Serialize)]
pub struct CloudStatus {
    pub provider: String,
    pub configured: bool,
    pub connected: bool,
    pub account_email: Option<String>,
}

#[tauri::command]
pub fn get_cloud_status() -> Result<CloudStatus, String> {
    let provider = GoogleDriveProvider;
    Ok(CloudStatus {
        provider: provider.name().to_string(),
        configured: get_setting("google.client_id")?.is_some(),
        connected: get_setting("google.refresh_token")?.is_some(),
        account_email: get_setting("google.account_email")?,
    })
}

#[tauri::command]
pub fn save_google_client_id(client_id: String) -> Result<CloudStatus, String> {
    let client_id = client_id.trim();
    if client_id.is_empty()
        || client_id.len() > 256
        || !client_id.contains(".apps.googleusercontent.com")
    {
        return Err("L’identifiant client Google ne semble pas valide.".to_string());
    }
    set_setting("google.client_id", client_id)?;
    get_cloud_status()
}
