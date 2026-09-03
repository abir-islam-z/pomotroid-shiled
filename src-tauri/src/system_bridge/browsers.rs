//! Multi-browser tab inspector and redirector (Brave, Safari, Arc, Chrome, Edge)
//! Serves block screen directly from Pomotroid's localhost server (http://pomotroid.shield:1314/blocked).
//!
//! Architecture:
//! 1. Pure Metadata-Driven: Zero blacklisted domains, zero whitelisted domains.
//!    Sites are blocked solely by detecting their HTML <head> metadata (RTA, 18+ rating, schema.org).
//! 2. Never block search queries: Users can search for any topic on Google, Bing, DuckDuckGo, etc.
//!    Search engine result pages are never intercepted.
//! 3. Focus mode domain blocking: User-defined focus domains only during active focus sessions.

use std::process::Command;
use crate::system_bridge::meta_scanner::MetaScanner;

pub fn redirect_external_browsers(
    focus_domains: &[String],
    is_focus_active: bool,
    adult_shield_active: bool,
    _explicit_adult_domains: &[String],
    meta_scanner: &MetaScanner,
) {
    if !adult_shield_active && (!is_focus_active || focus_domains.is_empty()) {
        return;
    }

    #[cfg(target_os = "macos")]
    {
        let block_url = "http://pomotroid.shield:1314/blocked";

        // Adult sites discovered dynamically & implicitly via their HTML metadata
        let adult_domains: Vec<String> = if adult_shield_active {
            meta_scanner.get_adult_domains()
        } else {
            Vec::new()
        };

        let adult_checks: Vec<String> = adult_domains
            .iter()
            .map(|d| format!("u contains \"{d}\""))
            .collect();

        let adult_expr = if adult_checks.is_empty() {
            "false".to_string()
        } else {
            adult_checks.join(" or ")
        };

        // User focus domains (active only during focus rounds)
        let focus_checks: Vec<String> = if is_focus_active {
            focus_domains
                .iter()
                .map(|d| {
                    let clean = d.trim().to_lowercase();
                    format!("u contains \"{clean}\"")
                })
                .collect()
        } else {
            Vec::new()
        };
        let focus_expr = if focus_checks.is_empty() {
            "false".to_string()
        } else {
            focus_checks.join(" or ")
        };

        let script = format!(
            r#"
            set blockedUrl to "{block_url}"
            set visitedUrls to ""

            tell application "System Events"
                set isBrave to (exists (processes where name is "Brave Browser"))
                set isChrome to (exists (processes where name is "Google Chrome"))
                set isSafari to (exists (processes where name is "Safari"))
                set isArc to (exists (processes where name is "Arc"))
                set isEdge to (exists (processes where name is "Microsoft Edge"))
            end tell

            -- Brave Browser
            if isBrave then
                tell application "Brave Browser"
                    repeat with w in windows
                        repeat with t in tabs of w
                            set u to URL of t
                            set isInternal to (u starts with "http://localhost" or u starts with "http://127.0.0.1" or u starts with "http://pomotroid.shield" or u starts with "file://" or u starts with "chrome://" or u starts with "brave://" or u starts with "edge://" or u starts with "arc://" or u starts with "about:" or u starts with "chrome-extension://")
                            if not isInternal and not (u starts with blockedUrl) then
                                set isSearch to (u contains "/search?" or u contains "/search/")
                                if not isSearch then
                                    set visitedUrls to visitedUrls & u & linefeed
                                    if ({adult_expr}) then
                                        set URL of t to blockedUrl & "?blocked=" & u & "&type=adult"
                                    end if
                                end if
                                if ({focus_expr}) then
                                    set URL of t to blockedUrl & "?blocked=" & u & "&type=focus"
                                end if
                            end if
                        end repeat
                    end repeat
                end tell
            end if

            -- Google Chrome
            if isChrome then
                tell application "Google Chrome"
                    repeat with w in windows
                        repeat with t in tabs of w
                            set u to URL of t
                            set isInternal to (u starts with "http://localhost" or u starts with "http://127.0.0.1" or u starts with "http://pomotroid.shield" or u starts with "file://" or u starts with "chrome://" or u starts with "brave://" or u starts with "edge://" or u starts with "arc://" or u starts with "about:" or u starts with "chrome-extension://")
                            if not isInternal and not (u starts with blockedUrl) then
                                set isSearch to (u contains "/search?" or u contains "/search/")
                                if not isSearch then
                                    set visitedUrls to visitedUrls & u & linefeed
                                    if ({adult_expr}) then
                                        set URL of t to blockedUrl & "?blocked=" & u & "&type=adult"
                                    end if
                                end if
                                if ({focus_expr}) then
                                    set URL of t to blockedUrl & "?blocked=" & u & "&type=focus"
                                end if
                            end if
                        end repeat
                    end repeat
                end tell
            end if

            -- Safari
            if isSafari then
                tell application "Safari"
                    repeat with w in windows
                        repeat with t in tabs of w
                            set u to URL of t
                            set isInternal to (u starts with "http://localhost" or u starts with "http://127.0.0.1" or u starts with "http://pomotroid.shield" or u starts with "file://" or u starts with "chrome://" or u starts with "brave://" or u starts with "edge://" or u starts with "arc://" or u starts with "about:" or u starts with "chrome-extension://")
                            if not isInternal and not (u starts with blockedUrl) then
                                set isSearch to (u contains "/search?" or u contains "/search/")
                                if not isSearch then
                                    set visitedUrls to visitedUrls & u & linefeed
                                    if ({adult_expr}) then
                                        set URL of t to blockedUrl & "?blocked=" & u & "&type=adult"
                                    end if
                                end if
                                if ({focus_expr}) then
                                    set URL of t to blockedUrl & "?blocked=" & u & "&type=focus"
                                end if
                            end if
                        end repeat
                    end repeat
                end tell
            end if

            -- Arc
            if isArc then
                tell application "Arc"
                    repeat with w in windows
                        repeat with t in tabs of w
                            set u to URL of t
                            set isInternal to (u starts with "http://localhost" or u starts with "http://127.0.0.1" or u starts with "http://pomotroid.shield" or u starts with "file://" or u starts with "chrome://" or u starts with "brave://" or u starts with "edge://" or u starts with "arc://" or u starts with "about:" or u starts with "chrome-extension://")
                            if not isInternal and not (u starts with blockedUrl) then
                                set isSearch to (u contains "/search?" or u contains "/search/")
                                if not isSearch then
                                    set visitedUrls to visitedUrls & u & linefeed
                                    if ({adult_expr}) then
                                        set URL of t to blockedUrl & "?blocked=" & u & "&type=adult"
                                    end if
                                end if
                                if ({focus_expr}) then
                                    set URL of t to blockedUrl & "?blocked=" & u & "&type=focus"
                                end if
                            end if
                        end repeat
                    end repeat
                end tell
            end if

            -- Microsoft Edge
            if isEdge then
                tell application "Microsoft Edge"
                    repeat with w in windows
                        repeat with t in tabs of w
                            set u to URL of t
                            set isInternal to (u starts with "http://localhost" or u starts with "http://127.0.0.1" or u starts with "http://pomotroid.shield" or u starts with "file://" or u starts with "chrome://" or u starts with "brave://" or u starts with "edge://" or u starts with "arc://" or u starts with "about:" or u starts with "chrome-extension://")
                            if not isInternal and not (u starts with blockedUrl) then
                                set isSearch to (u contains "/search?" or u contains "/search/")
                                if not isSearch then
                                    set visitedUrls to visitedUrls & u & linefeed
                                    if ({adult_expr}) then
                                        set URL of t to blockedUrl & "?blocked=" & u & "&type=adult"
                                    end if
                                end if
                                if ({focus_expr}) then
                                    set URL of t to blockedUrl & "?blocked=" & u & "&type=focus"
                                end if
                            end if
                        end repeat
                    end repeat
                end tell
            end if

            return visitedUrls
            "#
        );

        let output = Command::new("/usr/bin/osascript")
            .arg("-e")
            .arg(script)
            .output();

        // Feed visited external domains to MetaScanner for implicit metadata analysis
        if adult_shield_active {
            if let Ok(out) = output {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let u = line.trim();
                    if let Some(domain) = extract_domain(u) {
                        meta_scanner.inspect_domain_async(domain);
                    }
                }
            }
        }
    }
}

/// Extract domain name from a URL string
fn extract_domain(url: &str) -> Option<String> {
    let clean = url.trim();
    let after_proto = if let Some(idx) = clean.find("://") {
        &clean[idx + 3..]
    } else {
        clean
    };
    let host_part = after_proto.split('/').next()?.split('?').next()?;
    let domain = host_part.split(':').next()?.trim().to_lowercase();
    if domain.contains('.') && !domain.starts_with("127.0.0.1") && !domain.starts_with("localhost") {
        Some(domain)
    } else {
        None
    }
}
