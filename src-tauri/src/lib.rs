use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

const STORE_FILE_NAME: &str = "kanpe-store.json";
const STORE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopStore {
    version: u32,
    settings: Option<Value>,
    cards: Vec<Value>,
}

impl Default for DesktopStore {
    fn default() -> Self {
        Self {
            version: STORE_VERSION,
            settings: None,
            cards: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopInfo {
    update_channel: String,
    store_path: String,
}

#[tauri::command]
fn get_desktop_info(app: tauri::AppHandle) -> Result<DesktopInfo, String> {
    Ok(DesktopInfo {
        update_channel: update_channel(),
        store_path: store_path(&app)?.display().to_string(),
    })
}

#[tauri::command]
fn get_update_channel() -> String {
    update_channel()
}

#[tauri::command]
fn load_store(app: tauri::AppHandle) -> Result<DesktopStore, String> {
    let path = store_path(&app)?;
    if !path.exists() {
        return Ok(DesktopStore::default());
    }

    let bytes = fs::read(&path).map_err(|error| format!("Failed to read store: {error}"))?;
    let mut store: DesktopStore =
        serde_json::from_slice(&bytes).map_err(|error| format!("Failed to parse store: {error}"))?;
    store.version = STORE_VERSION;
    Ok(store)
}

#[tauri::command]
fn save_store(app: tauri::AppHandle, mut payload: DesktopStore) -> Result<(), String> {
    payload.version = STORE_VERSION;
    let path = store_path(&app)?;
    let parent = path
        .parent()
        .ok_or_else(|| "Store directory could not be resolved".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("Failed to create store directory: {error}"))?;

    let json =
        serde_json::to_vec_pretty(&payload).map_err(|error| format!("Failed to serialize store: {error}"))?;
    let temporary_path = path.with_extension("json.tmp");
    fs::write(&temporary_path, json).map_err(|error| format!("Failed to write store: {error}"))?;

    if path.exists() {
        fs::remove_file(&path).map_err(|error| format!("Failed to replace store: {error}"))?;
    }
    fs::rename(&temporary_path, &path).map_err(|error| format!("Failed to commit store: {error}"))?;
    Ok(())
}

fn store_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Failed to resolve app data directory: {error}"))?;
    Ok(dir.join(STORE_FILE_NAME))
}

fn update_channel() -> String {
    std::env::var("PIP_KANPE_UPDATE_CHANNEL").unwrap_or_else(|_| "stable".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_desktop_info,
            get_update_channel,
            load_store,
            save_store
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

