use crate::cloud::valid_access_token;
use crate::saves::scan_game_saves;
use crate::storage::{get_setting, list_games, set_setting};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process;

const DRIVE_FILES_URL: &str = "https://www.googleapis.com/drive/v3/files";
const DRIVE_UPLOAD_URL: &str = "https://www.googleapis.com/upload/drive/v3/files";
const ROOT_FOLDER: &str = "Miracle RenPy Launcher";

#[derive(Debug, Deserialize)]
struct DriveFile {
    id: String,
}

#[derive(Debug, Deserialize)]
struct DriveFileList {
    files: Vec<DriveFile>,
}

#[derive(Debug, Serialize)]
struct DriveFileMetadata<'a> {
    name: &'a str,
    #[serde(rename = "mimeType")]
    mime_type: &'a str,
    parents: Vec<&'a str>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SyncManifest {
    version: u32,
    game_id: String,
    generated_at: String,
    files: Vec<ManifestFile>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ManifestFile {
    relative_path: String,
    size: u64,
    modified_at: String,
    hash: String,
    remote_file_id: String,
}

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub uploaded_files: usize,
    pub folder_name: String,
    pub manifest_file_id: String,
}

#[derive(Debug, Serialize)]
pub struct PullResult {
    pub downloaded_files: usize,
    pub unchanged_files: usize,
    pub backup_directory: Option<String>,
    pub conflicts: Vec<String>,
}

#[tauri::command]
pub fn sync_game_to_drive(id: String) -> Result<SyncResult, String> {
    let game = list_games()?
        .into_iter()
        .find(|game| game.id == id)
        .ok_or_else(|| "Jeu introuvable dans la bibliothèque.".to_string())?;
    let root = fs::canonicalize(&game.path)
        .map_err(|error| format!("Dossier du jeu inaccessible : {error}"))?;
    let token = valid_access_token()?;
    let client = Client::new();
    let app_folder = ensure_folder(&client, &token, ROOT_FOLDER, None)?;
    let game_folder = ensure_folder(&client, &token, &game.name, Some(&app_folder))?;
    let saves = scan_game_saves(game.id.clone())?;
    let mut manifest_files = Vec::with_capacity(saves.len());

    for save in &saves {
        let source = safe_local_save_path(&root, &save.relative_path)?;
        let remote_id = upload_file(
            &client,
            &token,
            &game_folder,
            &save.relative_path,
            "application/octet-stream",
            &source,
        )?;
        manifest_files.push(ManifestFile {
            relative_path: save.relative_path.clone(),
            size: save.size,
            modified_at: save.modified_at.clone(),
            hash: save.hash.clone(),
            remote_file_id: remote_id,
        });
    }

    let generated_at = unix_timestamp();
    let game_id = game.id.clone();
    let manifest = SyncManifest {
        version: 1,
        game_id,
        generated_at,
        files: manifest_files,
    };
    let manifest_path = std::env::temp_dir().join(format!("mrl-manifest-{}.json", game.id));
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).map_err(json_error)?,
    )
    .map_err(|error| format!("Manifeste local impossible à créer : {error}"))?;
    let manifest_id = upload_file(
        &client,
        &token,
        &game_folder,
        "manifest.json",
        "application/json",
        &manifest_path,
    );
    let _ = fs::remove_file(&manifest_path);
    let manifest_id = manifest_id?;
    remember_manifest(&game.id, &manifest)?;

    Ok(SyncResult {
        uploaded_files: saves.len(),
        folder_name: game.name,
        manifest_file_id: manifest_id,
    })
}

