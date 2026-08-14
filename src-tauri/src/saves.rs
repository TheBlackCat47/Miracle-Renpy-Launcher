use crate::storage::list_games;
use serde::Serialize;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;
use std::time::UNIX_EPOCH;

const MAX_FILES: usize = 5_000;

#[derive(Debug, Serialize)]
pub struct SaveFile {
    pub relative_path: String,
    pub size: u64,
    pub modified_at: String,
    pub hash: String,
}

#[derive(Debug, Serialize)]
pub struct BackupResult {
    pub backup_directory: String,
    pub file_count: usize,
    pub created_at: String,
}

#[tauri::command]
pub fn scan_game_saves(id: String) -> Result<Vec<SaveFile>, String> {
    let game = list_games()?
        .into_iter()
        .find(|game| game.id == id)
        .ok_or_else(|| "Jeu introuvable dans la bibliothèque.".to_string())?;
    let root = PathBuf::from(&game.path);
    let save_roots = [
        root.join("game/saves"),
        root.join("saves"),
        root.join("game/persistent"),
    ];
    let mut files = Vec::new();

    for save_root in save_roots.into_iter().filter(|path| path.is_dir()) {
        collect_files(&save_root, &root, &mut files)?;
        if files.len() >= MAX_FILES {
            break;
        }
    }

    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

#[tauri::command]
pub fn backup_game_saves(id: String) -> Result<BackupResult, String> {
    let game = list_games()?
        .into_iter()
        .find(|game| game.id == id)
        .ok_or_else(|| "Jeu introuvable dans la bibliothèque.".to_string())?;
    let files = scan_game_saves(game.id.clone())?;
    let root = fs::canonicalize(&game.path).map_err(io_error)?;
    let timestamp = unix_timestamp();
    let backup_directory = dirs_next::data_local_dir()
        .ok_or_else(|| "Impossible de déterminer le dossier de données local.".to_string())?
        .join("MiracleRenpyLauncher")
        .join("backups")
        .join(&game.id)
        .join(&timestamp);
    fs::create_dir_all(&backup_directory).map_err(io_error)?;

    for file in &files {
        let source = root.join(&file.relative_path);
        let canonical_source = fs::canonicalize(&source).map_err(io_error)?;
        if !canonical_source.starts_with(&root) || !canonical_source.is_file() {
            return Err("Une sauvegarde hors du dossier du jeu a été refusée.".to_string());
        }
        let target = backup_directory.join(&file.relative_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        let temporary = temporary_path(&target);
        fs::copy(&canonical_source, &temporary).map_err(io_error)?;
        fs::rename(&temporary, &target).map_err(io_error)?;
    }

    Ok(BackupResult {
        backup_directory: backup_directory.display().to_string(),
        file_count: files.len(),
        created_at: timestamp,
    })
}

fn collect_files(
    directory: &Path,
    game_root: &Path,
    files: &mut Vec<SaveFile>,
) -> Result<(), String> {
    if files.len() >= MAX_FILES {
        return Ok(());
    }

    let entries = fs::read_dir(directory).map_err(io_error)?;
    for entry in entries {
        if files.len() >= MAX_FILES {
            break;
        }
        let entry = entry.map_err(io_error)?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, game_root, files)?;
        } else if path.is_file() {
            let metadata = fs::metadata(&path).map_err(io_error)?;
            files.push(SaveFile {
                relative_path: path
                    .strip_prefix(game_root)
                    .unwrap_or(&path)
                    .display()
                    .to_string(),
                size: metadata.len(),
                modified_at: modified_timestamp(&metadata),
                hash: hash_file(&path)?,
            });
        }
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(io_error)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(io_error)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn modified_timestamp(metadata: &fs::Metadata) -> String {
    metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|| "0".to_string())
}

fn unix_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

fn temporary_path(target: &Path) -> PathBuf {
    let mut temporary = target.as_os_str().to_os_string();
    temporary.push(format!(".tmp-{}", process::id()));
    PathBuf::from(temporary)
}

fn io_error(error: impl std::fmt::Display) -> String {
    format!("Erreur de lecture des sauvegardes : {error}")
}
