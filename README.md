<div align="center">
  <img alt="Pomotroid Shield" src=".github/images/pomotroid-title.png" width="800px">
</div>

<p align="center"><strong>Pomotroid Shield</strong> — System-wide Focus Protection, Hardware Break Lock & Pomodoro Timer for macOS.</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-v2-blue?logo=tauri" alt="Tauri v2">
  <img src="https://img.shields.io/badge/Rust-1.80+-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Svelte-5-red?logo=svelte" alt="Svelte 5">
  <img src="https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey" alt="Platforms">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="License">
</p>

---

- [Overview](#overview)
- [Pomotroid Shield Features](#pomotroid-shield-features)
  - [1. System-Wide Focus Blocklist](#1-system-wide-focus-blocklist)
  - [2. 24/7 Adult Content Shield](#2-247-adult-content-shield)
  - [3. Multi-Display Hardware Break Screen Lock](#3-multi-display-hardware-break-screen-lock)
  - [4. Desktop Media Auto-Pause](#4-desktop-media-auto-pause)
  - [5. Chromium Companion Extension](#5-chromium-companion-extension)
- [Standard Pomotroid Features](#standard-pomotroid-features)
- [Statistics & Heatmap](#statistics--heatmap)
- [Themes](#themes)
- [WebSocket API](#websocket-api)
- [Installation & Setup](#installation--setup)
- [Development & Building](#development--building)
- [License](#license)

---

## Overview

**Pomotroid Shield** elevates the popular [Pomotroid](https://github.com/Splode/pomotroid) timer from a desktop app into a complete **system-level productivity and digital wellness suite**.

While standard timers remain confined to their application window, **Pomotroid Shield operates at the OS level**: enforcing distraction blocking across all desktop browsers, halting background media players, and projecting strict hardware-level break screens across all connected monitors.

Built on **Tauri 2**, **Rust**, and **Svelte 5**, with native macOS **MediaRemote** and **Cocoa** hardware window shielding.

---

## Pomotroid Shield Features

Configure these features under **Settings → Focus & System Protection**:

### 1. System-Wide Focus Blocklist
- **All-Browser Protection**: Automatically blocks distracting websites across **all desktop browsers** (Brave, Google Chrome, Safari, Arc, Microsoft Edge) during active Focus/Work rounds.
- **Dual-Layer Enforcement**:
  1. **OS Network Layer**: Routes distracting domains to `0.0.0.0` via atomic `/etc/hosts` synchronization.
  2. **Active Tab Inspection**: Scans and redirects existing browser tabs to a clean motivational block screen the moment a work round begins.
- **Customizable List**: Manage custom domains (e.g. `reddit.com`, `twitter.com`, `x.com`, `youtube.com`) directly from the Settings UI with 1-click add/remove and default reset.

### 2. 24/7 Adult Content Shield
- **Always-On Filtering**: Blocks over 100+ major adult networks, cam sites, and explicit domains 24 hours a day, 7 days a week, regardless of whether a timer is active.
- **Developer-Safe Whitelisting**: Strict internal guards protect local development (`localhost`, `127.0.0.1`, dev ports `1420`, `1314`), local files (`file:///Users/...`), and browser internal URLs (`about:`, `chrome:`, `brave:`, extensions) to prevent false positives.

### 3. Multi-Display Hardware Break Screen Lock
- **True System-Level Kiosk Shield**:
  - Automatically activates when a **Short Break** begins.
  - Spans across **all connected monitors** (`NSScreen.screens`) at `CGShieldingWindowLevel()`, shielding the desktop, hiding the Dock, and hiding the Menu Bar.
- **Strict Lockdown Mode**:
  - Enforces `.disableProcessSwitching`, `.disableForceQuit`, and blocks exit shortcuts (`ESC`, `Cmd+Q`, `Cmd+W`, `Cmd+Tab`) during break sessions.
  - Automatically reclaims key focus if an external window or notification attempts to steal the foreground.
  - Keeps the user away from distractions until the break countdown naturally finishes.
- **Mindful Aesthetic Design**:
  - **Live Circular Progress Arc**: Theme-synced countdown dial showing remaining seconds and round numbers (`Round 2 of 4`).
  - **Dynamic Wall-Clock Time**: Calculates expected unlock time (e.g. `Auto-unlocks at 5:20 PM`).
  - **Guided Breathing Exercise**: Rhythmic pulsating circle guiding mindful breathing (*Inhale... Hold... Exhale... Relax...*).
  - **Recovery Activity Cards**: Mindful nudges for stretching, hydration, and the 20-20-20 eye rest rule.
- **Preview Mode**: Test the break screen anytime via the **"Preview Break Lock Screen"** button in Settings (preview mode allows pressing `ESC` to dismiss).

### 4. Desktop Media Auto-Pause
- **Native macOS MediaRemote Integration**:
  - Interacts directly with macOS `/System/Library/PrivateFrameworks/MediaRemote.framework` using dynamic C bindings (zero AppleScript latency, sub-millisecond execution).
  - Controls macOS Control Center / Now Playing directly.
- **Universal App Support**: Pauses desktop media across Spotify, Apple Music, YouTube (in Brave/Chrome), and VLC whenever a break begins or the timer is paused.
- **System Tray Media Toggle**: Control Center media playback can also be toggled directly from the Pomotroid menu bar tray.

### 5. Chromium Companion Extension
- **Pure Timer Companion**: Lightweight Manifest V3 companion extension for Brave, Chrome, Arc, and Edge located in `/pomotroid-bridge`.
- **Live Toolbar Badge**: Displays live countdown minutes (`25m`, `04m`) and phase badges (`WRK`, `BRK`) right in your browser toolbar.
- **Local WebSocket Sync**: Connects securely to Pomotroid's local WebSocket server (`ws://127.0.0.1:1314`) with zero internet traffic or cloud dependencies.

---

## Standard Pomotroid Features

- **Configurable Timer**: Customise work duration, short break duration, long break duration, and work rounds per long break.
- **Session History & Analytics**: Daily breakdown, weekly streak tracking, and a 52-week focus heatmap.
- **38 Bundled Themes**: Including Dracula, Nord, Tokyo Night, Catppuccin, Gruvbox, Rose Pine, Crimson White, and more. Automatically switches with OS light/dark appearance.
- **Live Custom Themes**: Drop a JSON file into the themes directory for immediate hot-reloading without restarting the app.
- **8 Built-in Languages**: English, Spanish, French, German, Japanese, Simplified Chinese, Turkish, and Portuguese.
- **Dynamic Tray Icon**: Real-time progress arc and round indicator in the macOS menu bar and system tray.
- **Desktop Audio**: High-quality alert sounds for work and break completions, with optional independent work/break ticking sounds.
- **Global Shortcuts**: Control the timer (`Toggle`, `Skip`, `Reset`) globally across macOS via custom key combinations.

---

## Statistics & Heatmap

Pomotroid Shield records every completed work session into an embedded SQLite database (`pomotroid_shield.db`):
- **Daily Stats**: Total focus minutes, completed rounds, and hourly productivity distribution.
- **Weekly Trends**: Day-by-day comparison with current streak and longest streak records.
- **52-Week Heatmap**: Year-long visualization of your daily focus habits.

<div align="center">
  <img alt="Pomotroid statistics window" src=".github/images/pomotroid-stats.png" width="800px">
</div>

---

## Themes

Pomotroid Shield features full CSS custom property integration and ships with 38 themes.

![Screenshots of Pomotroid using various themes](.github/images/pomotroid-themes-preview.png)

See [THEMES.md](./THEMES.md) for the complete list of bundled themes and instructions on crafting custom themes.

---

## WebSocket API

Pomotroid Shield includes an embedded WebSocket server enabled by default on port `1314` (`ws://127.0.0.1:1314`).

### Messages

**Client → Server**

| Message | Description |
| :--- | :--- |
| `{ "type": "getState" }` | Request immediate snapshot of current timer state |

**Server → Client Events**

| Event | Payload | Description |
| :--- | :--- | :--- |
| `state` | `TimerSnapshot` | Response to `getState` |
| `timer:started` | `{ round_type, total_secs }` | Fired when timer starts countdown |
| `timer:tick` | `{ elapsed_secs, total_secs }` | Fired every second while timer is running |
| `timer:paused` | `{ elapsed_secs }` | Fired when timer is paused |
| `timer:resumed` | `{ elapsed_secs }` | Fired when timer is resumed |
| `roundChange` | `TimerSnapshot` | Fired when transitioning to next round |
| `timer:reset` | `TimerSnapshot` | Fired when timer is reset to idle |

---

## Installation & Setup

### Prerequisites (macOS)
- macOS 12.0 (Monterey) or later (Apple Silicon & Intel supported).
- Administrative permissions for `/etc/hosts` if using OS-level Focus Domain blocking.

### Installing the Chromium Extension (Optional Companion)
1. Open your browser (`chrome://extensions` or `brave://extensions`).
2. Enable **Developer mode** (top right).
3. Click **Load unpacked** and select the `pomotroid-bridge` folder.
4. The extension icon will display live timer countdowns synchronized with Pomotroid Shield.

---

## Development & Building

### Requirements
- **Node.js**: v18 or newer
- **Rust**: stable toolchain (`cargo`, `rustc`)
- **Xcode Command Line Tools** (for macOS compilation): `xcode-select --install`

### Commands

```bash
# 1. Install frontend dependencies
npm install

# 2. Compile pre-built hardware break lock binary (macOS)
swiftc -O src-tauri/swift/mac_break_lock.swift -o src-tauri/bin/mac_break_lock

# 3. Run development server (with live hot-reload)
npm run tauri dev

# 4. Run test suite
cargo test --manifest-path src-tauri/Cargo.toml --lib
npm run check

# 5. Build production application bundle (.app and .dmg)
npm run tauri build
```

---

## License

MIT &copy; [Christopher Murphy](https://github.com/Splode) & [Abir Islam](https://github.com/abir-islam-z)
