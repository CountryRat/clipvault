use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::config::AppConfig;
use crate::db::{ClipEntry, Database};

pub struct ClipboardState {
    pub last_hash: Mutex<Option<String>>,
}

#[tauri::command]
pub async fn search_clips(
    db: State<'_, Database>,
    query: String,
    limit: usize,
    offset: usize,
    pinned_only: bool,
) -> Result<Vec<ClipEntry>, String> {
    db.search(&query, limit, offset, pinned_only)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_pin(db: State<'_, Database>, id: i64) -> Result<bool, String> {
    db.toggle_pin(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_clip(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cleanup_clips(db: State<'_, Database>, keep_count: usize) -> Result<usize, String> {
    db.cleanup(keep_count).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_clipboard(
    db: State<'_, Database>,
    state: State<'_, ClipboardState>,
) -> Result<Option<ClipEntry>, String> {
    let mut last_hash = state.last_hash.lock().map_err(|e| e.to_string())?;
    match crate::clipboard::try_read_clipboard(&mut last_hash) {
        Some(entry) => {
            db.insert(&entry).map_err(|e| e.to_string())?;
            Ok(Some(entry))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn hide_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn quit_app(app: tauri::AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub async fn get_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    Ok(AppConfig::load(&app_dir))
}

#[tauri::command]
pub async fn update_shortcut(
    app: tauri::AppHandle,
    shortcut_str: String,
) -> Result<String, String> {
    // Validate by parsing
    let shortcut = crate::config::parse_shortcut_str(&shortcut_str)?;

    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    // Save to config
    let mut config = AppConfig::load(&app_dir);
    config.shortcut = shortcut_str.clone();
    config.save(&app_dir)?;

    // Re-register: unregister all, then register new shortcut
    let gs = app.global_shortcut();
    gs.unregister_all()
        .map_err(|e| format!("Failed to unregister: {}", e))?;

    let app_handle = app.clone();
    gs.on_shortcut(shortcut, move |_app, _s, event| {
        if event.state == ShortcutState::Pressed {
            if let Some(window) = app_handle.get_webview_window("main") {
                match window.is_visible() {
                    Ok(true) => {
                        let _ = window.hide();
                    }
                    Ok(false) => {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.center();
                    }
                    Err(e) => {
                        log::error!("toggle window error: {}", e);
                    }
                }
            }
        }
    })
    .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    log::info!("Shortcut updated to: {}", shortcut_str);
    Ok(shortcut_str)
}

#[tauri::command]
pub async fn update_appearance(
    app: tauri::AppHandle,
    bg_color: String,
    opacity: f32,
) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let mut config = AppConfig::load(&app_dir);
    config.bg_color = bg_color;
    config.opacity = opacity;
    config.save(&app_dir)?;

    log::info!("Appearance updated: color={}, opacity={}", config.bg_color, config.opacity);
    Ok(())
}

#[tauri::command]
pub async fn save_window_position(
    app: tauri::AppHandle,
    x: f64,
    y: f64,
) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let mut config = AppConfig::load(&app_dir);
    config.window_x = Some(x);
    config.window_y = Some(y);
    config.save(&app_dir).ok(); // best-effort
    Ok(())
}

#[tauri::command]
pub async fn toggle_autostart(
    app: tauri::AppHandle,
) -> Result<bool, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let mut config = AppConfig::load(&app_dir);
    config.autostart = !config.autostart;

    #[cfg(target_os = "windows")]
    {
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_str = exe_path.to_string_lossy();
        let run_key = "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run";
        if config.autostart {
            std::process::Command::new("reg")
                .args(["add", run_key, "/v", "ClipVault", "/d", &exe_str, "/f"])
                .output()
                .map_err(|e| format!("注册表写入失败: {}", e))?;
        } else {
            std::process::Command::new("reg")
                .args(["delete", run_key, "/v", "ClipVault", "/f"])
                .output()
                .map_err(|e| format!("注册表删除失败: {}", e))?;
        }
    }

    config.save(&app_dir)?;
    log::info!("Autostart: {}", config.autostart);
    Ok(config.autostart)
}

#[tauri::command]
pub async fn mark_autostart_asked(app: tauri::AppHandle) -> Result<(), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let mut config = AppConfig::load(&app_dir);
    config.asked_autostart = true;
    config.save(&app_dir)?;
    Ok(())
}
