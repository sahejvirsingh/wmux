use tauri::{
    webview::WebviewBuilder, AppHandle, LogicalPosition, LogicalSize, Manager, Position, Size,
    State, Url, WebviewUrl,
};

use crate::AppState;

pub fn normalize_url(input: &str) -> Result<Url, String> {
    let trimmed = input.trim();
    let candidate = if trimmed.contains("://") || trimmed.starts_with("about:") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    Url::parse(&candidate).map_err(|e| format!("invalid url: {e}"))
}

pub fn close_webview(app: &AppHandle, pane_id: &str) {
    if let Some(webview) = app.get_webview(pane_id) {
        let _ = webview.close();
    }
}

fn main_window(app: &AppHandle) -> Result<tauri::Window, String> {
    app.get_window("main")
        .ok_or_else(|| "main window not found".to_string())
}

#[tauri::command]
pub async fn browser_webview_create(
    app: AppHandle,
    pane_id: String,
    url: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if app.get_webview(&pane_id).is_some() {
        return Ok(());
    }
    let url = normalize_url(&url)?;
    let window = main_window(&app)?;
    window
        .add_child(
            WebviewBuilder::new(pane_id, WebviewUrl::External(url)),
            Position::Logical(LogicalPosition::new(x, y)),
            Size::Logical(LogicalSize::new(width.max(1.0), height.max(1.0))),
        )
        .map_err(|e| format!("create webview failed: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn browser_webview_move(
    app: AppHandle,
    pane_id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if let Some(webview) = app.get_webview(&pane_id) {
        let _ = webview.set_position(Position::Logical(LogicalPosition::new(x, y)));
        let _ = webview.set_size(Size::Logical(LogicalSize::new(width.max(1.0), height.max(1.0))));
    }
    Ok(())
}

#[tauri::command]
pub async fn browser_webview_hide(app: AppHandle, pane_id: String) -> Result<(), String> {
    if let Some(webview) = app.get_webview(&pane_id) {
        let _ = webview.hide();
    }
    Ok(())
}

#[tauri::command]
pub async fn browser_webview_show(app: AppHandle, pane_id: String) -> Result<(), String> {
    if let Some(webview) = app.get_webview(&pane_id) {
        let _ = webview.show();
    }
    Ok(())
}

#[tauri::command]
pub async fn browser_webview_close(app: AppHandle, pane_id: String) -> Result<(), String> {
    close_webview(&app, &pane_id);
    Ok(())
}

#[tauri::command]
pub async fn browser_webview_eval(
    app: AppHandle,
    pane_id: String,
    js: String,
) -> Result<(), String> {
    if let Some(webview) = app.get_webview(&pane_id) {
        webview.eval(js).map_err(|e| format!("eval failed: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn browser_navigate(
    app: AppHandle,
    state: State<'_, AppState>,
    workspace_id: String,
    pane_id: String,
    url: String,
) -> Result<(), String> {
    let parsed = normalize_url(&url)?;
    let url_string = parsed.to_string();
    if let Some(webview) = app.get_webview(&pane_id) {
        webview
            .navigate(parsed)
            .map_err(|e| format!("navigate failed: {e}"))?;
    }
    {
        let mut workspaces = state.workspaces.write();
        workspaces.set_pane_url(&workspace_id, &pane_id, url_string);
    }
    crate::commands::emit_state(&app, &state.workspaces);
    Ok(())
}
