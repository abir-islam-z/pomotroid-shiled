//! Multi-browser tab inspector and redirector (Brave, Safari, Arc, Chrome, Edge)
//! Serves block screen directly from Pomotroid's localhost server (http://pomotroid.shield:1314/blocked).
//! Supports:
//! 1. Explicit user-defined adult domains
//! 2. Implicit URL & Title keyword heuristics (porn, sex, cumlouder, milf, xxx, hentai, etc.)
//! 3. Dynamic metadata/RTA rating inspection via MetaScanner
//! 4. 100% strict exclusion for localhost, local files (file://), and browser internals.

use std::process::Command;
use std::collections::HashSet;
use crate::system_bridge::hosts::DEFAULT_ADULT_DOMAINS;
use crate::system_bridge::meta_scanner::MetaScanner;

// Specific adult keywords (avoid short loose substrings like "cum" which matches "Documents")
pub const ADULT_HEURISTIC_KEYWORDS: &[&str] = &[
    "porn", "xxx", "hentai", "cumlouder", "cumshot", "cumming", "cumslut",
    "milf", "blowjob", "anal", "dildo", "fetish", "creampie", "boobs", "tits",
    "squirt", "fap", "shemale", "nude", "naked", "erotic", "bdsm",
    "threesome", "sex", "cuckold", "babe", "horny", "slut",
    "whore", "orgasm", "masturbat", "deepthroat", "pegging",
    "interracial", "stripchat", "chaturbate", "camsoda", "cam4",
    "bongacams", "myfreecams", "onlyfans", "fansly", "manyvids",
    "brazzers", "bangbros", "naughtyamerica", "realitykings",
    "kemono", "coomer", "faphouse", "fapello", "fapcat",
    "erome", "rule34", "nhentai", "hanime", "tube8",
    "beeg", "eporner", "spankbang", "redtube", "youporn",
    "xvideos", "xnxx", "xhamster", "hitomi.la", "e621.net",
    "simpcity", "thothub", "missav", "jable.tv"
];

// Title-specific adult keywords (avoid loose substrings)
pub const ADULT_TITLE_KEYWORDS: &[&str] = &[
    "porn", "sex", "xxx", "hentai", "cumlouder", "milf", "blowjob",
    "hardcore", "creampie", "erotic", "fap", "nude", "naked",
    "adult video", "sex video", "webcam", "free porn", "cam girl"
];

