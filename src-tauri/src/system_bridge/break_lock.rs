//! Break lock manager.
//! 100% pure Rust — no external Swift processes or binaries.

use std::sync::atomic::{AtomicBool, Ordering};

static IS_LOCKED: AtomicBool = AtomicBool::new(false);

pub fn is_break_lock_running() -> bool {
    IS_LOCKED.load(Ordering::Relaxed)
}

pub fn show_break_lock() {
    IS_LOCKED.store(true, Ordering::Relaxed);
    log::info!("[break_lock] break lock active");
}

pub fn show_break_lock_preview() {
    IS_LOCKED.store(true, Ordering::Relaxed);
    log::info!("[break_lock] break lock preview active");
}

pub fn close_break_lock() {
    IS_LOCKED.store(false, Ordering::Relaxed);
    log::info!("[break_lock] break lock dismissed");
}
