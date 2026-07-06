use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;

const MAIN_WINDOW_LABEL: &str = "main";
const PIP_WINDOW_LABEL: &str = "pip";
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PipWindowOptions {
    width: f64,
    height: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PipNavigatePayload {
    direction: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateCheckResult {
    available: bool,
    current_version: String,
    version: Option<String>,
    date: Option<String>,
    body: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShortcutInfo {
    previous: String,
    next: String,
    previous_registered: bool,
    next_registered: bool,
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
fn get_shortcut_info(app: tauri::AppHandle) -> ShortcutInfo {
    let (previous_shortcut, next_shortcut) = navigation_shortcuts();
    ShortcutInfo {
        previous: "Ctrl+F5".to_string(),
        next: "Ctrl+F6".to_string(),
        previous_registered: is_shortcut_registered(&app, previous_shortcut),
        next_registered: is_shortcut_registered(&app, next_shortcut),
    }
}

#[tauri::command]
fn load_store(app: tauri::AppHandle) -> Result<DesktopStore, String> {
    let path = store_path(&app)?;
    if !path.exists() {
        return Ok(DesktopStore::default());
    }

    let bytes = fs::read(&path).map_err(|error| format!("Failed to read store: {error}"))?;
    let mut store: DesktopStore = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Failed to parse store: {error}"))?;
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
    fs::create_dir_all(parent)
        .map_err(|error| format!("Failed to create store directory: {error}"))?;

    let json = serde_json::to_vec_pretty(&payload)
        .map_err(|error| format!("Failed to serialize store: {error}"))?;
    let temporary_path = path.with_extension("json.tmp");
    fs::write(&temporary_path, json).map_err(|error| format!("Failed to write store: {error}"))?;

    if path.exists() {
        fs::remove_file(&path).map_err(|error| format!("Failed to replace store: {error}"))?;
    }
    fs::rename(&temporary_path, &path)
        .map_err(|error| format!("Failed to commit store: {error}"))?;
    Ok(())
}

#[tauri::command]
async fn open_pip_window(app: tauri::AppHandle, options: PipWindowOptions) -> Result<(), String> {
    let width = options.width.clamp(320.0, 1280.0);
    let height = options.height.clamp(180.0, 720.0);

    if let Some(window) = app.get_webview_window(PIP_WINDOW_LABEL) {
        window
            .set_always_on_top(true)
            .map_err(|error| format!("Failed to keep PiP window on top: {error}"))?;
        window
            .set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)))
            .map_err(|error| format!("Failed to resize PiP window: {error}"))?;
        window
            .show()
            .map_err(|error| format!("Failed to show PiP window: {error}"))?;
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(
        &app,
        PIP_WINDOW_LABEL,
        tauri::WebviewUrl::App("pip.html".into()),
    )
    .title("PiP カンペ")
    .inner_size(width, height)
    .min_inner_size(320.0, 180.0)
    .resizable(true)
    .decorations(true)
    .always_on_top(true)
    .build()
    .map_err(|error| format!("Failed to create PiP window: {error}"))?;

    Ok(())
}

#[tauri::command]
fn update_pip_window(app: tauri::AppHandle, payload: Value) -> Result<bool, String> {
    let Some(window) = app.get_webview_window(PIP_WINDOW_LABEL) else {
        return Ok(false);
    };

    let payload_json = serde_json::to_string(&payload)
        .map_err(|error| format!("Failed to serialize PiP payload: {error}"))?;
    window
        .eval(format!("window.__PIP_KANPE_RENDER__?.({payload_json});"))
        .map_err(|error| format!("Failed to update PiP window: {error}"))?;
    Ok(true)
}

#[tauri::command]
fn request_pip_snapshot(app: tauri::AppHandle) -> Result<(), String> {
    dispatch_main_event(&app, "pip:request-snapshot", &Value::Null)
}

#[tauri::command]
fn step_pip_card(app: tauri::AppHandle, direction: i32) -> Result<(), String> {
    dispatch_main_event(
        &app,
        "pip:navigate",
        &PipNavigatePayload {
            direction: direction.signum(),
        },
    )
}

#[tauri::command]
async fn check_update(app: tauri::AppHandle) -> Result<UpdateCheckResult, String> {
    let current_version = app.package_info().version.to_string();
    let update = app
        .updater()
        .map_err(|error| format!("Failed to initialize updater: {error}"))?
        .check()
        .await
        .map_err(|error| format!("Failed to check update: {error}"))?;

    Ok(match update {
        Some(update) => UpdateCheckResult {
            available: true,
            current_version,
            version: Some(update.version),
            date: update.date.map(|date| date.to_string()),
            body: update.body,
        },
        None => UpdateCheckResult {
            available: false,
            current_version,
            version: None,
            date: None,
            body: None,
        },
    })
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

fn dispatch_main_event<T: Serialize>(
    app: &tauri::AppHandle,
    event_name: &str,
    detail: &T,
) -> Result<(), String> {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Err("Main window not found".to_string());
    };

    let event_name_json = serde_json::to_string(event_name)
        .map_err(|error| format!("Failed to serialize event name: {error}"))?;
    let detail_json = serde_json::to_string(detail)
        .map_err(|error| format!("Failed to serialize event detail: {error}"))?;
    window
        .eval(format!(
            "window.dispatchEvent(new CustomEvent({event_name_json}, {{ detail: {detail_json} }}));"
        ))
        .map_err(|error| format!("Failed to dispatch event to main window: {error}"))
}

#[cfg(desktop)]
fn navigation_shortcuts() -> (
    tauri_plugin_global_shortcut::Shortcut,
    tauri_plugin_global_shortcut::Shortcut,
) {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

    (
        Shortcut::new(Some(Modifiers::CONTROL), Code::F5),
        Shortcut::new(Some(Modifiers::CONTROL), Code::F6),
    )
}

#[cfg(desktop)]
fn is_shortcut_registered(
    app: &tauri::AppHandle,
    shortcut: tauri_plugin_global_shortcut::Shortcut,
) -> bool {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    app.global_shortcut().is_registered(shortcut)
}

#[cfg(not(desktop))]
fn is_shortcut_registered(_app: &tauri::AppHandle, _shortcut: ()) -> bool {
    false
}

#[cfg(desktop)]
fn register_global_shortcuts(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

    let (previous_shortcut, next_shortcut) = navigation_shortcuts();
    let handler_previous = previous_shortcut;
    let handler_next = next_shortcut;

    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }

                let direction = if shortcut == &handler_previous {
                    -1
                } else if shortcut == &handler_next {
                    1
                } else {
                    return;
                };

                let _ = dispatch_main_event(app, "pip:navigate", &PipNavigatePayload { direction });
            })
            .build(),
    )?;

    if let Err(error) = app.global_shortcut().register(previous_shortcut) {
        eprintln!("Failed to register Ctrl+F5 shortcut: {error}");
    }
    if let Err(error) = app.global_shortcut().register(next_shortcut) {
        eprintln!("Failed to register Ctrl+F6 shortcut: {error}");
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            #[cfg(desktop)]
            register_global_shortcuts(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_update,
            get_desktop_info,
            get_shortcut_info,
            get_update_channel,
            load_store,
            open_pip_window,
            request_pip_snapshot,
            save_store,
            step_pip_card,
            update_pip_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
