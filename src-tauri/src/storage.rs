use crate::commands::inspect_game_directory;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
pub struct GameRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub save_directory: Option<String>,
    pub executable: Option<String>,
    pub confidence: String,
    pub save_count: usize,
    pub identity_hint: String,
    pub added_at: String,
}

#[tauri::command]
pub fn list_games() -> Result<Vec<GameRecord>, String> {
    let connection = open_connection()?;
    let mut statement = connection
        .prepare(
            "SELECT id, name, path, save_directory, executable, confidence, save_count, identity_hint, added_at
             FROM games ORDER BY name COLLATE NOCASE",
        )
        .map_err(db_error)?;
    let records = statement
        .query_map([], |row| {
            Ok(GameRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                save_directory: row.get(3)?,
                executable: row.get(4)?,
                confidence: row.get(5)?,
                save_count: row.get(6)?,
                identity_hint: row.get(7)?,
                added_at: row.get(8)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;

    Ok(records)
}

#[tauri::command]
pub fn register_game(path: String) -> Result<GameRecord, String> {
    let inspection = inspect_game_directory(path)?;
    if !inspection.is_renpy {
        return Err("Ce dossier ne présente pas suffisamment de marqueurs Ren’Py.".to_string());
    }

    let connection = open_connection()?;
    let id = blake3::hash(format!("{}|{}", inspection.identity_hint, inspection.path).as_bytes())
        .to_hex()
        .to_string();
    let now = unix_timestamp();
    let save_directory = find_external_save_directory(&inspection);
    let existing_id = connection
        .query_row(
            "SELECT id FROM games WHERE path = ?1",
            params![inspection.path],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(db_error)?;

    if existing_id.is_some() {
        connection
            .execute(
                "UPDATE games SET name = ?1, save_directory = ?2, executable = ?3, confidence = ?4,
                 save_count = ?5, identity_hint = ?6, updated_at = ?7 WHERE path = ?8",
                params![
                    inspection.folder_name,
                    save_directory,
                    inspection.executable,
                    inspection.confidence,
                    inspection.save_directories.len(),
                    inspection.identity_hint,
                    now,
                    inspection.path
                ],
            )
            .map_err(db_error)?;
    } else {
        connection
            .execute(
                "INSERT INTO games
                (id, name, path, save_directory, executable, confidence, save_count, identity_hint, added_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
                params![
                    id,
                    inspection.folder_name,
                    inspection.path,
                    save_directory,
                    inspection.executable,
                    inspection.confidence,
                    inspection.save_directories.len(),
                    inspection.identity_hint,
                    now
                ],
            )
            .map_err(db_error)?;
    }

    list_games()?
        .into_iter()
        .find(|game| game.path == inspection.path)
        .ok_or_else(|| "Le jeu n’a pas pu être relu après son enregistrement.".to_string())
}

pub(crate) fn open_connection() -> Result<Connection, String> {
    let data_dir = dirs_next::data_local_dir()
        .ok_or_else(|| "Impossible de déterminer le dossier de données local.".to_string())?
        .join("MiracleRenpyLauncher");
    fs::create_dir_all(&data_dir).map_err(db_error)?;
    let database_path: PathBuf = data_dir.join("mrl.sqlite");
    let connection = Connection::open(database_path).map_err(db_error)?;
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS schema_migrations (
                 version INTEGER PRIMARY KEY,
                 applied_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS games (
                 id TEXT PRIMARY KEY NOT NULL,
                 name TEXT NOT NULL,
                 path TEXT NOT NULL UNIQUE,
                 save_directory TEXT,
                 executable TEXT,
                 confidence TEXT NOT NULL,
                 save_count INTEGER NOT NULL DEFAULT 0,
                 identity_hint TEXT NOT NULL,
                 added_at TEXT NOT NULL,
                 updated_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS settings (
                 key TEXT PRIMARY KEY NOT NULL,
                 value TEXT NOT NULL,
                 updated_at TEXT NOT NULL
             );
             INSERT OR IGNORE INTO schema_migrations (version, applied_at)
             VALUES (1, strftime('%s', 'now'));",
        )
        .map_err(db_error)?;
    ensure_save_directory_column(&connection)?;
    Ok(connection)
}

fn ensure_save_directory_column(connection: &Connection) -> Result<(), String> {
    let exists = connection
        .query_row(
            "SELECT 1 FROM pragma_table_info('games') WHERE name = 'save_directory'",
            [],
            |_| Ok(()),
        )
        .optional()
        .map_err(db_error)?
        .is_some();
    if !exists {
        connection
            .execute("ALTER TABLE games ADD COLUMN save_directory TEXT", [])
            .map_err(db_error)?;
    }
    Ok(())
}

fn find_external_save_directory(inspection: &crate::commands::GameInspection) -> Option<String> {
    let appdata = dirs_next::data_dir()?.join("RenPy");
    let executable_stem = Path::new(inspection.executable.as_deref()?)
        .file_stem()?
        .to_str()
        .map(normalize_identity_part)?;
    let folder_name = normalize_identity_part(&inspection.folder_name);
    fs::read_dir(appdata)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .find(|entry| {
            let name = normalize_identity_part(&entry.file_name().to_string_lossy());
            (name.starts_with(&executable_stem) || name.starts_with(&folder_name))
                && has_save_files(&entry.path())
        })
        .map(|entry| entry.path().display().to_string())
}

fn has_save_files(path: &Path) -> bool {
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };
    entries.filter_map(Result::ok).any(|entry| {
        let path = entry.path();
        path.is_file()
            && (path
                .extension()
                .is_some_and(|extension| extension == "save")
                || path.file_name().is_some_and(|name| name == "persistent"))
    })
}

fn normalize_identity_part(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub(crate) fn get_setting(key: &str) -> Result<Option<String>, String> {
    let connection = open_connection()?;
    connection
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_error)
}

pub(crate) fn set_setting(key: &str, value: &str) -> Result<(), String> {
    let connection = open_connection()?;
    connection
        .execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, unix_timestamp()],
        )
        .map_err(db_error)?;
    Ok(())
}

fn unix_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

fn db_error(error: impl std::fmt::Display) -> String {
    format!("Erreur SQLite : {error}")
}
