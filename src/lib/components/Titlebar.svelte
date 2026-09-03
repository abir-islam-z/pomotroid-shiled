<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { isMac } from '$lib/utils/platform';
  import { setWindowVisibility } from '$lib/ipc';
  import { settings } from '$lib/stores/settings';

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
  <!-- Center: Modern macOS Segmented View Controller -->
  <div class="nav-segmented" data-tauri-drag-region="false">
    <button
      class="nav-tab"
      class:active={currentView === 'timer'}
      onclick={() => onnavigate('timer')}
      aria-label="Timer View"
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10"/>
        <polyline points="12 6 12 12 16 14"/>
      </svg>
      <span>Timer</span>
    </button>

    <button
      class="nav-tab"
      class:active={currentView === 'stats'}
      onclick={() => onnavigate('stats')}
      aria-label="Statistics View"
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="20" x2="18" y2="10"/>
        <line x1="12" y1="20" x2="12" y2="4"/>
        <line x1="6" y1="20" x2="6" y2="14"/>
      </svg>
      <span>Stats</span>
    </button>

    <button
      class="nav-tab"
      class:active={currentView === 'settings'}
      onclick={() => onnavigate('settings')}
      aria-label="Settings View"
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" stroke-linecap="round" stroke-linejoin="round">
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
      <span>Settings</span>
    </button>
  </div>

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
</nav>

<style>
  .titlebar {
    height: 44px;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 12px;
    position: relative;
    flex-shrink: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    background: rgba(255, 255, 255, 0.015);
  }

  .macos {
    padding-left: 74px; /* Proper offset for macOS traffic lights */
  }

  /* Modern macOS Segmented Control */
  .nav-segmented {
    display: inline-flex;
    align-items: center;
    background: rgba(0, 0, 0, 0.28);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 2.5px;
    gap: 2px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  .nav-tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    background: transparent;
    border: none;
    border-radius: 6px;
    font-size: 0.77rem;
    font-weight: 450;
    color: rgba(255, 255, 255, 0.62);
    cursor: pointer;
    transition: all 0.16s cubic-bezier(0.16, 1, 0.3, 1);
    white-space: nowrap;
  }

  .nav-tab:hover {
    color: rgba(255, 255, 255, 0.95);
    background: rgba(255, 255, 255, 0.05);
  }

  .nav-tab.active {
    background: rgba(255, 255, 255, 0.14);
    color: #ffffff;
    font-weight: 500;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.35);
  }

  .win-controls {
    display: flex;
    gap: 6px;
    margin-left: auto;
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
