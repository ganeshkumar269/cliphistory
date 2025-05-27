use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, Position};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

mod clip;
mod clipboard_handler;
mod database;
mod utils;

pub struct AppState {
    db: Mutex<Database>,
    last_clipboard_content: Mutex<Option<String>>, // To track changes
}

impl AppState {
    fn new() -> Self {
        AppState {
            db: Mutex::new(Database::new()),
            last_clipboard_content: Mutex::new(None),
        }
    }
}

use database::Database;
use crate::clipboard_handler::ClipboardHandler;

#[tauri::command]
fn get_all_clips(state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    match state.db.lock() {
        Ok(clips_guard) => {
            let b = clips_guard.deref();
            let res = b.get_all_clips_str(1000); // nobody(me) wants more than 1000 in UI
            Ok(res)
        }
        Err(e) => Err(format!("Failed to lock clips: {}", e)),
    }
}

#[tauri::command]
fn on_search(term: String, state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    println!("Rust: Searching for term: {}", term);
    let term = term.to_lowercase(); // For case-insensitive search
    if term.trim().is_empty() {
        return get_all_clips(state);
    }

    match state.db.lock() {
        Ok(db) => {
            let filtered_clips = db
                .search(term)
                .unwrap()
                .into_iter()
                .map(|x| x.value)
                .collect::<Vec<String>>();
            Ok(filtered_clips)
        }
        Err(e) => Err(format!("Failed to lock clips for search: {}", e)),
    }
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i, &open_i])?;

            let tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        println!("quit menu item was clicked");
                        app.exit(0);
                    }
                    "open" => {
                        println!("open menu item was clicked");
                        utils::make_window_active(app);
                    }
                    _ => {
                        println!("menu item {:?} not handled", event.id);
                    }
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;
            let app_handle_for_thread: tauri::AppHandle = app.handle().clone();
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
                let shortcut_hotkey = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyC);
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new().with_handler(move |_app, shortcut, event| {
                        println!("{:?}", shortcut);
                        if shortcut == &shortcut_hotkey {
                            match event.state() {
                                ShortcutState::Pressed => {
                                    println!("Ctrl-Shift-C Pressed!");
                                    utils::make_window_active(_app);
                                }
                                ShortcutState::Released => {
                                    println!("Ctrl-Shift-C Released!");
                                }
                            }
                        }
                    })
                        .build(),
                )?;
                app.global_shortcut().register(shortcut_hotkey)?;
            }
            let clipboard_handler = ClipboardHandler::new(app_handle_for_thread);
            clipboard_handler.monitor_clipboard();
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_all_clips, on_search])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
