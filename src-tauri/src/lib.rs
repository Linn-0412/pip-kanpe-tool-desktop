use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;

const MAIN_WINDOW_LABEL: &str = "main";
const PIP_WINDOW_LABEL: &str = "pip";
const STORE_FILE_NAME: &str = "kanpe-store.json";
const STORE_VERSION: u32 = 1;
const DEFAULT_PREVIOUS_SHORTCUT: &str = "Ctrl+F5";
const DEFAULT_NEXT_SHORTCUT: &str = "Ctrl+F6";
const SUPPORT_URL: &str = "https://ofuse.me/linn0412";

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

#[derive(Debug, Clone)]
struct NavigationShortcutState {
    previous: String,
    next: String,
}

impl Default for NavigationShortcutState {
    fn default() -> Self {
        Self {
            previous: DEFAULT_PREVIOUS_SHORTCUT.to_string(),
            next: DEFAULT_NEXT_SHORTCUT.to_string(),
        }
    }
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateInstallResult {
    installed: bool,
    version: Option<String>,
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
    shortcut_info(&app)
}

#[tauri::command]
fn set_navigation_shortcuts(
    app: tauri::AppHandle,
    previous: String,
    next: String,
) -> Result<ShortcutInfo, String> {
    apply_navigation_shortcuts(&app, previous.trim(), next.trim())?;
    Ok(shortcut_info(&app))
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
    let (width, height) = clamp_pip_window_size(&options);

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
fn resize_pip_window(app: tauri::AppHandle, options: PipWindowOptions) -> Result<bool, String> {
    let Some(window) = app.get_webview_window(PIP_WINDOW_LABEL) else {
        return Ok(false);
    };

    let (width, height) = clamp_pip_window_size(&options);
    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)))
        .map_err(|error| format!("Failed to resize PiP window: {error}"))?;
    Ok(true)
}

fn clamp_pip_window_size(options: &PipWindowOptions) -> (f64, f64) {
    (
        options.width.clamp(320.0, 1280.0),
        options.height.clamp(180.0, 720.0),
    )
}

