use crate::commands::inspect_game_directory;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
pub struct GameRecord {
    pub id: String,
    pub name: String,
    pub path: String,
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
            "SELECT id, name, path, executable, confidence, save_count, identity_hint, added_at
             FROM games ORDER BY name COLLATE NOCASE",
        )
        .map_err(db_error)?;
    let records = statement
        .query_map([], |row| {
            Ok(GameRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                executable: row.get(3)?,
                confidence: row.get(4)?,
                save_count: row.get(5)?,
                identity_hint: row.get(6)?,
                added_at: row.get(7)?,
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
                "UPDATE games SET name = ?1, executable = ?2, confidence = ?3,
                 save_count = ?4, identity_hint = ?5, updated_at = ?6 WHERE path = ?7",
                params![
                    inspection.folder_name,
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
                 (id, name, path, executable, confidence, save_count, identity_hint, added_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
                params![
                    id,
                    inspection.folder_name,
                    inspection.path,
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

fn open_connection() -> Result<Connection, String> {
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
                 executable TEXT,
                 confidence TEXT NOT NULL,
                 save_count INTEGER NOT NULL DEFAULT 0,
                 identity_hint TEXT NOT NULL,
                 added_at TEXT NOT NULL,
                 updated_at TEXT NOT NULL
             );
             INSERT OR IGNORE INTO schema_migrations (version, applied_at)
             VALUES (1, strftime('%s', 'now'));",
        )
        .map_err(db_error)?;
    Ok(connection)
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