#[tauri::command]
pub fn sync_game_from_drive(id: String) -> Result<PullResult, String> {
    let game = list_games()?
        .into_iter()
        .find(|game| game.id == id)
        .ok_or_else(|| "Jeu introuvable dans la bibliothèque.".to_string())?;
    let root = fs::canonicalize(&game.path)
        .map_err(|error| format!("Dossier du jeu inaccessible : {error}"))?;
    let token = valid_access_token()?;
    let client = Client::new();
    let app_folder = find_folder(&client, &token, ROOT_FOLDER, None)?
        .ok_or_else(|| "Aucun dossier MRL n’existe encore dans Google Drive.".to_string())?;
    let game_folder = find_folder(&client, &token, &game.name, Some(&app_folder))?
        .ok_or_else(|| "Ce jeu n’a pas encore été synchronisé vers Google Drive.".to_string())?;
    let manifest_id = find_file(
        &client,
        &token,
        &format!(
            "name = 'manifest.json' and '{}' in parents and trashed = false",
            escape_query_value(&game_folder)
        ),
    )?
    .ok_or_else(|| "Le manifeste Drive de ce jeu est introuvable.".to_string())?;
    let manifest =
        serde_json::from_slice::<SyncManifest>(&download_file(&client, &token, &manifest_id)?)
            .map_err(|error| format!("Manifeste Drive invalide : {error}"))?;
    if manifest.game_id != game.id {
        return Err("Le manifeste Drive ne correspond pas à ce jeu.".to_string());
    }

    let local_files: HashMap<String, String> = scan_game_saves(game.id.clone())?
        .into_iter()
        .map(|file| (normalize_relative(&file.relative_path), file.hash))
        .collect();
    let previous = remembered_manifest(&game.id)?;
    let previous_hashes = previous
        .as_ref()
        .map(|manifest| {
            manifest
                .files
                .iter()
                .map(|file| (normalize_relative(&file.relative_path), file.hash.clone()))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    let mut changed = Vec::new();
    let mut conflicts = Vec::new();
    for file in &manifest.files {
        let relative = safe_relative_path(&file.relative_path)?;
        let local_hash = local_files.get(&relative);
        if local_hash != Some(&file.hash) {
            let local_changed = previous_hashes
                .get(&relative)
                .map(|previous_hash| local_hash != Some(previous_hash))
                .unwrap_or(false);
            let remote_changed = previous_hashes
                .get(&relative)
                .map(|previous_hash| previous_hash != &file.hash)
                .unwrap_or(false);
            if local_changed && remote_changed {
                conflicts.push(relative);
                continue;
            }
            changed.push((file, relative));
        }
    }
    if changed.is_empty() {
        if conflicts.is_empty() {
            remember_manifest(&game.id, &manifest)?;
        }
        return Ok(PullResult {
            downloaded_files: 0,
            unchanged_files: manifest.files.len() - conflicts.len(),
            backup_directory: None,
            conflicts,
        });
    }

    let backup = crate::saves::backup_game_saves(game.id.clone())?;
    for (file, relative) in &changed {
        let target = root.join(relative);
        let bytes = download_file(&client, &token, &file.remote_file_id)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        let temporary = temporary_path(&target);
        fs::write(&temporary, bytes).map_err(io_error)?;
        if target.exists() {
            let previous = previous_path(&target);
            fs::rename(&target, previous).map_err(io_error)?;
        }
        fs::rename(&temporary, &target).map_err(io_error)?;
    }

    if conflicts.is_empty() {
        remember_manifest(&game.id, &manifest)?;
    }

    Ok(PullResult {
        downloaded_files: changed.len(),
        unchanged_files: manifest.files.len() - changed.len() - conflicts.len(),
        backup_directory: Some(backup.backup_directory),
        conflicts,
    })
}

fn ensure_folder(
    client: &Client,
    token: &str,
    name: &str,
    parent: Option<&str>,
) -> Result<String, String> {
    let escaped_name = escape_query_value(name);
    let mut query = format!(
        "name = '{}' and mimeType = 'application/vnd.google-apps.folder' and trashed = false",
        escaped_name
    );
    if let Some(parent) = parent {
        query.push_str(&format!(" and '{}' in parents", escape_query_value(parent)));
    }
    if let Some(file) = find_file(client, token, &query)? {
        return Ok(file);
    }
    let metadata = DriveFileMetadata {
        name,
        mime_type: "application/vnd.google-apps.folder",
        parents: parent.into_iter().collect(),
    };
    client
        .post(DRIVE_FILES_URL)
        .query(&[("fields", "id")])
        .bearer_auth(token)
        .json(&metadata)
        .send()
        .map_err(drive_error)
        .and_then(parse_file_id)
}

fn find_folder(
    client: &Client,
    token: &str,
    name: &str,
    parent: Option<&str>,
) -> Result<Option<String>, String> {
    let mut query = format!(
        "name = '{}' and mimeType = 'application/vnd.google-apps.folder' and trashed = false",
        escape_query_value(name)
    );
    if let Some(parent) = parent {
        query.push_str(&format!(" and '{}' in parents", escape_query_value(parent)));
    }
    find_file(client, token, &query)
}

fn upload_file(
    client: &Client,
    token: &str,
    parent: &str,
    name: &str,
    mime_type: &str,
    source: &Path,
) -> Result<String, String> {
    let query = format!(
        "name = '{}' and '{}' in parents and trashed = false",
        escape_query_value(name),
        escape_query_value(parent)
    );
    let file_id = find_file(client, token, &query)?;
    let id = match file_id {
        Some(id) => id,
        None => {
            let metadata = DriveFileMetadata {
                name,
                mime_type,
                parents: vec![parent],
            };
            client
                .post(DRIVE_FILES_URL)
                .query(&[("fields", "id")])
                .bearer_auth(token)
                .json(&metadata)
                .send()
                .map_err(drive_error)
                .and_then(parse_file_id)?
        }
    };
    let bytes = fs::read(source)
        .map_err(|error| format!("Fichier de sauvegarde inaccessible : {error}"))?;
    client
        .patch(format!("{DRIVE_UPLOAD_URL}/{id}"))
        .query(&[("uploadType", "media"), ("fields", "id")])
        .bearer_auth(token)
        .header(reqwest::header::CONTENT_TYPE, mime_type)
        .body(bytes)
        .send()
        .map_err(drive_error)
        .and_then(parse_file_id)
}

fn find_file(client: &Client, token: &str, query: &str) -> Result<Option<String>, String> {
    let response = client
        .get(DRIVE_FILES_URL)
        .query(&[("q", query), ("spaces", "drive"), ("fields", "files(id)")])
        .bearer_auth(token)
        .send()
        .map_err(drive_error)?
        .error_for_status()
        .map_err(drive_error)?
        .json::<DriveFileList>()
        .map_err(|error| format!("Réponse de recherche Google Drive invalide : {error}"))?;
    Ok(response.files.into_iter().next().map(|file| file.id))
}

fn download_file(client: &Client, token: &str, id: &str) -> Result<Vec<u8>, String> {
    client
        .get(format!("{DRIVE_FILES_URL}/{id}"))
        .query(&[("alt", "media")])
        .bearer_auth(token)
        .send()
        .map_err(drive_error)
        .and_then(|response| {
            response
                .error_for_status()
                .map_err(drive_error)
                .and_then(|response| {
                    response
                        .bytes()
                        .map(|bytes| bytes.to_vec())
                        .map_err(drive_error)
                })
        })
}

fn parse_file_id(response: reqwest::blocking::Response) -> Result<String, String> {
    response
        .error_for_status()
        .map_err(drive_error)?
        .json::<DriveFile>()
        .map(|file| file.id)
        .map_err(|error| format!("Réponse Google Drive invalide : {error}"))
}

fn safe_local_save_path(root: &Path, relative: &str) -> Result<std::path::PathBuf, String> {
    let candidate = root.join(relative);
    let canonical = fs::canonicalize(&candidate)
        .map_err(|error| format!("Sauvegarde inaccessible ({relative}) : {error}"))?;
    if !canonical.starts_with(root) || !canonical.is_file() {
        return Err("Une sauvegarde hors du dossier du jeu a été refusée.".to_string());
    }
    Ok(canonical)
}

fn safe_relative_path(relative: &str) -> Result<String, String> {
    let normalized = normalize_relative(relative);
    let path = Path::new(&normalized);
    let valid_root = normalized.starts_with("game/saves/")
        || normalized.starts_with("saves/")
        || normalized.starts_with("game/persistent/");
    if !valid_root
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("Chemin de sauvegarde Drive refusé : {relative}"));
    }
    Ok(normalized)
}

