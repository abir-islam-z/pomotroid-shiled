<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { isMac } from '$lib/utils/platform';
  import { setWindowVisibility } from '$lib/ipc';
  import { settings } from '$lib/stores/settings';
  import { timerState } from '$lib/stores/timer';
  import Tooltip from '$lib/components/Tooltip.svelte';
  import * as m from '$paraglide/messages.js';

  let {
    currentView = 'timer',
    onnavigate = () => {},
  }: {
    currentView?: 'timer' | 'stats' | 'settings';
    onnavigate?: (view: 'timer' | 'stats' | 'settings') => void;
  } = $props();

  let maximized = $state(false);
  let suppressTitlebarHover = $state(false);

  // Active theme color dynamically follows the active theme's round/focus color
  let activeThemeColor = $derived.by(() => {
    if (currentView === 'timer') {
      const rt = $timerState?.round_type;
      if (rt === 'short-break') return 'var(--color-short-round)';
      if (rt === 'long-break') return 'var(--color-long-round)';
      return 'var(--color-focus-round)';
    }
    return 'var(--color-focus-round)';
  });

  function blurTitlebarControl() {
    const active = document.activeElement;
    if (active instanceof HTMLElement && active.closest('.titlebar')) {
      active.blur();
    }
  }

  function suppressRestoredTitlebarState() {
    suppressTitlebarHover = true;
    blurTitlebarControl();
  }

  onMount(() => {
    const win = getCurrentWebviewWindow();
    win.isMaximized().then((v) => {
      maximized = v;
    });
    const unlisten = win.onResized(async () => {
      maximized = await win.isMaximized();
    });
    const clearRestoredTitlebarFocus = () => {
      if (suppressTitlebarHover) requestAnimationFrame(blurTitlebarControl);
    };
    const clearSuppressedTitlebarHover = () => {
      suppressTitlebarHover = false;
    };
    window.addEventListener('focus', clearRestoredTitlebarFocus);
    document.addEventListener('pointermove', clearSuppressedTitlebarHover);
    return () => {
      unlisten.then((fn) => fn());
      window.removeEventListener('focus', clearRestoredTitlebarFocus);
      document.removeEventListener('pointermove', clearSuppressedTitlebarHover);
    };
  });

  async function minimize() {
    suppressRestoredTitlebarState();
    if ($settings.min_to_tray) {
      await setWindowVisibility(false);
    } else {
      await getCurrentWebviewWindow().minimize();
    }
  }

  function toggleMaximize() {
    getCurrentWebviewWindow().toggleMaximize();
  }

  async function close() {
    suppressRestoredTitlebarState();
    await getCurrentWebviewWindow().close();
  }
</script>

