use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Serialize)]
pub struct GameInspection {
    pub path: String,
    pub folder_name: String,
    pub is_renpy: bool,
    pub confidence: &'static str,
    pub executable: Option<String>,
    pub identity_hint: String,
    pub save_directories: Vec<String>,
    pub markers: Vec<String>,
}

#[tauri::command]
pub fn inspect_game_directory(path: String) -> Result<GameInspection, String> {
    let root = PathBuf::from(path.trim());
    if !root.is_dir() {
        return Err("Le dossier indiqué n’existe pas ou n’est pas accessible.".to_string());
    }

    let folder_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Jeu Ren'Py")
        .to_string();
    let game_dir = root.join("game");
    let has_game_dir = game_dir.is_dir();
    let has_renpy_dir = game_dir.join("renpy").is_dir() || root.join("renpy").is_dir();
    let has_script_marker = has_script_marker(&game_dir);
    let executable = find_executable(&root);

    let mut markers = Vec::new();
    if has_game_dir {
        markers.push("game/".to_string());
    }
    if has_renpy_dir {
        markers.push("renpy/".to_string());
    }
    if has_script_marker {
        markers.push("scripts Ren'Py".to_string());
    }
    if executable.is_some() {
        markers.push("exécutable Windows".to_string());
    }

    let is_renpy = has_game_dir && (has_renpy_dir || has_script_marker);
    let confidence = if is_renpy && has_renpy_dir && executable.is_some() {
        "high"
    } else if is_renpy {
        "medium"
    } else {
        "none"
    };
    let save_directories = find_save_directories(&root);
    let executable_stem = executable
        .as_deref()
        .and_then(|value| Path::new(value).file_stem())
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let identity_hint = format!(
        "{}:{}",
        normalize_identity_part(&folder_name),
        normalize_identity_part(executable_stem)
    );

    Ok(GameInspection {
        path: root.display().to_string(),
        folder_name,
        is_renpy,
        confidence,
        executable,
        identity_hint,
        save_directories,
        markers,
    })
}

fn has_script_marker(game_dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(game_dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry
            .path()
            .extension()
            .is_some_and(|extension| matches!(extension.to_str(), Some("rpy" | "rpyc")))
    })
}

fn find_executable(root: &Path) -> Option<String> {
    let entries = fs::read_dir(root).ok()?;
    entries.flatten().find_map(|entry| {
        let path = entry.path();
        (path.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("exe")))
        .then(|| path.display().to_string())
    })
}

fn find_save_directories(root: &Path) -> Vec<String> {
    [
        root.join("game/saves"),
        root.join("saves"),
        root.join("game/persistent"),
    ]
    .into_iter()
    .filter(|path| path.is_dir())
    .map(|path| path.display().to_string())
    .collect()
}

fn normalize_identity_part(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '-')
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::inspect_game_directory;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn detects_a_renpy_directory_with_saves() {
        let root = tempdir().expect("temporary directory");
        fs::create_dir_all(root.path().join("game/renpy")).expect("renpy marker");
        fs::create_dir_all(root.path().join("game/saves")).expect("save directory");
        File::create(root.path().join("MyGame.exe")).expect("game executable");

        let result = inspect_game_directory(root.path().display().to_string()).expect("inspection");

        assert!(result.is_renpy);
        assert_eq!(result.confidence, "high");
        assert_eq!(result.save_directories.len(), 1);
        assert!(result.executable.is_some());
    }

    #[test]
    fn rejects_a_directory_without_renpy_markers() {
        let root = tempdir().expect("temporary directory");
        fs::create_dir(root.path().join("game")).expect("game directory");

        let result = inspect_game_directory(root.path().display().to_string()).expect("inspection");

        assert!(!result.is_renpy);
        assert_eq!(result.confidence, "none");
    }
}
