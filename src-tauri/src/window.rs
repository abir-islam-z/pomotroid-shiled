use tauri::{AppHandle, Manager};

#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;

use crate::settings;

/// Returns whether the user has configured the app to completely hide the Dock icon.
pub fn should_hide_dock(app: &AppHandle) -> bool {
    if let Some(db) = app.try_state::<crate::db::DbState>() {
        if let Ok(conn) = db.lock() {
            if let Ok(s) = settings::load(&conn) {
                return s.hide_dock_icon;
            }
        }
    }
    false
}

/// Explicitly sets the macOS application dock icon using the bundled icon image.
/// In dev mode (`npm run tauri dev`), Cargo launches the raw unbundled binary
/// which causes macOS to show the generic terminal "exec" icon in the Dock.
/// Setting `applicationIconImage` ensures the beautiful Pomotroid icon is always displayed.
#[cfg(target_os = "macos")]
pub fn ensure_dock_icon() {
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject};

    static ICON_BYTES: &[u8] = include_bytes!("../icons/icon.png");

    unsafe {
        let ns_app_class = match AnyClass::get(c"NSApplication") {
            Some(c) => c,
            None => return,
        };
        let app: *mut AnyObject = msg_send![ns_app_class, sharedApplication];
        if app.is_null() {
            return;
        }

        let ns_data_class = match AnyClass::get(c"NSData") {
            Some(c) => c,
            None => return,
        };
        let data: *mut AnyObject = msg_send![
            ns_data_class,
            dataWithBytes: ICON_BYTES.as_ptr(),
            length: ICON_BYTES.len()
        ];
        if data.is_null() {
            return;
        }

        let ns_image_class = match AnyClass::get(c"NSImage") {
            Some(c) => c,
            None => return,
        };
        let image: *mut AnyObject = msg_send![ns_image_class, alloc];
        let image: *mut AnyObject = msg_send![image, initWithData: data];
        if !image.is_null() {
            let _: () = msg_send![app, setApplicationIconImage: image];
            let _: () = msg_send![image, release];
        }
    }
}

/// Update the macOS activation policy / dock visibility.
/// On non-macOS platforms this is a no-op.
pub fn set_dock_visible(app: &AppHandle, visible: bool) {
    #[cfg(target_os = "macos")]
    {
        let policy = if visible && !should_hide_dock(app) {
            ensure_dock_icon();
            ActivationPolicy::Regular
        } else {
            ActivationPolicy::Accessory
        };
        let _ = app.set_activation_policy(policy);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, visible);
}

/// Show, unminimize, and focus the main window, restoring the macOS Dock icon
/// (unless the user has enabled `hide_dock_icon`).
pub fn show_main_window(app: &AppHandle) {
    set_dock_visible(app, true);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Hide the main window and remove the app icon from the macOS Dock.
pub fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    set_dock_visible(app, false);
}