pub fn redirect_external_browsers(
    focus_domains: &[String],
    is_focus_active: bool,
    adult_shield_active: bool,
    explicit_adult_domains: &[String],
    meta_scanner: &MetaScanner,
) {
    if !adult_shield_active && (!is_focus_active || focus_domains.is_empty()) {
        return;
    }

    #[cfg(target_os = "macos")]
    {
        let block_url = "http://pomotroid.shield:1314/blocked";

        // Build adult URL match expression
        let mut adult_set = HashSet::new();
        for d in explicit_adult_domains {
            let clean = d.trim().to_lowercase();
            if !clean.is_empty() {
                adult_set.insert(clean);
            }
        }
        for d in DEFAULT_ADULT_DOMAINS {
            adult_set.insert(d.trim().to_lowercase());
        }
        for kw in ADULT_HEURISTIC_KEYWORDS {
            adult_set.insert(kw.trim().to_lowercase());
        }

        let adult_url_checks: Vec<String> = if adult_shield_active {
            adult_set
                .into_iter()
                .map(|kw| format!("u contains \"{kw}\""))
                .collect()
        } else {
            Vec::new()
        };
        let adult_url_expr = if adult_url_checks.is_empty() {
            "false".to_string()
        } else {
            adult_url_checks.join(" or ")
        };

        // Build adult Title match expression
        let adult_tit_checks: Vec<String> = if adult_shield_active {
            ADULT_TITLE_KEYWORDS
                .iter()
                .map(|kw| format!("tit contains \"{kw}\""))
                .collect()
        } else {
            Vec::new()
        };
        let adult_tit_expr = if adult_tit_checks.is_empty() {
            "false".to_string()
        } else {
            adult_tit_checks.join(" or ")
        };

        // Build focus domain match expression
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
            set discoveredDomains to ""

            tell application "System Events"
                set isBrave to (exists (processes where name is "Brave Browser"))
                set isChrome to (exists (processes where name is "Google Chrome"))
                set isSafari to (exists (processes where name is "Safari"))
                set isArc to (exists (processes where name is "Arc"))
                set isEdge to (exists (processes where name is "Microsoft Edge"))
            end tell

            -- Helper checking internal/local addresses that must NEVER be blocked
            -- Brave Browser
            if isBrave then
                tell application "Brave Browser"
                    repeat with w in windows
                        repeat with t in tabs of w
                            set u to URL of t
                            set isInternal to (u starts with "http://localhost" or u starts with "http://127.0.0.1" or u starts with "http://pomotroid.shield" or u starts with "file://" or u starts with "chrome://" or u starts with "brave://" or u starts with "edge://" or u starts with "arc://" or u starts with "about:" or u starts with "chrome-extension://")
                            if not isInternal then
                                set tit to title of t
                                set discoveredDomains to discoveredDomains & u & linefeed
                                if (({adult_url_expr}) or ({adult_tit_expr})) and not (u starts with blockedUrl) then
                                    set URL of t to blockedUrl & "?blocked=" & u & "&type=adult"
                                else if ({focus_expr}) and not (u starts with blockedUrl) then
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
                            if not isInternal then
                                set tit to title of t
                                set discoveredDomains to discoveredDomains & u & linefeed
                                if (({adult_url_expr}) or ({adult_tit_expr})) and not (u starts with blockedUrl) then
                                    set URL of t to blockedUrl & "?blocked=" & u & "&type=adult"
                                else if ({focus_expr}) and not (u starts with blockedUrl) then
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
                            if not isInternal then
                                set tit to title of t
                                set discoveredDomains to discoveredDomains & u & linefeed
                                if (({adult_url_expr}) or ({adult_tit_expr})) and not (u starts with blockedUrl) then
                                    set URL of t to blockedUrl & "?blocked=" & u & "&type=adult"
                                else if ({focus_expr}) and not (u starts with blockedUrl) then
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
                            if not isInternal then
                                set tit to title of t
                                set discoveredDomains to discoveredDomains & u & linefeed
                                if (({adult_url_expr}) or ({adult_tit_expr})) and not (u starts with blockedUrl) then
                                    set URL of t to blockedUrl & "?blocked=" & u & "&type=adult"
                                else if ({focus_expr}) and not (u starts with blockedUrl) then
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
                            if not isInternal then
                                set tit to title of t
                                set discoveredDomains to discoveredDomains & u & linefeed
                                if (({adult_url_expr}) or ({adult_tit_expr})) and not (u starts with blockedUrl) then
                                    set URL of t to blockedUrl & "?blocked=" & u & "&type=adult"
                                else if ({focus_expr}) and not (u starts with blockedUrl) then
                                    set URL of t to blockedUrl & "?blocked=" & u & "&type=focus"
                                end if
                            end if
                        end repeat
                    end repeat
                end tell
            end if

            return discoveredDomains
            "#
        );

        let output = Command::new("/usr/bin/osascript")
            .args(["-e", &script])
            .output();

        // Feed visited external web domains to MetaScanner
        if adult_shield_active {
            if let Ok(out) = output {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                        if !trimmed.starts_with("http://127.0.0.1")
                            && !trimmed.starts_with("http://localhost")
                            && !trimmed.starts_with("http://pomotroid.shield")
                        {
                            let clean = trimmed.split("://").nth(1).unwrap_or("");
                            let host = clean.split("/").next().unwrap_or("").split(":").next().unwrap_or("");
                            if !host.is_empty() && !host.ends_with(".local") && !host.ends_with(".test") {
                                meta_scanner.inspect_domain_async(host.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Non-macOS uses DNS host blocking
    }
}
