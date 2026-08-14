use crate::storage::{list_games, GameRecord};
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

#[derive(Debug, Serialize)]
pub struct BackupRecord {
    pub directory: String,
    pub created_at: String,
    pub file_count: usize,
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
        collect_files(&save_root, &root, "", &mut files)?;
        if files.len() >= MAX_FILES {
            break;
        }
    }
    if files.len() < MAX_FILES {
        if let Some(external) = game.save_directory.as_deref().map(PathBuf::from) {
            if external.is_dir() {
                collect_files(&external, &external, "external", &mut files)?;
            }
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
        let canonical_source = resolve_save_path(&game, &root, Path::new(&file.relative_path))?;
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

#[tauri::command]
pub fn list_backups(id: String) -> Result<Vec<BackupRecord>, String> {
    let backup_root = backup_root(&id)?;
    if !backup_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut backups = Vec::new();
    for entry in fs::read_dir(&backup_root).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let directory = entry.path();
        if directory.is_dir() {
            let file_count = collect_backup_files(&directory)?.len();
            backups.push(BackupRecord {
                directory: directory.display().to_string(),
                created_at: entry.file_name().to_string_lossy().to_string(),
                file_count,
            });
        }
    }
    backups.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(backups)
}

#[tauri::command]
pub fn restore_backup(id: String, directory: String) -> Result<BackupResult, String> {
    let game = list_games()?
        .into_iter()
        .find(|game| game.id == id)
        .ok_or_else(|| "Jeu introuvable dans la bibliothèque.".to_string())?;
    let backup_root = fs::canonicalize(backup_root(&game.id)?).map_err(io_error)?;
    let selected = fs::canonicalize(&directory).map_err(io_error)?;
    if selected == backup_root || !selected.starts_with(&backup_root) || !selected.is_dir() {
        return Err("Le dossier de restauration n’est pas un backup MRL valide.".to_string());
    }

    let current_backup = backup_game_saves(game.id.clone())?;
    let game_root = fs::canonicalize(&game.path).map_err(io_error)?;
    let files = collect_backup_files(&selected)?;
    for source in &files {
        let relative = source.strip_prefix(&selected).map_err(io_error)?;
        if !is_save_relative_path(relative) {
            return Err(
                "Le backup contient un chemin qui n’est pas une sauvegarde autorisée.".to_string(),
            );
        }
        let target = save_target_path(&game, &game_root, relative)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        let temporary = temporary_path(&target);
        fs::copy(source, &temporary).map_err(io_error)?;
        if target.exists() {
            let mut previous = target.as_os_str().to_os_string();
            previous.push(format!(".pre-restore-{}", process::id()));
            fs::rename(&target, PathBuf::from(previous)).map_err(io_error)?;
        }
        fs::rename(&temporary, &target).map_err(io_error)?;
    }

    Ok(BackupResult {
        backup_directory: current_backup.backup_directory,
        file_count: files.len(),
        created_at: current_backup.created_at,
    })
}

fn collect_files(
    directory: &Path,
    relative_root: &Path,
    prefix: &str,
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
            collect_files(&path, relative_root, prefix, files)?;
        } else if path.is_file() {
            let metadata = fs::metadata(&path).map_err(io_error)?;
            files.push(SaveFile {
                relative_path: prefixed_relative_path(&path, relative_root, prefix),
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

fn backup_root(id: &str) -> Result<PathBuf, String> {
    Ok(dirs_next::data_local_dir()
        .ok_or_else(|| "Impossible de déterminer le dossier de données local.".to_string())?
        .join("MiracleRenpyLauncher")
        .join("backups")
        .join(id))
}

fn collect_backup_files(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_paths(directory, &mut files)?;
    Ok(files)
}

fn collect_paths(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(io_error)? {
        let path = entry.map_err(io_error)?.path();
        if path.is_dir() {
            collect_paths(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn is_save_relative_path(path: &Path) -> bool {
    let value = path.to_string_lossy().replace('\\', "/");
    value.starts_with("external/")
        || value.starts_with("game/saves/")
        || value.starts_with("saves/")
        || value.starts_with("game/persistent/")
}

pub(crate) fn resolve_save_path(
    game: &GameRecord,
    game_root: &Path,
    relative: &Path,
) -> Result<PathBuf, String> {
    let value = relative.to_string_lossy().replace('\\', "/");
    let (base, child) = save_base_and_child(game, game_root, &value)?;
    let base = fs::canonicalize(base).map_err(io_error)?;
    let candidate = base.join(child);
    let canonical = fs::canonicalize(&candidate).map_err(io_error)?;
    if !canonical.starts_with(&base) || !canonical.is_file() {
        return Err("Une sauvegarde hors des dossiers autorisés a été refusée.".to_string());
    }
    Ok(canonical)
}

fn save_target_path(
    game: &GameRecord,
    game_root: &Path,
    relative: &Path,
) -> Result<PathBuf, String> {
    let value = relative.to_string_lossy().replace('\\', "/");
    let (base, child) = save_base_and_child(game, game_root, &value)?;
    let child_path = Path::new(child);
    if child_path.is_absolute()
        || child_path
            .components()
            .any(|component| component == std::path::Component::ParentDir)
    {
        return Err("Un chemin de restauration non autorisé a été refusé.".to_string());
    }
    Ok(PathBuf::from(base).join(child_path))
}

fn save_base_and_child<'a>(
    game: &'a GameRecord,
    game_root: &'a Path,
    value: &'a str,
) -> Result<(&'a str, &'a str), String> {
    if let Some(child) = value.strip_prefix("external/") {
        Ok((
            game.save_directory
                .as_deref()
                .ok_or_else(|| "Le dossier AppData de sauvegarde est introuvable.".to_string())?,
            child,
        ))
    } else {
        Ok((
            game_root
                .to_str()
                .ok_or_else(|| "Chemin du jeu invalide.".to_string())?,
            value,
        ))
    }
}

fn prefixed_relative_path(path: &Path, relative_root: &Path, prefix: &str) -> String {
    let relative = path
        .strip_prefix(relative_root)
        .unwrap_or(path)
        .display()
        .to_string();
    if prefix.is_empty() {
        relative
    } else {
        format!("{prefix}/{relative}")
    }
}

fn temporary_path(target: &Path) -> PathBuf {
    let mut temporary = target.as_os_str().to_os_string();
    temporary.push(format!(".tmp-{}", process::id()));
    PathBuf::from(temporary)
}

fn io_error(error: impl std::fmt::Display) -> String {
    format!("Erreur de lecture des sauvegardes : {error}")
}
