use tauri::Manager;

pub fn make_window_active(app_handle: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        Ok(())
    } else {
        eprintln!("Could not get main window instance to show.");
        Err("Could not get main window instance back from window".to_string())
    }
}