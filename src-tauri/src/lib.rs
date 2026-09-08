mod autostart;
mod commands;
mod db;
mod models;

use db::{init_db, DbState};
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

fn open_and_maximize_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.maximize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .or_else(|_| app.path().app_local_data_dir())
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            let _ = std::fs::create_dir_all(&app_dir);
            let db_path = app_dir.join("issues.db");
            let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
            init_db(&conn).map_err(|e| e.to_string())?;
            let _ = commands::cleanup_orphaned_attachments(&conn, app.handle());
            app.manage(DbState(Mutex::new(conn)));

            let bg_handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                    if let Some(state) = bg_handle.try_state::<DbState>() {
                        if let Ok(conn) = state.0.lock() { let _ = commands::cleanup_orphaned_attachments(&conn, &bg_handle); }
                    } else {
                        break;
                    }
                }
            });

            let show_item = MenuItem::with_id(app, "show", "Open", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        open_and_maximize_window(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        open_and_maximize_window(tray.app_handle());
                    }
                });

            if let Some(icon) = app.default_window_icon() { tray_builder = tray_builder.icon(icon.clone()); }
            tray_builder.build(app)?;

            let is_minimized = std::env::args().any(|arg| arg == "--minimized" || arg == "--autostart");
            if !is_minimized { open_and_maximize_window(app.handle()); }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_issues,
            commands::create_issue,
            commands::update_issue,
            commands::toggle_issue_status,
            commands::update_issue_meta,
            commands::get_issue_events,
            commands::get_issue_revisions,
            commands::get_labels,
            commands::create_label,
            commands::update_label,
            commands::delete_label,
            commands::get_projects,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            commands::get_autostart,
            commands::set_autostart,
            commands::save_attachment,
            commands::get_attachment
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}