<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { isMac } from '$lib/utils/platform';
  import { setWindowVisibility } from '$lib/ipc';
  import { settings } from '$lib/stores/settings';
  import * as m from '$paraglide/messages.js';
  import Tooltip from './Tooltip.svelte';

  let {
    currentView = 'timer',
    onnavigate = () => {},
  }: {
    currentView?: 'timer' | 'stats' | 'settings';
    onnavigate?: (view: 'timer' | 'stats' | 'settings') => void;
  } = $props();

  let maximized = $state(false);
  let suppressTitlebarHover = $state(false);

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
  <!-- Left section -->
  <div class="titlebar-left">
    {#if currentView !== 'timer'}
      <!-- Native Apple back button -->
      <button
        class="nav-back-btn"
        onclick={() => onnavigate('timer')}
        aria-label="Back to Timer"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6"/>
        </svg>
        <span>Timer</span>
      </button>
    {/if}
  </div>

  <!-- Center section: title when in subviews -->
  <div class="titlebar-center">
    {#if currentView === 'stats'}
      <span class="view-title">Statistics</span>
    {:else if currentView === 'settings'}
      <span class="view-title">Settings</span>
    {/if}
  </div>

  <!-- Right section: action icons or window controls -->
  <div class="controls">
    {#if currentView === 'timer'}
      <Tooltip text={m.tooltip_statistics()}>
        <button
          class="btn-icon"
          onclick={() => onnavigate('stats')}
          aria-label="Statistics"
        >
          <!-- Activity / Chart icon -->
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="20" x2="18" y2="10"/>
            <line x1="12" y1="20" x2="12" y2="4"/>
            <line x1="6" y1="20" x2="6" y2="14"/>
          </svg>
        </button>
      </Tooltip>

      <Tooltip text={m.tooltip_settings()}>
        <button
          class="btn-icon"
          onclick={() => onnavigate('settings')}
          aria-label="Settings"
        >
          <!-- Sliders icon -->
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
    {:else}
      <!-- Quick switch buttons in subviews -->
      {#if currentView === 'stats'}
        <Tooltip text={m.tooltip_settings()}>
          <button
            class="btn-icon"
            onclick={() => onnavigate('settings')}
            aria-label="Settings"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
      {:else}
        <Tooltip text={m.tooltip_statistics()}>
          <button
            class="btn-icon"
            onclick={() => onnavigate('stats')}
            aria-label="Statistics"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="20" x2="18" y2="10"/>
              <line x1="12" y1="20" x2="12" y2="4"/>
              <line x1="6" y1="20" x2="6" y2="14"/>
            </svg>
          </button>
        </Tooltip>
      {/if}

      <!-- Done button -->
      <button
        class="btn-done"
        onclick={() => onnavigate('timer')}
        aria-label="Done"
      >
        Done
      </button>
    {/if}

    {#if !isMac}
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
    padding: 0 12px;
    position: relative;
    flex-shrink: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    transition: border-color 0.2s ease;
  }

  .macos {
    padding-left: 74px; /* Space for macOS traffic lights */
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    min-width: 80px;
  }

  .titlebar-center {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    pointer-events: none;
  }

  .view-title {
    font-size: 0.82rem;
    font-weight: 500;
    letter-spacing: -0.01em;
    color: rgba(255, 255, 255, 0.75);
  }

  .nav-back-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 9px 4px 6px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    color: rgba(255, 255, 255, 0.75);
    font-size: 0.78rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .nav-back-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #ffffff;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-left: auto;
  }

  .btn-done {
    padding: 4px 10px;
    font-size: 0.76rem;
    font-weight: 500;
    background: rgba(255, 255, 255, 0.12);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    color: #ffffff;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-done:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .btn-icon {
    background: rgba(255, 255, 255, 0.05);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    cursor: pointer;
    color: var(--color-foreground-darker);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 7px;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12);
    transition: all 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .btn-icon:focus {
    outline: none;
  }

  .btn-icon:focus-visible {
    outline: 2px solid rgba(255, 255, 255, 0.3);
    outline-offset: 1px;
  }

  .titlebar:not(.suppress-hover) .btn-icon:hover {
    color: var(--color-foreground);
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.2);
    transform: scale(1.05);
  }

  .btn-icon:active {
    transform: scale(0.94);
  }

  .titlebar:not(.suppress-hover) .btn-icon.close:hover {
    color: #ffffff;
    background: #FF453A;
    border-color: #FF453A;
  }
</style>