#[tauri::command]
fn open_support_url(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    app.opener()
        .open_url(SUPPORT_URL, None::<&str>)
        .map_err(|error| format!("Failed to open support URL: {error}"))
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

#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<UpdateInstallResult, String> {
    let update = app
        .updater()
        .map_err(|error| format!("Failed to initialize updater: {error}"))?
        .check()
        .await
        .map_err(|error| format!("Failed to check update: {error}"))?;

    let Some(update) = update else {
        return Ok(UpdateInstallResult {
            installed: false,
            version: None,
        });
    };

    let version = update.version.clone();
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|error| format!("Failed to install update: {error}"))?;

    Ok(UpdateInstallResult {
        installed: true,
        version: Some(version),
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
fn parse_navigation_shortcuts(
    previous: &str,
    next: &str,
) -> Result<
    (
        tauri_plugin_global_shortcut::Shortcut,
        tauri_plugin_global_shortcut::Shortcut,
    ),
    String,
> {
    if previous.eq_ignore_ascii_case(next) {
        return Err("前/次に同じショートカットは設定できません".to_string());
    }

    let previous_shortcut = previous
        .parse()
        .map_err(|error| format!("前のカンペのショートカット形式が正しくありません: {error}"))?;
    let next_shortcut = next
        .parse()
        .map_err(|error| format!("次のカンペのショートカット形式が正しくありません: {error}"))?;

    Ok((previous_shortcut, next_shortcut))
}

#[cfg(desktop)]
fn configured_navigation_shortcuts(
    app: &tauri::AppHandle,
) -> (
    tauri_plugin_global_shortcut::Shortcut,
    tauri_plugin_global_shortcut::Shortcut,
) {
    let state = app.state::<Mutex<NavigationShortcutState>>();
    let shortcuts = state.lock().unwrap().clone();

    parse_navigation_shortcuts(&shortcuts.previous, &shortcuts.next).unwrap_or_else(|_| {
        parse_navigation_shortcuts(DEFAULT_PREVIOUS_SHORTCUT, DEFAULT_NEXT_SHORTCUT)
            .expect("default shortcuts must be valid")
    })
}

#[cfg(desktop)]
fn configured_navigation_shortcut_labels(app: &tauri::AppHandle) -> (String, String) {
    let state = app.state::<Mutex<NavigationShortcutState>>();
    let shortcuts = state.lock().unwrap();
    (shortcuts.previous.clone(), shortcuts.next.clone())
}

#[cfg(desktop)]
fn is_shortcut_registered(
    app: &tauri::AppHandle,
    shortcut: tauri_plugin_global_shortcut::Shortcut,
) -> bool {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    app.global_shortcut().is_registered(shortcut)
}

#[cfg(desktop)]
fn shortcut_info(app: &tauri::AppHandle) -> ShortcutInfo {
    let (previous, next) = configured_navigation_shortcut_labels(app);
    let (previous_shortcut, next_shortcut) = configured_navigation_shortcuts(app);

    ShortcutInfo {
        previous,
        next,
        previous_registered: is_shortcut_registered(app, previous_shortcut),
        next_registered: is_shortcut_registered(app, next_shortcut),
    }
}

#[cfg(desktop)]
fn apply_navigation_shortcuts(
    app: &tauri::AppHandle,
    previous: &str,
    next: &str,
) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let (previous_shortcut, next_shortcut) = parse_navigation_shortcuts(previous, next)?;
    let (current_previous, current_next) = configured_navigation_shortcuts(app);

    let _ = app.global_shortcut().unregister(current_previous);
    let _ = app.global_shortcut().unregister(current_next);

    if let Err(error) = app.global_shortcut().register(previous_shortcut) {
        let _ = app.global_shortcut().register(current_previous);
        let _ = app.global_shortcut().register(current_next);
        return Err(format!(
            "前のカンペのショートカットを登録できませんでした: {error}"
        ));
    }

    if let Err(error) = app.global_shortcut().register(next_shortcut) {
        let _ = app.global_shortcut().unregister(previous_shortcut);
        let _ = app.global_shortcut().register(current_previous);
        let _ = app.global_shortcut().register(current_next);
        return Err(format!(
            "次のカンペのショートカットを登録できませんでした: {error}"
        ));
    }

    let state = app.state::<Mutex<NavigationShortcutState>>();
    let mut shortcuts = state.lock().unwrap();
    shortcuts.previous = previous.to_string();
    shortcuts.next = next.to_string();
    Ok(())
}

#[cfg(not(desktop))]
fn is_shortcut_registered(_app: &tauri::AppHandle, _shortcut: ()) -> bool {
    false
}

#[cfg(not(desktop))]
fn shortcut_info(_app: &tauri::AppHandle) -> ShortcutInfo {
    ShortcutInfo {
        previous: DEFAULT_PREVIOUS_SHORTCUT.to_string(),
        next: DEFAULT_NEXT_SHORTCUT.to_string(),
        previous_registered: false,
        next_registered: false,
    }
}

#[cfg(not(desktop))]
fn apply_navigation_shortcuts(
    _app: &tauri::AppHandle,
    _previous: &str,
    _next: &str,
) -> Result<(), String> {
    Ok(())
}

#[cfg(desktop)]
fn register_global_shortcuts(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::ShortcutState;

    app.manage(Mutex::new(NavigationShortcutState::default()));
    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }

                let (previous_shortcut, next_shortcut) = configured_navigation_shortcuts(app);
                let direction = if shortcut == &previous_shortcut {
                    -1
                } else if shortcut == &next_shortcut {
                    1
                } else {
                    return;
                };

                let _ = dispatch_main_event(app, "pip:navigate", &PipNavigatePayload { direction });
            })
            .build(),
    )?;

    if let Err(error) =
        apply_navigation_shortcuts(app, DEFAULT_PREVIOUS_SHORTCUT, DEFAULT_NEXT_SHORTCUT)
    {
        eprintln!("Failed to register default navigation shortcuts: {error}");
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            install_update,
            load_store,
            open_support_url,
            open_pip_window,
            request_pip_snapshot,
            resize_pip_window,
            save_store,
            set_navigation_shortcuts,
            step_pip_card,
            update_pip_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
