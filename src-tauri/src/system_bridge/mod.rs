//! Native macOS System Bridge for Pomotroid
//!
//! Provides system-level focus blocking (/etc/hosts),
//! multi-browser URL redirection, desktop media control (Spotify/Music/VLC),
//! and full-screen hardware break lock overlay (SwiftUI).

pub mod hosts;
pub mod media;
pub mod break_lock;
pub mod browsers;

use crate::settings::Settings;

pub struct SystemBridge;

impl SystemBridge {
    pub fn new() -> Self {
        Self
    }

    /// Parse comma-separated blocked domains from settings
    pub fn parse_domains(raw: &str) -> Vec<String> {
        raw.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Called when a Work round starts or resumes
    pub fn on_work_active(&self, settings: &Settings) {
        log::info!("[system_bridge] on_work_active: engaging focus protections");
        
        // 1. Ensure break lock overlay is closed
        break_lock::close_break_lock();

        // 2. Apply /etc/hosts site blocking if enabled
        if settings.system_block_enabled {
            let domains = Self::parse_domains(&settings.system_blocked_domains);
            let _ = hosts::update_hosts(true, &domains, settings.system_adult_shield_enabled);
            // 3. Redirect existing tabs in Safari, Brave, Arc, Chrome, Edge
            browsers::redirect_external_browsers(&domains);
        }

        // 4. Resume media playback if enabled
        if settings.system_media_pause_enabled {
            media::resume_media();
        }
    }

    /// Called when a Work round is paused
    pub fn on_work_paused(&self, settings: &Settings) {
        log::info!("[system_bridge] on_work_paused: lifting site blocking & pausing media");

        // 1. Temporarily lift focus block so user can browse
        let _ = hosts::update_hosts(false, &[], settings.system_adult_shield_enabled);

        // 2. Pause media playback if enabled
        if settings.system_media_pause_enabled {
            let _ = media::pause_all_media();
        }
    }

    /// Called when a Short Break starts
    pub fn on_short_break_active(&self, settings: &Settings) {
        log::info!("[system_bridge] on_short_break_active: launching break lock & pausing media");

        // 1. Lift focus block for break time
        let _ = hosts::update_hosts(false, &[], settings.system_adult_shield_enabled);

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

        // Lift focus block
        let _ = hosts::update_hosts(false, &[], settings.system_adult_shield_enabled);

        // Pause media
        if settings.system_media_pause_enabled {
            let _ = media::pause_all_media();
        }

        // Long break allows normal screen use (break lock is short-break only)
        break_lock::close_break_lock();
    }

    /// Called when timer is reset or idle
    pub fn on_timer_idle(&self, settings: &Settings) {
        log::info!("[system_bridge] on_timer_idle: cleaning up blocks and locks");
        let _ = hosts::update_hosts(false, &[], settings.system_adult_shield_enabled);
        break_lock::close_break_lock();
    }

    /// Called on app shutdown
    pub fn on_shutdown(&self) {
        log::info!("[system_bridge] on_shutdown: removing all Pomotroid blocks");
        let _ = hosts::clean_all();
        break_lock::close_break_lock();
    }
}
