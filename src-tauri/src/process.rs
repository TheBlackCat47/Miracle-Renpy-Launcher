use crate::storage::{list_games, GameRecord};
use serde::Serialize;
use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

struct TrackedProcess {
    child: Child,
    started_at: Instant,
    game: GameRecord,
}

static PROCESSES: OnceLock<Mutex<HashMap<String, TrackedProcess>>> = OnceLock::new();

#[derive(Debug, Serialize)]
pub struct RunningGame {
    pub id: String,
    pub name: String,
    pub elapsed_seconds: u64,
}

#[tauri::command]
pub fn launch_game(id: String) -> Result<RunningGame, String> {
    let game = list_games()?
        .into_iter()
        .find(|game| game.id == id)
        .ok_or_else(|| "Jeu introuvable dans la bibliothèque.".to_string())?;
    let executable = game
        .executable
        .clone()
        .ok_or_else(|| "Aucun exécutable n’a été détecté pour ce jeu.".to_string())?;

    let processes = process_store();
    let mut processes = processes
        .lock()
        .map_err(|_| "Le suivi des processus est momentanément indisponible.".to_string())?;
    reap_finished(&mut processes);
    if let Some(process) = processes.get(&game.id) {
        return Ok(running_game(process));
    }

    let child = Command::new(&executable)
        .current_dir(&game.path)
        .spawn()
        .map_err(|error| format!("Impossible de lancer le jeu : {error}"))?;
    let process = TrackedProcess {
        child,
        started_at: Instant::now(),
        game,
    };
    let result = running_game(&process);
    processes.insert(result.id.clone(), process);
    Ok(result)
}

#[tauri::command]
pub fn get_running_games() -> Result<Vec<RunningGame>, String> {
    let processes = process_store();
    let mut processes = processes
        .lock()
        .map_err(|_| "Le suivi des processus est momentanément indisponible.".to_string())?;
    reap_finished(&mut processes);
    Ok(processes.values().map(running_game).collect())
}

fn process_store() -> &'static Mutex<HashMap<String, TrackedProcess>> {
    PROCESSES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn reap_finished(processes: &mut HashMap<String, TrackedProcess>) {
    processes.retain(|_, process| match process.child.try_wait() {
        Ok(Some(_)) | Err(_) => false,
        Ok(None) => true,
    });
}

fn running_game(process: &TrackedProcess) -> RunningGame {
    RunningGame {
        id: process.game.id.clone(),
        name: process.game.name.clone(),
        elapsed_seconds: process.started_at.elapsed().as_secs(),
    }
}