fn normalize_relative(relative: &str) -> String {
    relative.replace('\\', "/")
}

fn temporary_path(target: &Path) -> PathBuf {
    let mut temporary = target.as_os_str().to_os_string();
    temporary.push(format!(".tmp-sync-{}", process::id()));
    PathBuf::from(temporary)
}

fn previous_path(target: &Path) -> PathBuf {
    let mut previous = target.as_os_str().to_os_string();
    previous.push(format!(".pre-sync-{}-{}", process::id(), unix_timestamp()));
    PathBuf::from(previous)
}

fn io_error(error: impl std::fmt::Display) -> String {
    format!("Erreur d’écriture des sauvegardes synchronisées : {error}")
}

fn manifest_setting_key(game_id: &str) -> String {
    format!("sync.last_manifest.{game_id}")
}

fn remember_manifest(game_id: &str, manifest: &SyncManifest) -> Result<(), String> {
    let value = serde_json::to_string(manifest).map_err(json_error)?;
    set_setting(&manifest_setting_key(game_id), &value)
}

fn remembered_manifest(game_id: &str) -> Result<Option<SyncManifest>, String> {
    get_setting(&manifest_setting_key(game_id))?
        .map(|value| {
            serde_json::from_str(&value)
                .map_err(|error| format!("Manifeste local mémorisé invalide : {error}"))
        })
        .transpose()
}

fn escape_query_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn drive_error(error: impl std::fmt::Display) -> String {
    format!("Google Drive a refusé la synchronisation : {error}")
}

fn json_error(error: serde_json::Error) -> String {
    format!("Manifeste JSON invalide : {error}")
}

fn unix_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
