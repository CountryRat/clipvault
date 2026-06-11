mod clipboard;
mod commands;
mod config;
mod db;

use commands::ClipboardState;
use config::AppConfig;
use db::Database;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                // Save position before hiding
                if let Ok(pos) = window.outer_position() {
                    let app_dir = app
                        .path()
                        .app_data_dir()
                        .unwrap_or_default();
                    let mut config = AppConfig::load(&app_dir);
                    config.window_x = Some(pos.x as f64);
                    config.window_y = Some(pos.y as f64);
                    config.save(&app_dir).ok();
                }
                let _ = window.hide();
            }
            Ok(false) => {
                let _ = window.show();
                let _ = window.set_focus();
            }
            Err(e) => {
                log::error!("Failed to toggle window visibility: {}", e);
            }
        }
    } else {
        log::warn!("toggle_window: main window not found");
    }
}

/// Try to acquire a single-instance lock. Returns true if we're the first instance.
fn try_single_instance(app_dir: &std::path::Path) -> bool {
    let lock_file = app_dir.join("instance.lock");
    match std::fs::read_to_string(&lock_file) {
        Ok(pid_str) => {
            // Check if the PID is still alive
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    let output = std::process::Command::new("tasklist")
                        .args(["/FI", &format!("PID eq {}", pid)])
                        .creation_flags(0x08000000) // CREATE_NO_WINDOW
                        .output();
                    if let Ok(output) = output {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if stdout.contains(&pid.to_string()) {
                            return false; // existing instance is running
                        }
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    // On Unix, check /proc
                    if std::path::Path::new(&format!("/proc/{}", pid)).exists() {
                        return false;
                    }
                }
            }
        }
        Err(_) => {}
    }

    // Write our PID
    let pid = std::process::id();
    std::fs::write(&lock_file, pid.to_string()).ok();
    true
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            // --- App data directory ---
            let app_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| {
                    log::error!("Failed to get app data dir: {}", e);
                    e
                })
                .expect("Cannot run without an app data directory");

            // --- Single instance lock ---
            if !try_single_instance(&app_dir) {
                log::info!("Another instance is already running, exiting");
                std::process::exit(0);
            }

            log::info!("App data dir: {:?}", app_dir);

            // --- Load config ---
            let app_config = AppConfig::load(&app_dir);
            log::info!("Loaded shortcut: {}", app_config.shortcut);

            // --- Initialize database ---
            let database = Database::new(app_dir.clone()).unwrap_or_else(|e| {
                log::error!("Failed to initialize database: {}", e);
                panic!("Database init failed: {}", e);
            });
            app.manage(database);

            // --- Clipboard polling state ---
            app.manage(ClipboardState {
                last_hash: Mutex::new(None),
            });

            // --- Register global shortcut ---
            let gs = app.global_shortcut();
            match app_config.parse_shortcut() {
                Ok(shortcut) => {
                    let app_handle = app.handle().clone();
                    if let Err(e) = gs.on_shortcut(shortcut, move |_app, _s, event| {
                        if event.state == ShortcutState::Pressed {
                            toggle_window(&app_handle);
                        }
                    }) {
                        log::error!("Failed to register shortcut: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Invalid shortcut in config: {}", e);
                }
            }

            // --- System tray ---
            let show_hide =
                MenuItem::with_id(app, "toggle", "显示/隐藏", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_hide, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(
                    app.default_window_icon()
                        .cloned()
                        .unwrap_or_else(|| {
                            log::warn!("No default window icon");
                            tauri::image::Image::new(&[], 0, 0)
                        }),
                )
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle" => toggle_window(app),
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Down,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // --- Window position / close prevention ---
            if let Some(window) = app.get_webview_window("main") {
                // Restore saved position
                if let (Some(x), Some(y)) = (app_config.window_x, app_config.window_y) {
                    if x > 0.0 && y > 0.0 {
                        let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
                    }
                }

                let window_clone = window.clone();
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        // Save position before hiding
                        if let Ok(pos) = window_clone.outer_position() {
                            let app_dir = app_handle
                                .path()
                                .app_data_dir()
                                .unwrap_or_default();
                            let mut config = AppConfig::load(&app_dir);
                            config.window_x = Some(pos.x as f64);
                            config.window_y = Some(pos.y as f64);
                            config.save(&app_dir).ok();
                        }
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_clips,
            commands::toggle_pin,
            commands::delete_clip,
            commands::cleanup_clips,
            commands::check_clipboard,
            commands::hide_window,
            commands::quit_app,
            commands::get_config,
            commands::update_shortcut,
            commands::update_appearance,
            commands::save_window_position,
            commands::toggle_autostart,
            commands::mark_autostart_asked,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
