//! System-wide Hardware Break Lock Overlay Manager.
//! Spawns native macOS hardware shield overlay across all connected monitors at CGShieldingWindowLevel.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn get_lock_binary_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let parent = exe.parent().unwrap_or(&exe);
        let candidate1 = parent.join("mac_break_lock");
        if candidate1.exists() {
            return candidate1;
        }
    }

    let dev_paths = [
        PathBuf::from("/Users/abir/Downloads/pomotroid-with-mac-system-bridge/src-tauri/bin/mac_break_lock"),
        PathBuf::from("src-tauri/bin/mac_break_lock"),
        PathBuf::from("bin/mac_break_lock"),
    ];

    for p in dev_paths {
        if p.exists() {
            return p;
        }
    }

    PathBuf::from("/Users/abir/Downloads/pomotroid-with-mac-system-bridge/src-tauri/bin/mac_break_lock")
}

pub fn is_break_lock_running() -> bool {
    #[cfg(target_os = "macos")]
    {
        let status = Command::new("/usr/bin/pgrep")
            .args(["-x", "mac_break_lock"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match status {
            Ok(s) => s.success(),
            Err(_) => false,
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

pub fn show_break_lock(duration_secs: u64, round_number: u32, rounds_total: u32) {
    #[cfg(target_os = "macos")]
    {
        if is_break_lock_running() {
            return;
        }

        let bin = get_lock_binary_path();
        if bin.exists() {
            let bin_clone = bin.clone();
            std::thread::spawn(move || {
                let _ = Command::new(&bin_clone)
                    .arg("--duration")
                    .arg(duration_secs.to_string())
                    .arg("--round")
                    .arg(round_number.to_string())
                    .arg("--rounds-total")
                    .arg(rounds_total.to_string())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            });
            log::info!("[break_lock] launched system-wide hardware break lock (dur={}s, round={}/{})", duration_secs, round_number, rounds_total);
        } else {
            log::warn!("[break_lock] hardware break lock binary not found at {:?}", bin);
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

        let bin = get_lock_binary_path();
        if bin.exists() {
            let bin_clone = bin.clone();
            std::thread::spawn(move || {
                let _ = Command::new(&bin_clone)
                    .arg("--preview")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            });
            log::info!("[break_lock] launched hardware lock preview from {:?}", bin);
        } else {
            log::warn!("[break_lock] hardware break lock binary not found at {:?}", bin);
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
        std::thread::spawn(|| {
            let _ = Command::new("/usr/bin/pkill")
                .args(["-x", "mac_break_lock"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        });
        log::info!("[break_lock] closed system-wide break lock");
    }

    #[cfg(not(target_os = "macos"))]
    {}
}
