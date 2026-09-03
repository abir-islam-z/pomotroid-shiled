//! Full-screen hardware break lock overlay manager
//! Supports macOS native SwiftUI hardware shield overlay and cross-platform fallback.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn get_lock_binary_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let parent = exe.parent().unwrap_or(&exe);
        let candidate1 = parent.join("pomotroid_break_lock");
        if candidate1.exists() {
            return candidate1;
        }
        let candidate2 = parent.join("mac_break_lock");
        if candidate2.exists() {
            return candidate2;
        }
    }

    let p1 = PathBuf::from("/Users/abir/Downloads/pomotroid-with-mac-system-bridge/src-tauri/bin/pomotroid_break_lock");
    if p1.exists() {
        return p1;
    }

    let p2 = PathBuf::from("/Users/abir/Downloads/pomotroid-with-mac-system-bridge/src-tauri/bin/mac_break_lock");
    if p2.exists() {
        return p2;
    }

    PathBuf::from("/Users/abir/Downloads/pomotroid-bridge/native-host/mac_break_lock")
}

pub fn is_break_lock_running() -> bool {
    #[cfg(target_os = "macos")]
    {
        let s1 = Command::new("/usr/bin/pgrep")
            .args(["-x", "pomotroid_break_lock"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if let Ok(s) = s1 {
            if s.success() {
                return true;
            }
        }

        let s2 = Command::new("/usr/bin/pgrep")
            .args(["-x", "mac_break_lock"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match s2 {
            Ok(s) => s.success(),
            Err(_) => false,
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

pub fn show_break_lock() {
    #[cfg(target_os = "macos")]
    {
        if is_break_lock_running() {
            return;
        }

        let bin_path = get_lock_binary_path();
        if bin_path.exists() {
            let spawn_res = Command::new(&bin_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            match spawn_res {
                Ok(_) => log::info!("[break_lock] launched hardware lock overlay from {:?}", bin_path),
                Err(e) => log::error!("[break_lock] failed to spawn lock overlay: {e}"),
            }
        } else {
            log::warn!("[break_lock] binary not found at {:?}", bin_path);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        log::info!("[break_lock] non-macOS break lock triggered");
    }
}

pub fn show_break_lock_preview() {
    #[cfg(target_os = "macos")]
    {
        if is_break_lock_running() {
            close_break_lock();
        }

        let bin_path = get_lock_binary_path();
        if bin_path.exists() {
            let spawn_res = Command::new(&bin_path)
                .arg("--preview")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            match spawn_res {
                Ok(_) => log::info!("[break_lock] launched lock preview from {:?}", bin_path),
                Err(e) => log::error!("[break_lock] failed to spawn lock preview: {e}"),
            }
        } else {
            log::warn!("[break_lock] binary not found at {:?}", bin_path);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        log::info!("[break_lock] non-macOS break lock preview triggered");
    }
}

pub fn close_break_lock() {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("/usr/bin/pkill")
            .args(["-x", "pomotroid_break_lock"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        let _ = Command::new("/usr/bin/pkill")
            .args(["-x", "mac_break_lock"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        log::info!("[break_lock] closed break lock overlay");
    }

    #[cfg(not(target_os = "macos"))]
    {}
}
