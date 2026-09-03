//! Native macOS System Bridge for Pomotroid
//!
//! Provides system-level focus blocking (/etc/hosts),
//! continuous multi-browser URL & Title redirection (Brave, Chrome, Safari, Arc, Edge),
//! dynamic HTML metadata/RTA rating scanner, desktop media control (Spotify/Music/VLC),
//! and full-screen hardware break lock overlay.

pub mod hosts;
pub mod media;
pub mod break_lock;
pub mod browsers;
pub mod meta_scanner;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::settings::Settings;
use meta_scanner::MetaScanner;

#[derive(Clone, Default)]
struct SettingsSnapshot {
    block_enabled: bool,
    adult_shield_enabled: bool,
    blocked_domains: Vec<String>,
    adult_domains: Vec<String>,
}

pub struct SystemBridge {
    is_work_active: Arc<AtomicBool>,
    settings_cache: Arc<Mutex<SettingsSnapshot>>,
    meta_scanner: Arc<MetaScanner>,
}

impl SystemBridge {
    pub fn new() -> Self {
        let is_work_active = Arc::new(AtomicBool::new(false));
        let settings_cache = Arc::new(Mutex::new(SettingsSnapshot::default()));
        let meta_scanner = Arc::new(MetaScanner::new());

        // Continuous multi-browser background watcher thread
        let work_flag = Arc::clone(&is_work_active);
        let settings_ref = Arc::clone(&settings_cache);
        let scanner_ref = Arc::clone(&meta_scanner);

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(1500));

                let (block_enabled, adult_enabled, domains, adult_domains) = {
                    let s = settings_ref.lock().unwrap();
                    (
                        s.block_enabled,
                        s.adult_shield_enabled,
                        s.blocked_domains.clone(),
                        s.adult_domains.clone(),
                    )
                };

                let is_work = work_flag.load(Ordering::Relaxed);

                // Actively inspect and redirect tabs in Brave, Chrome, Safari, Arc, Edge
                browsers::redirect_external_browsers(
                    &domains,
                    block_enabled && is_work,
                    adult_enabled,
                    &adult_domains,
                    &scanner_ref,
                );
            }
        });

        Self {
            is_work_active,
            settings_cache,
            meta_scanner,
        }
    }

    /// Parse comma-separated blocked domains from settings
    pub fn parse_domains(raw: &str) -> Vec<String> {
        raw.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn sync_settings(&self, settings: &Settings) {
        let mut s = self.settings_cache.lock().unwrap();
        s.block_enabled = settings.system_block_enabled;
        s.adult_shield_enabled = settings.system_adult_shield_enabled;
        s.blocked_domains = Self::parse_domains(&settings.system_blocked_domains);
        s.adult_domains = Self::parse_domains(&settings.system_adult_domains);
    }

    /// Called on app startup to engage baseline protections (e.g. 24/7 Adult Shield)
    pub fn init(&self, settings: &Settings) {
        log::info!("[system_bridge] init: loading baseline protections (adult_shield={})", settings.system_adult_shield_enabled);
        self.sync_settings(settings);
        self.is_work_active.store(false, Ordering::Relaxed);
        let adult_domains = Self::parse_domains(&settings.system_adult_domains);
        let _ = hosts::update_hosts(false, &[], settings.system_adult_shield_enabled, &adult_domains);
    }

    /// Called when a Work round starts or resumes
    pub fn on_work_active(&self, settings: &Settings) {
        log::info!("[system_bridge] on_work_active: engaging focus protections");
        self.sync_settings(settings);
        self.is_work_active.store(true, Ordering::Relaxed);

        // 1. Ensure break lock overlay is closed
        break_lock::close_break_lock();

        // 2. Apply /etc/hosts site blocking
        let domains = Self::parse_domains(&settings.system_blocked_domains);
        let adult_domains = Self::parse_domains(&settings.system_adult_domains);
        if settings.system_block_enabled {
            let _ = hosts::update_hosts(true, &domains, settings.system_adult_shield_enabled, &adult_domains);
        } else {
            let _ = hosts::update_hosts(false, &[], settings.system_adult_shield_enabled, &adult_domains);
        }

        // 3. Immediate browser redirection sweep
        browsers::redirect_external_browsers(
            &domains,
            settings.system_block_enabled,
            settings.system_adult_shield_enabled,
            &adult_domains,
            &self.meta_scanner,
        );

        // 4. Resume media playback if enabled
        if settings.system_media_pause_enabled {
            media::resume_media();
        }
    }

    /// Called when a Work round is paused
    pub fn on_work_paused(&self, settings: &Settings) {
        log::info!("[system_bridge] on_work_paused: lifting site blocking & pausing media");
        self.sync_settings(settings);
        self.is_work_active.store(false, Ordering::Relaxed);

        // 1. Temporarily lift focus block so user can browse
        let adult_domains = Self::parse_domains(&settings.system_adult_domains);
        let _ = hosts::update_hosts(false, &[], settings.system_adult_shield_enabled, &adult_domains);

        // 2. Pause media playback if enabled
        if settings.system_media_pause_enabled {
            let _ = media::pause_all_media();
        }
    }

    /// Called when a Short Break starts
    pub fn on_short_break_active(&self, settings: &Settings) {
        log::info!("[system_bridge] on_short_break_active: launching break lock & pausing media");
        self.sync_settings(settings);
        self.is_work_active.store(false, Ordering::Relaxed);

        // 1. Lift focus block for break time (adult shield stays active 24/7)
        let adult_domains = Self::parse_domains(&settings.system_adult_domains);
        let _ = hosts::update_hosts(false, &[], settings.system_adult_shield_enabled, &adult_domains);

        // 2. Pause media playback
        if settings.system_media_pause_enabled {
            let _ = media::pause_all_media();
        }

        // 3. Launch full-screen hardware break lock overlay if enabled
        if settings.system_break_lock_enabled {
            break_lock::show_break_lock();
        }
    }

    /// Called when a Long Break starts
    pub fn on_long_break_active(&self, settings: &Settings) {
        log::info!("[system_bridge] on_long_break_active: pausing media");
        self.sync_settings(settings);
        self.is_work_active.store(false, Ordering::Relaxed);

        // Lift focus block (adult shield stays active 24/7)
        let adult_domains = Self::parse_domains(&settings.system_adult_domains);
        let _ = hosts::update_hosts(false, &[], settings.system_adult_shield_enabled, &adult_domains);

        // Pause media
        if settings.system_media_pause_enabled {
            let _ = media::pause_all_media();
        }

        break_lock::close_break_lock();
    }

    /// Called when timer is reset or idle
    pub fn on_timer_idle(&self, settings: &Settings) {
        log::info!("[system_bridge] on_timer_idle: cleaning up blocks and locks");
        self.sync_settings(settings);
        self.is_work_active.store(false, Ordering::Relaxed);

        let adult_domains = Self::parse_domains(&settings.system_adult_domains);
        let _ = hosts::update_hosts(false, &[], settings.system_adult_shield_enabled, &adult_domains);
        break_lock::close_break_lock();
    }

    /// Called on app shutdown
    pub fn on_shutdown(&self) {
        log::info!("[system_bridge] on_shutdown: removing all Pomotroid blocks");
        let _ = hosts::clean_all();
        break_lock::close_break_lock();
    }
}
