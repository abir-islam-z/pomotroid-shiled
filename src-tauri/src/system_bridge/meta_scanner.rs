//! Real-time HTML <head> metadata & rating inspector for Pomotroid Shield.
//! Implicitly detects adult sites via:
//! - RTA (Restricted to Adults) standards: "RTA-5042-1996-1400-1577-RTA"
//! - Rating meta tags: name="rating", property="og:rating", http-equiv="pics-label"
//! - Classification meta tags: name="classification", content="adult"
//! - Title & description adult terms

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

    /// Check if a domain has already been classified as adult
    pub fn is_domain_adult(&self, domain: &str) -> bool {
        let clean = domain.trim().to_lowercase();
        let cache = self.cache.lock().unwrap();
        cache.get(&clean).copied().unwrap_or(false)
    }

    /// Mark a domain as adult directly (e.g. from title/URL match)
    pub fn mark_adult(&self, domain: &str) {
        let clean = domain.trim().to_lowercase();
        if !clean.is_empty() {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(clean, true);
        }
    }

    /// Asynchronously inspect an unknown domain's HTML metadata
    pub fn inspect_domain_async(&self, domain: String) {
        let clean = domain.trim().to_lowercase();
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

        // Fetch only first 8KB of head/html using curl with 2-second timeout
        let output = Command::new("/usr/bin/curl")
            .args([
                "-s",
                "-L",
                "--max-time", "2",
                "-r", "0-8192",
                "-A", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko)",
                &target_url,
            ])
            .output();

        let bytes = match output {
            Ok(out) => out.stdout,
            Err(_) => return false,
        };

        let html = String::from_utf8_lossy(&bytes).to_lowercase();

        // 1. Check RTA (Restricted to Adults) universal standard tag
        if html.contains("rta-5042") || html.contains("content=\"rta") || html.contains("name=\"rta\"") {
            return true;
        }

        // 2. Check rating meta tags
        if html.contains("name=\"rating\"")
            || html.contains("http-equiv=\"rating\"")
            || html.contains("property=\"og:rating\"")
            || html.contains("name=\"rating\"")
        {
            if html.contains("adult")
                || html.contains("mature")
                || html.contains("restricted")
                || html.contains("rta")
                || html.contains("18")
            {
                return true;
            }
        }

        // 3. Check PICS / ICRA adult rating labels
        if html.contains("pics-label") && (html.contains("adult") || html.contains("sex")) {
            return true;
        }

        // 4. Check classification tags
        if html.contains("name=\"classification\"") && (html.contains("adult") || html.contains("porn") || html.contains("xxx")) {
            return true;
        }

        // 5. Check page title for high-confidence adult keywords
        if let Some(start) = html.find("<title>") {
            if let Some(end) = html[start..].find("</title>") {
                let title = &html[start + 7..start + end];
                let adult_title_words = [
                    "porn", "xxx", "hentai", "cum", "milf", "blowjob",
                    "anal sex", "dildo", "creampie", "erotic", "fap",
                    "hardcore", "adult video", "sex video", "webcam"
                ];
                for w in adult_title_words {
                    if title.contains(w) {
                        return true;
                    }
                }
            }
        }

        false
    }
}
