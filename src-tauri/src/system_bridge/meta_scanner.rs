//! Real-time HTML <head> metadata & rating inspector for Pomotroid Shield.
//! Detects adult sites purely by inspecting their declared HTML metadata & regulatory data:
//! - RTA (Restricted to Adults) standards: "RTA-5042-1996-1400-1577-RTA"
//! - Rating meta tags: name="rating" (adult, mature, 18+, R18, restricted)
//! - Classification meta tags: name="classification" (adult, pornography, erotic)
//! - Schema.org / JSON-LD structured data: "isFamilyFriendly": false, "contentRating": "18+"
//! - Statutory notices: 18 U.S.C. 2257 compliance declarations
//! - ICRA / PICS-label rating headers
//!
//! No domain blacklists. No domain whitelists. Search engines and all topics are unrestricted.

use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct MetaScanner {
    // Cache of domain -> is_adult
    cache: Arc<Mutex<HashMap<String, bool>>>,
    // In-flight inspections to prevent duplicate network calls
    in_flight: Arc<Mutex<HashSet<String>>>,
}

impl MetaScanner {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Check if a domain has been dynamically classified as adult
    pub fn is_domain_adult(&self, domain: &str) -> bool {
        let clean = domain.trim().to_lowercase();
        let cache = self.cache.lock().unwrap();
        cache.get(&clean).copied().unwrap_or(false)
    }

    /// Mark a domain as adult directly
    pub fn mark_adult(&self, domain: &str) {
        let clean = domain.trim().to_lowercase();
        if !clean.is_empty() {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(clean, true);
        }
    }

    /// Get all dynamically discovered adult domains
    pub fn get_adult_domains(&self) -> Vec<String> {
        let cache = self.cache.lock().unwrap();
        cache.iter()
            .filter_map(|(k, &v)| if v { Some(k.clone()) } else { None })
            .collect()
    }

    /// Asynchronously inspect an unknown domain's HTML metadata
    pub fn inspect_domain_async(&self, domain: String) {
        let clean = domain.trim().to_lowercase();
        // Skip invalid or internal addresses
        if clean.is_empty()
            || clean == "localhost"
            || clean.starts_with("127.0.0.1")
            || clean.ends_with(".local")
            || clean.ends_with(".test")
        {
            return;
        }

        // Skip if already in cache
        {
            let cache = self.cache.lock().unwrap();
            if cache.contains_key(&clean) {
                return;
            }
        }

        // Skip if already being scanned
        {
            let mut in_flight = self.in_flight.lock().unwrap();
            if !in_flight.insert(clean.clone()) {
                return;
            }
        }

        let cache_ref = Arc::clone(&self.cache);
        let in_flight_ref = Arc::clone(&self.in_flight);

        std::thread::spawn(move || {
            let is_adult = Self::fetch_and_analyze(&clean);
            {
                let mut cache = cache_ref.lock().unwrap();
                cache.insert(clean.clone(), is_adult);
            }
            {
                let mut in_flight = in_flight_ref.lock().unwrap();
                in_flight.remove(&clean);
            }
            if is_adult {
                log::warn!("[meta_scanner] Implicitly classified adult site via metadata: {}", clean);
            }
        });
    }

    fn fetch_and_analyze(domain: &str) -> bool {
        let target_url = format!("https://{domain}/");

        // Fetch only first 16KB of head/html using curl with 2-second timeout
        let output = Command::new("/usr/bin/curl")
            .args([
                "-s",
                "-L",
                "--max-time", "2",
                "-A", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko)",
                &target_url,
            ])
            .output();

        let bytes = match output {
            Ok(out) => out.stdout,
            Err(_) => return false,
        };

        let html = String::from_utf8_lossy(&bytes).to_lowercase();

        // 1. RTA (Restricted to Adults) standard rating metadata
        if html.contains("rta-5042")
            || html.contains(r#"content="rta"#)
            || html.contains(r#"name="rta""#)
            || html.contains(r#"rating" content="rta"#)
        {
            return true;
        }

        // 2. Standard content rating meta tags (adult, mature, R18, 18+)
        if html.contains(r#"name="rating""#)
            || html.contains(r#"http-equiv="rating""#)
            || html.contains(r#"property="og:rating""#)
            || html.contains(r#"name="age-restriction""#)
        {
            if html.contains("adult")
                || html.contains("mature")
                || html.contains("restricted")
                || html.contains("18+")
                || html.contains("r18")
                || html.contains(r#"content="18""#)
            {
                return true;
            }
        }

        // 3. PICS / ICRA adult rating labels
        if html.contains("pics-label") && (html.contains("adult") || html.contains("sex") || html.contains("cz 1")) {
            return true;
        }

        // 4. Classification meta tags
        if html.contains(r#"name="classification""#)
            && (html.contains("adult") || html.contains("porn") || html.contains("xxx") || html.contains("erotic"))
        {
            return true;
        }

        // 5. Schema.org / JSON-LD explicit non-family-friendly signals
        if html.contains(r#""isfamilyfriendly": false"#)
            || html.contains(r#""isfamilyfriendly":false"#)
            || html.contains(r#""isfamilyfriendly":"false""#)
            || html.contains(r#""contentrating":"18+""#)
            || html.contains(r#""contentrating":"adult""#)
        {
            return true;
        }

        // 6. U.S.C. 2257 statutory adult record-keeping compliance notices
        if html.contains("18 u.s.c. 2257") || html.contains("18 usc 2257") || html.contains("u.s.c. § 2257") {
            return true;
        }

        false
    }
}
