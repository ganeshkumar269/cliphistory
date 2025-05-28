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



// #![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

// Ensure this function is only compiled on macOS
#[cfg(target_os = "macos")]
mod macos_utils {
    use objc::runtime::{Class, Object, Sel};
    use objc_id::Id;
    use objc::{msg_send, sel, sel_impl}; // <-- IMPORT sel and sel_impl
    use cocoa::base::{id as cocoa_id, nil, YES, NO}; // YES/NO might be useful
    use cocoa::foundation::{NSAutoreleasePool, NSString as CocoaNSString}; // Use Cocoa's NSString
    // For managing memory

    // Helper function to convert an Objective-C NSString to a Rust String
    fn nsstring_to_rust_string(ns_string_ptr: cocoa_id) -> Option<String> {
        if ns_string_ptr == nil {
            return None;
        }
        unsafe {
            let slice = std::slice::from_raw_parts(
                CocoaNSString::UTF8String(ns_string_ptr) as *const u8,
                CocoaNSString::len(ns_string_ptr) as usize,
            );
            match std::str::from_utf8(slice) {
                Ok(s) if !s.is_empty() => Some(s.to_owned()),
                _ => None,
            }
        }
    }

    pub fn get_frontmost_app_name() -> Option<String> {
        unsafe {
            let _pool = NSAutoreleasePool::new(nil); // Autorelease pool for memory management

            let workspace_class = Class::get("NSWorkspace")?;
            let shared_workspace: cocoa_id = msg_send![workspace_class, sharedWorkspace]; // id is *mut Object

            let frontmost_app: cocoa_id = msg_send![shared_workspace, frontmostApplication];
            if frontmost_app == nil {
                return None;
            }

            // NSRunningApplication's localizedName property
            let localized_name_nsstring: cocoa_id = msg_send![frontmost_app, localizedName];
            nsstring_to_rust_string(localized_name_nsstring)
        }
    }
}

// Example Tauri command that uses this utility
// #[tauri::command]
pub fn get_active_app_name_macos() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        if let Some(app_name) = macos_utils::get_frontmost_app_name() {
            Ok(app_name)
        } else {
            Err("Could not determine frontmost application.".to_string())
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("This command is only available on macOS.".to_string())
    }
}


