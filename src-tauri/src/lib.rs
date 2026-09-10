mod commands;
mod config;
mod ipc;
mod model;
mod notifications;
mod persistence;
mod pty;
mod ssh;

use std::sync::Arc;
use std::time::Duration;

use parking_lot::{Mutex, RwLock};
use tauri::{AppHandle, Manager, RunEvent};
use tauri::menu::{Menu, MenuItem};

use crate::commands::workspace::refresh_metadata;
use crate::config::settings::{db_path, ensure_wmux_dir, load_settings, Settings};
use crate::model::WorkspaceManager;
use crate::persistence::scrollback::ScrollbackDb;
use crate::persistence::snapshot::{restore_session, save_session, session_exists};
use crate::pty::{PtyManager, PtySinkEvent};

#[derive(Clone)]
pub struct AppState {
    pub pty: PtyManager,
    pub workspaces: Arc<RwLock<WorkspaceManager>>,
    pub settings: Arc<RwLock<Settings>>,
    pub db: Arc<Mutex<ScrollbackDb>>,
    pub app: AppHandle,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            ensure_wmux_dir()?;
            let workspaces = Arc::new(RwLock::new(WorkspaceManager::new()));
            let settings = Arc::new(RwLock::new(load_settings()));
            let db_path = db_path();
            let db = Arc::new(Mutex::new(
                ScrollbackDb::open(db_path).map_err(|e| format!("open db failed: {e}"))?,
            ));

            let mut pty = PtyManager::new();
            {
                let workspaces = workspaces.clone();
                let settings = settings.clone();
                let app = app_handle.clone();
                pty.set_sink(Arc::new(move |pty_id, event| {
                    on_pty_sink_event(&app, &workspaces, &settings, pty_id, event);
                }));
            }

            let state = AppState {
                pty,
                workspaces,
                settings,
                db,
                app: app_handle.clone(),
            };
            app.manage(state.clone());

            setup_tray(app)?;

            let auto_restore = state.settings.read().auto_restore;
            if auto_restore && session_exists() {
                match restore_session(&state) {
                    Ok(n) => log::info!("restored {n} workspaces"),
                    Err(e) => log::warn!("session restore failed: {e}"),
                }
                commands::emit_state(app.handle(), &state.workspaces);
            }

            ipc::start_pipe_server(app_handle.clone());

            spawn_metadata_task(state.clone());
            spawn_periodic_save(state.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::terminal::spawn_pty_cmd,
            commands::terminal::write_pty_cmd,
            commands::terminal::resize_pty_cmd,
            commands::terminal::kill_pty_cmd,
            commands::terminal::detach_pty_cmd,
            commands::terminal::list_ptys_cmd,
            commands::workspace::get_state,
            commands::workspace::workspace_create,
            commands::workspace::workspace_delete,
            commands::workspace::workspace_rename,
            commands::workspace::workspace_select,
            commands::workspace::workspace_reorder,
            commands::workspace::pane_split,
            commands::workspace::pane_close,
            commands::workspace::pane_focus,
            commands::workspace::pane_navigate,
            commands::workspace::pane_zoom,
            commands::workspace::pane_resize,
            commands::workspace::workspace_register_pty,
            commands::workspace::workspace_attach_pty,
            commands::workspace::workspace_clear_notifications,
            commands::workspace::browser_split,
            commands::browser::browser_webview_create,
            commands::browser::browser_webview_move,
            commands::browser::browser_webview_hide,
            commands::browser::browser_webview_show,
            commands::browser::browser_webview_close,
            commands::browser::browser_webview_eval,
            commands::browser::browser_navigate,
            commands::settings::get_settings_cmd,
            commands::settings::set_settings_cmd,
            commands::settings::list_themes_cmd,
            commands::settings::hooks_status_cmd,
            commands::settings::hooks_install_cmd,
            commands::ssh::ssh_test_cmd,
            commands::ssh::ssh_connect_cmd,
            commands::ssh::ssh_save_cmd,
            commands::ssh::ssh_list_cmd,
            commands::ssh::ssh_delete_cmd,
            commands::session::session_save,
            commands::session::session_restore,
            commands::session::session_exists_cmd,
        ])
        .on_menu_event(|app, event| match event.id().as_ref() {
            "wmux-new-workspace" => {
                let state = app.state::<AppState>();
                commands::workspace::create_workspace(app, &state, None);
            }
            "wmux-restore" => {
                let state = app.state::<AppState>();
                match commands::session::restore(app, &state) {
                    Ok(n) => log::info!("restored {n} workspaces from tray"),
                    Err(e) => log::warn!("session restore failed: {e}"),
                }
            }
            "wmux-toggle" => {
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(true) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            "wmux-quit" => {
                let state = app.state::<AppState>();
                state.pty.kill_all();
                let _ = save_session(&state);
                app.exit(0);
            }
            _ => {}
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building wmux")
        .run(|app, event| {
            if let RunEvent::ExitRequested { .. } = event {
                let state = app.state::<AppState>();
                state.pty.kill_all();
                let _ = save_session(&state);
            }
        });
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let new_workspace = MenuItem::with_id(app, "wmux-new-workspace", "New Workspace", true, None::<&str>)?;
    let restore = MenuItem::with_id(app, "wmux-restore", "Restore Session", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "wmux-toggle", "Show / Hide", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "wmux-quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&new_workspace, &restore, &toggle, &quit])?;
    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

fn spawn_metadata_task(state: AppState) {
    let app = state.app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            refresh_metadata(&app, &state);
        }
    });
}

fn spawn_periodic_save(state: AppState) {
    let app = state.app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            let _ = save_session(&state);
            let _ = &app;
        }
    });
}

fn on_pty_sink_event(
    app: &AppHandle,
    workspaces: &Arc<RwLock<WorkspaceManager>>,
    settings: &Arc<RwLock<Settings>>,
    pty_id: &str,
    event: PtySinkEvent,
) {
    match event {
        PtySinkEvent::Notification(notification) => {
            let toast = settings.read().notifications.toasts;
            let marked = {
                let ws = workspaces.read();
                ws.find_workspace_by_pty(pty_id).map(|(id, _)| id)
            };
            if let Some(ws_id) = marked {
                workspaces.write().mark_attention(&ws_id, pty_id);
                commands::emit_state(app, workspaces);
            }
            if toast {
                use tauri_plugin_notification::NotificationExt;
                let _ = app
                    .notification()
                    .builder()
                    .title(notification.title)
                    .body(notification.message)
                    .show();
            }
        }
        PtySinkEvent::Title(title) => {
            let ws = workspaces.read();
            if let Some((ws_id, _)) = ws.find_workspace_by_pty(pty_id) {
                drop(ws);
                workspaces.write().set_pane_title(&ws_id, pty_id, title);
                commands::emit_state(app, workspaces);
            }
        }
        PtySinkEvent::Cwd(cwd) => {
            let ws = workspaces.read();
            if let Some((ws_id, _)) = ws.find_workspace_by_pty(pty_id) {
                drop(ws);
                workspaces.write().set_pane_cwd(&ws_id, pty_id, cwd);
                commands::emit_state(app, workspaces);
            }
        }
        PtySinkEvent::Exited(_) => {}
    }
}