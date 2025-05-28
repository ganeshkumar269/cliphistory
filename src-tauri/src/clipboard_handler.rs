use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use arboard::Clipboard;
use tauri::{AppHandle, Emitter, Manager};
use crate::{utils, AppState};
use crate::clip::Clip;

pub struct ClipboardHandler {
    clipboard: Clipboard,
    app_handle: tauri::AppHandle,
}

impl ClipboardHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        let initial_state_guard: tauri::State<AppState> = app_handle.state();
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Ok(initial_text) = clipboard.get_text() {
                    if !initial_text.trim().is_empty() {
                        *initial_state_guard.last_clipboard_content.lock().unwrap() = Some(initial_text.clone());
                    }
                }
                ClipboardHandler {
                    clipboard,
                    app_handle,
                }
            },
            Err(e) => {
                panic!("Setup: Failed to initialize clipboard: {:?}", e);
            }
        }
    }

    pub fn monitor_clipboard(mut self){
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(300));

                let current_app_state: tauri::State<AppState> = self.app_handle.state::<AppState>();

                match self.clipboard.get_text() {
                    Ok(current_text) => {
                        if current_text.trim().is_empty() {
                            continue;
                        }

                        let mut last_content_guard = current_app_state.last_clipboard_content.lock().unwrap();
                        let should_insert = match last_content_guard.as_deref() {
                            Some(last_known_text) if *last_known_text == current_text => false,
                            _ => true,
                        };

                        if should_insert {
                            println!("Monitor Thread: Detected new clip: {}", current_text.chars().take(50).collect::<String>());
                            let active_app_name = utils::get_active_app_name_macos().unwrap();
                            println!("active app name {}", active_app_name);

                            let mut db_gaurd = current_app_state.db.lock().unwrap();
                            db_gaurd.upsert(Clip::new(current_text.clone(), active_app_name));

                            *last_content_guard = Some(current_text.clone());
                            drop(last_content_guard);

                            self.app_handle.emit("clips_updated", current_text).unwrap_or_else(|e| {
                                eprintln!("Monitor Thread: Failed to emit clips_updated event: {}", e);
                            });
                        } else {
                            drop(last_content_guard); // Release lock if no insert
                        }
                    }
                    Err(arboard::Error::ContentNotAvailable) => { /* Normal, clipboard has non-text content */ }
                    Err(e) => {
                        eprintln!("Monitor Thread: Error reading clipboard: {:?}", e);
                    }
                }
            }
        });
    }

}
