//! Cross-platform desktop media player controller (Spotify, Apple Music, VLC, MPRIS)
//! Supports macOS (AppleScript), Linux (playerctl / MPRIS), and Windows (PowerShell / Media Key).

use std::process::{Command, Stdio};

pub fn get_playing_apps() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        let script = r#"
            set playingList to ""
            if application "Spotify" is running then
                tell application "Spotify"
                    if player state is playing then set playingList to playingList & "Spotify,"
                end tell
            end if
            if application "Music" is running then
                tell application "Music"
                    if player state is playing then set playingList to playingList & "Music,"
                end tell
            end if
            if application "VLC" is running then
                tell application "VLC"
                    if playing then set playingList to playingList & "VLC,"
                end tell
            end if
            return playingList
        "#;

        let output = match Command::new("/usr/bin/osascript")
            .args(["-e", script])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
        {
            Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
            Err(_) => return Vec::new(),
        };

        output
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("playerctl")
            .args(["-l"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        match output {
            Ok(out) => String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    #[cfg(target_os = "windows")]
    {
        Vec::new()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Vec::new()
    }
}

pub fn pause_all_media() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        let script = r#"
            set pausedList to ""
            if application "Spotify" is running then
                tell application "Spotify"
                    if player state is playing then
                        pause
                        set pausedList to pausedList & "Spotify,"
                    end if
                end tell
            end if
            if application "Music" is running then
                tell application "Music"
                    if player state is playing then
                        pause
                        set pausedList to pausedList & "Music,"
                    end if
                end tell
            end if
            if application "VLC" is running then
                tell application "VLC"
                    if playing then
                        pause
                        set pausedList to pausedList & "VLC,"
                    end if
                end tell
            end if
            return pausedList
        "#;

        let output = match Command::new("/usr/bin/osascript")
            .args(["-e", script])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
        {
            Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
            Err(_) => return Vec::new(),
        };

        let paused: Vec<String> = output
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if !paused.is_empty() {
            log::info!("[media] paused desktop players: {:?}", paused);
        }
        paused
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("playerctl")
            .args(["-a", "pause"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        log::info!("[media] paused players via playerctl");
        vec!["playerctl".to_string()]
    }

    #[cfg(target_os = "windows")]
    {
        // Send virtual key for Media Play/Pause (0xB3) via PowerShell
        let _ = Command::new("powershell")
            .args(["-Command", " = New-Object -ComObject Wscript.Shell; .SendKeys([char]179)"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        vec!["WindowsMedia".to_string()]
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Vec::new()
    }
}

pub fn resume_media() {
    #[cfg(target_os = "macos")]
    {
        let script = r#"
            if application "Spotify" is running then
                tell application "Spotify"
                    if player state is paused then play
                end tell
            else if application "Music" is running then
                tell application "Music"
                    if player state is paused then play
                end tell
            else if application "VLC" is running then
                tell application "VLC"
                    if not playing then play
                end tell
            end if
        "#;

        let _ = Command::new("/usr/bin/osascript")
            .args(["-e", script])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        log::info!("[media] resumed desktop media player");
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("playerctl")
            .args(["-a", "play"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        log::info!("[media] resumed players via playerctl");
    }

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("powershell")
            .args(["-Command", " = New-Object -ComObject Wscript.Shell; .SendKeys([char]179)"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {}
}
