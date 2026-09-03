//! Multi-browser tab inspector and redirector (Brave, Safari, Arc, Chrome, Edge)

use std::process::{Command, Stdio};

pub fn redirect_external_browsers(domains: &[String]) {
    if domains.is_empty() {
        return;
    }

    let domain_checks: Vec<String> = domains
        .iter()
        .map(|d| {
            let clean = d.trim().to_lowercase();
            format!("u contains \"{clean}\"")
        })
        .collect();

    let check_expr = domain_checks.join(" or ");
    let block_url = "file:///Users/abir/Downloads/pomotroid-bridge/blocked.html";

    let script = format!(
        r#"
        set blockedUrl to "{block_url}"

        -- Brave
        if application "Brave Browser" is running then
            tell application "Brave Browser"
                repeat with w in windows
                    repeat with t in tabs of w
                        set u to URL of t
                        if ({check_expr}) and not (u starts with blockedUrl) then
                            set URL of t to blockedUrl & "?blocked=" & u
                        end if
                    end repeat
                end repeat
            end tell
        end if

        -- Safari
        if application "Safari" is running then
            tell application "Safari"
                repeat with w in windows
                    repeat with t in tabs of w
                        set u to URL of t
                        if ({check_expr}) and not (u starts with blockedUrl) then
                            set URL of t to blockedUrl & "?blocked=" & u
                        end if
                    end repeat
                end repeat
            end tell
        end if

        -- Arc
        if application "Arc" is running then
            tell application "Arc"
                repeat with w in windows
                    repeat with t in tabs of w
                        set u to URL of t
                        if ({check_expr}) and not (u starts with blockedUrl) then
                            set URL of t to blockedUrl & "?blocked=" & u
                        end if
                    end repeat
                end repeat
            end tell
        end if

        -- Chrome
        if application "Google Chrome" is running then
            tell application "Google Chrome"
                repeat with w in windows
                    repeat with t in tabs of w
                        set u to URL of t
                        if ({check_expr}) and not (u starts with blockedUrl) then
                            set URL of t to blockedUrl & "?blocked=" & u
                        end if
                    end repeat
                end repeat
            end tell
        end if

        -- Edge
        if application "Microsoft Edge" is running then
            tell application "Microsoft Edge"
                repeat with w in windows
                    repeat with t in tabs of w
                        set u to URL of t
                        if ({check_expr}) and not (u starts with blockedUrl) then
                            set URL of t to blockedUrl & "?blocked=" & u
                        end if
                    end repeat
                end repeat
            end tell
        end if
        "#
    );

    let _ = Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