<nav class="titlebar" class:macos={isMac} class:suppress-hover={suppressTitlebarHover} data-tauri-drag-region>
  <!-- Center: Draggable area, optionally showing subview title if active -->
  <div class="titlebar-center" data-tauri-drag-region>
    {#if currentView === 'stats'}
      <span class="view-title">Statistics</span>
    {:else if currentView === 'settings'}
      <span class="view-title">{m.settings_title ? m.settings_title() : 'Settings'}</span>
    {/if}
  </div>

  <!-- Right: Modern Top-Right Nav Icons (no tab system, standalone with proper gap) -->
  <div class="titlebar-actions" data-tauri-drag-region="false" style="--active-theme-color: {activeThemeColor};">
    <Tooltip text={m.nav_timer ? m.nav_timer() : 'Timer'} placement="below">
      <button
        class="nav-icon-btn"
        class:active={currentView === 'timer'}
        onclick={() => onnavigate('timer')}
        aria-label="Timer"
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"/>
          <polyline points="12 6 12 12 16 14"/>
        </svg>
      </button>
    </Tooltip>

    <Tooltip text={m.tooltip_statistics ? m.tooltip_statistics() : 'Statistics'} placement="below">
      <button
        class="nav-icon-btn"
        class:active={currentView === 'stats'}
        onclick={() => onnavigate('stats')}
        aria-label="Statistics"
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="20" x2="18" y2="10"/>
          <line x1="12" y1="20" x2="12" y2="4"/>
          <line x1="6" y1="20" x2="6" y2="14"/>
        </svg>
      </button>
    </Tooltip>

    <Tooltip text={m.tooltip_settings ? m.tooltip_settings() : 'Settings'} placement="below">
      <button
        class="nav-icon-btn"
        class:active={currentView === 'settings'}
        onclick={() => onnavigate('settings')}
        aria-label="Settings"
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="4" y1="21" x2="4" y2="14"/>
          <line x1="4" y1="10" x2="4" y2="3"/>
          <line x1="12" y1="21" x2="12" y2="12"/>
          <line x1="12" y1="8" x2="12" y2="3"/>
          <line x1="20" y1="21" x2="20" y2="16"/>
          <line x1="20" y1="12" x2="20" y2="3"/>
          <line x1="1" y1="14" x2="7" y2="14"/>
          <line x1="9" y1="8" x2="15" y2="8"/>
          <line x1="17" y1="16" x2="23" y2="16"/>
        </svg>
      </button>
    </Tooltip>

    <!-- Windows/Linux controls -->
    {#if !isMac}
      <div class="win-controls">
        <button class="btn-icon" onclick={minimize} aria-label="Minimize">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <line x1="1" y1="6" x2="11" y2="6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
        <button class="btn-icon" onclick={toggleMaximize} aria-label={maximized ? 'Restore' : 'Maximize'}>
          {#if maximized}
            <svg width="12" height="12" viewBox="0 0 12 12">
              <rect x="3" y="1" width="8" height="8" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <path d="M1 4 L1 11 L8 11" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          {:else}
            <svg width="12" height="12" viewBox="0 0 12 12">
              <rect x="1" y="1" width="10" height="10" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
            </svg>
          {/if}
        </button>
        <button class="btn-icon close" onclick={close} aria-label="Close">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <line x1="1" y1="1" x2="11" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="11" y1="1" x2="1" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    {/if}
  </div>
</nav>

<style>
  .titlebar {
    height: 42px;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 14px;
    position: relative;
    flex-shrink: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    background: rgba(255, 255, 255, 0.015);
  }

  .macos {
    padding-left: 76px; /* Offset for macOS traffic lights */
  }

  .titlebar-center {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    pointer-events: none;
  }

  .view-title {
    font-size: 0.8rem;
    font-weight: 550;
    letter-spacing: -0.01em;
    color: rgba(255, 255, 255, 0.72);
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
  }

  .titlebar-actions {
    display: flex;
    align-items: center;
    gap: 8px; /* Clean, modern gap between top-right icons */
    margin-left: auto;
  }

  /* Clean, Understated Matte Glassmorphic Icon Buttons */
  .nav-icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: rgba(255, 255, 255, 0.04);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 7px;
    color: var(--color-foreground-darker, rgba(255, 255, 255, 0.55));
    cursor: pointer;
    transition: all 0.18s ease;
  }

  .nav-icon-btn:hover {
    color: var(--color-foreground, #ffffff);
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.1);
  }

  /* Active state dynamically follows the active theme color */
  .nav-icon-btn.active {
    color: var(--active-theme-color, var(--color-focus-round, #FF5A52));
    background: color-mix(in srgb, var(--active-theme-color, var(--color-focus-round, #FF5A52)) 12%, rgba(255, 255, 255, 0.04));
    border-color: color-mix(in srgb, var(--active-theme-color, var(--color-focus-round, #FF5A52)) 28%, rgba(255, 255, 255, 0.06));
  }

  .nav-icon-btn.active:hover {
    background: color-mix(in srgb, var(--active-theme-color, var(--color-focus-round, #FF5A52)) 16%, rgba(255, 255, 255, 0.06));
    border-color: color-mix(in srgb, var(--active-theme-color, var(--color-focus-round, #FF5A52)) 36%, rgba(255, 255, 255, 0.08));
    color: var(--active-theme-color, var(--color-focus-round, #FF5A52));
  }

  .nav-icon-btn:active {
    transform: scale(0.94);
  }

  .win-controls {
    display: flex;
    gap: 6px;
    margin-left: 6px;
  }

  .btn-icon {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    cursor: pointer;
    color: var(--color-foreground-darker);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 6px;
    transition: all 0.15s;
  }

  .btn-icon:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.12);
  }

  .btn-icon.close:hover {
    color: #ffffff;
    background: #FF453A;
    border-color: #FF453A;
  }
</style>
