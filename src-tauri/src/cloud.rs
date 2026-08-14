use crate::storage::{get_setting, set_setting};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use keyring::Entry;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use url::Url;
use uuid::Uuid;

const KEYRING_SERVICE: &str = "MiracleRenpyLauncher";
const REFRESH_TOKEN_KEY: &str = "google.refresh_token";
const ACCESS_TOKEN_KEY: &str = "google.access_token";

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

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    email: Option<String>,
}

#[tauri::command]
pub fn get_cloud_status() -> Result<CloudStatus, String> {
    let provider = GoogleDriveProvider;
    Ok(CloudStatus {
        provider: provider.name().to_string(),
        configured: get_setting("google.client_id")?.is_some(),
        connected: matches!(
            keyring_entry(REFRESH_TOKEN_KEY),
            Ok(entry) if entry.get_password().is_ok()
        ),
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

#[tauri::command]
pub fn start_google_auth() -> Result<CloudStatus, String> {
    let client_id = get_setting("google.client_id")?
        .ok_or_else(|| "Configurez d’abord l’identifiant client Google.".to_string())?;
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|error| format!("Impossible d’ouvrir le callback OAuth local : {error}"))?;
    let port = listener
        .local_addr()
        .map_err(|error| format!("Impossible de lire le port OAuth local : {error}"))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}/oauth2/callback");
    let state = Uuid::new_v4().simple().to_string();
    let verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));

    let mut authorization_url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .map_err(|error| error.to_string())?;
    authorization_url
        .query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", "https://www.googleapis.com/auth/drive.file")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("state", &state)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256");
    open::that(authorization_url.as_str())
        .map_err(|error| format!("Impossible d’ouvrir le navigateur : {error}"))?;

    listener
        .set_nonblocking(false)
        .map_err(|error| error.to_string())?;
    let (mut stream, _) = listener
        .accept()
        .map_err(|error| format!("Le callback OAuth n’a pas été reçu : {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .map_err(|error| error.to_string())?;
    let request = read_callback_request(&mut stream)?;
    let callback = Url::parse(&format!("http://localhost{request}"))
        .map_err(|error| format!("Callback OAuth invalide : {error}"))?;
    let query: std::collections::HashMap<_, _> = callback.query_pairs().into_owned().collect();
    write_callback_response(&mut stream)?;

    if query.get("state") != Some(&state) {
        return Err("La vérification de sécurité OAuth a échoué.".to_string());
    }
    if let Some(error) = query.get("error") {
        return Err(format!("Google a refusé l’autorisation : {error}"));
    }
    let code = query
        .get("code")
        .ok_or_else(|| "Google n’a pas retourné de code d’autorisation.".to_string())?;

    let token = Client::new()
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code.as_str()),
            ("client_id", client_id.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
            ("code_verifier", verifier.as_str()),
        ])
        .send()
        .map_err(|error| format!("Échange OAuth impossible : {error}"))?
        .error_for_status()
        .map_err(|error| format!("Google a refusé le token OAuth : {error}"))?
        .json::<TokenResponse>()
        .map_err(|error| format!("Réponse OAuth invalide : {error}"))?;

    let refresh_token = token
        .refresh_token
        .ok_or_else(|| "Google n’a pas fourni de refresh token.".to_string())?;
    keyring_entry(REFRESH_TOKEN_KEY)?
        .set_password(&refresh_token)
        .map_err(|error| format!("Impossible de sécuriser le refresh token : {error}"))?;
    keyring_entry(ACCESS_TOKEN_KEY)?
        .set_password(&token.access_token)
        .map_err(|error| format!("Impossible de sécuriser le token d’accès : {error}"))?;

    if let Ok(user_info) = Client::new()
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(&token.access_token)
        .send()
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.json::<UserInfo>())
    {
        if let Some(email) = user_info.email {
            set_setting("google.account_email", &email)?;
        }
    }
    get_cloud_status()
}

#[tauri::command]
pub fn disconnect_google() -> Result<CloudStatus, String> {
    for key in [REFRESH_TOKEN_KEY, ACCESS_TOKEN_KEY] {
        if let Ok(entry) = keyring_entry(key) {
            let _ = entry.delete_credential();
        }
    }
    set_setting("google.account_email", "")?;
    get_cloud_status()
}

fn keyring_entry(key: &str) -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, key)
        .map_err(|error| format!("Gestionnaire d’identifiants indisponible : {error}"))
}

fn read_callback_request(stream: &mut TcpStream) -> Result<String, String> {
    let mut buffer = [0_u8; 8192];
    let size = stream
        .read(&mut buffer)
        .map_err(|error| format!("Lecture du callback OAuth impossible : {error}"))?;
    let request = String::from_utf8_lossy(&buffer[..size]);
    request
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("GET "))
        .and_then(|line| line.split_whitespace().next())
        .map(str::to_string)
        .ok_or_else(|| "Requête de callback OAuth invalide.".to_string())
}

fn write_callback_response(stream: &mut TcpStream) -> Result<(), String> {
    let body = "<html><body><h2>MRL est connecté.</h2><p>Vous pouvez fermer cette fenêtre.</p></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(), body
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| format!("Réponse OAuth impossible : {error}"))
}
