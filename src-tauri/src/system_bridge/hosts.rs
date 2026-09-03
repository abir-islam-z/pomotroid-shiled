//! Cross-platform /etc/hosts manager for Pomotroid
//! Supports macOS, Windows, and Linux with platform-specific paths and DNS flush commands.

use std::fs;
use std::process::{Command, Stdio};

#[cfg(target_os = "windows")]
pub const HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";

#[cfg(not(target_os = "windows"))]
pub const HOSTS_PATH: &str = "/etc/hosts";

pub const FOCUS_START: &str = "# === POMOTROID FOCUS BLOCK START ===";
pub const FOCUS_END: &str = "# === POMOTROID FOCUS BLOCK END ===";
pub const ADULT_START: &str = "# === POMOTROID 24/7 ADULT SHIELD START ===";
pub const ADULT_END: &str = "# === POMOTROID 24/7 ADULT SHIELD END ===";

pub const DEFAULT_ADULT_DOMAINS: &[&str] = &[
    "pornhub.com", "xvideos.com", "xnxx.com", "xhamster.com",
    "redtube.com", "youporn.com", "chaturbate.com", "onlyfans.com",
    "stripchat.com", "livejasmin.com", "cam4.com", "bongacams.com",
    "eporner.com", "spankbang.com", "tube8.com", "beeg.com",
    "hqporner.com", "tnaflix.com", "motherless.com", "heavy-r.com",
    "faphouse.com", "brazzers.com", "bangbros.com", "naughtyamerica.com",
    "realitykings.com", "porn.com", "rule34.xxx", "e-hentai.org",
    "gelbooru.com", "danbooru.donmai.us", "nhentai.net", "hanime.tv",
    "luscious.net", "erome.com", "coomer.party", "kemono.party",
];

pub fn is_hosts_writable() -> bool {
    let metadata = match fs::metadata(HOSTS_PATH) {
        Ok(m) => m,
        Err(_) => return false,
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        (mode & 0o222) != 0
    }
    #[cfg(windows)]
    {
        !metadata.permissions().readonly()
    }
}

pub fn flush_dns_cache() {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("/usr/bin/dscacheutil")
            .arg("-flushcache")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        let _ = Command::new("/usr/bin/killall")
            .args(["-HUP", "mDNSResponder"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("ipconfig")
            .arg("/flushdns")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    #[cfg(target_os = "linux")]
    {
        // Try resolvectl first, fallback to systemd-resolve
        let _ = Command::new("resolvectl")
            .arg("flush-caches")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn remove_section(content: &str, start_marker: &str, end_marker: &str) -> String {
    let mut result = Vec::new();
    let mut in_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == start_marker {
            in_section = true;
            continue;
        }
        if trimmed == end_marker {
            in_section = false;
            continue;
        }
        if !in_section {
            result.push(line);
        }
    }

    result.join("
")
}

fn expand_domain(domain: &str) -> Vec<String> {
    let mut d = domain.trim().to_lowercase();
    if let Some(stripped) = d.strip_prefix("https://") {
        d = stripped.to_string();
    } else if let Some(stripped) = d.strip_prefix("http://") {
        d = stripped.to_string();
    }
    if let Some(idx) = d.find('/') {
        d = d[..idx].to_string();
    }
    if let Some(idx) = d.find(':') {
        d = d[..idx].to_string();
    }
    if d.is_empty() {
        return Vec::new();
    }

    let mut res = vec![d.clone()];
    if d.starts_with("www.") {
        res.push(d[4..].to_string());
    } else {
        res.push(format!("www.{d}"));
    }
    res
}

fn build_section(start_marker: &str, end_marker: &str, title: &str, domains: &[String]) -> String {
    let mut out = String::new();
    out.push_str("
");
    out.push_str(start_marker);
    out.push_str("
");
    out.push_str(&format!("# {title} (Managed automatically by Pomotroid)
"));

    let mut seen = std::collections::HashSet::new();
    for d in domains {
        for exp in expand_domain(d) {
            if seen.insert(exp.clone()) {
                out.push_str(&format!("0.0.0.0 {exp}
"));
                out.push_str(&format!("::1 {exp}
"));
            }
        }
    }

    out.push_str(end_marker);
    out.push_str("
");
    out
}

pub fn update_hosts(
    focus_blocked: bool,
    focus_domains: &[String],
    adult_blocked: bool,
) -> Result<(), String> {
    let existing = fs::read_to_string(HOSTS_PATH).map_err(|e| e.to_string())?;

    let mut cleaned = remove_section(&existing, FOCUS_START, FOCUS_END);
    cleaned = remove_section(&cleaned, ADULT_START, ADULT_END);
    let mut content = cleaned.trim_end().to_string();

    if focus_blocked && !focus_domains.is_empty() {
        content.push_str(&build_section(
            FOCUS_START,
            FOCUS_END,
            "ACTIVE FOCUS SESSION BLOCK",
            focus_domains,
        ));
    }

    if adult_blocked {
        let adult_strings: Vec<String> = DEFAULT_ADULT_DOMAINS.iter().map(|s| s.to_string()).collect();
        content.push_str(&build_section(
            ADULT_START,
            ADULT_END,
            "24/7 ADULT CONTENT SHIELD",
            &adult_strings,
        ));
    }

    content.push('\n');

    fs::write(HOSTS_PATH, content).map_err(|e| e.to_string())?;
    flush_dns_cache();

    log::info!("[hosts] updated {}: focus={focus_blocked}, adult={adult_blocked}", HOSTS_PATH);
    Ok(())
}

pub fn clean_all() -> Result<(), String> {
    let existing = fs::read_to_string(HOSTS_PATH).map_err(|e| e.to_string())?;
    let mut cleaned = remove_section(&existing, FOCUS_START, FOCUS_END);
    cleaned = remove_section(&cleaned, ADULT_START, ADULT_END);
    cleaned.push('\n');

    fs::write(HOSTS_PATH, cleaned).map_err(|e| e.to_string())?;
    flush_dns_cache();
    log::info!("[hosts] removed all Pomotroid blocks from {}", HOSTS_PATH);
    Ok(())
}
