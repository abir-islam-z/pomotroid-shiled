<script lang="ts">
  // Custom titlebar.
  import { onMount } from 'svelte';
  import { getCurrentWebviewWindow, WebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { isMac } from '$lib/utils/platform';
  import { setWindowVisibility } from '$lib/ipc';
  import { settings } from '$lib/stores/settings';
  import * as m from '$paraglide/messages.js';
  import Tooltip from './Tooltip.svelte';

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

  async function openSettings() {
    const existing = await WebviewWindow.getByLabel('settings');
    if (existing) {
      await existing.show();
      await existing.setFocus();
      return;
    }
    new WebviewWindow('settings', {
      url: '/settings',
      title: 'Pomotroid — Settings',
      width: 720,
      height: 520,
      decorations: isMac,
      transparent: isMac,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      windowEffects: isMac ? { effects: ['hudWindow'], state: 'active', radius: 18 } as any : undefined,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      titleBarStyle: isMac ? ('Overlay' as any) : undefined,
      hiddenTitle: isMac ? true : undefined,
      resizable: false,
      visible: false,
    });
  }

  async function openStats() {
    const existing = await WebviewWindow.getByLabel('stats');
    if (existing) {
      await existing.show();
      await existing.setFocus();
      return;
    }
    new WebviewWindow('stats', {
      url: '/stats',
      title: 'Pomotroid — Statistics',
      width: 840,
      height: 520,
      decorations: isMac,
      transparent: isMac,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      windowEffects: isMac ? { effects: ['hudWindow'], state: 'active', radius: 18 } as any : undefined,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      titleBarStyle: isMac ? ('Overlay' as any) : undefined,
      hiddenTitle: isMac ? true : undefined,
      resizable: false,
      visible: false,
    });
  }

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

{#snippet settingsBtn()}
  <Tooltip text={m.tooltip_settings()}>
    <button class="btn-icon" onclick={openSettings} aria-label="Settings">
      <!-- Apple SF Symbols-inspired Sliders -->
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
{/snippet}

{#snippet statsBtn()}
  <Tooltip text={m.tooltip_statistics()}>
    <button class="btn-icon" onclick={openStats} aria-label="Statistics">
      <!-- Apple SF Symbols-inspired Activity / Chart -->
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="20" x2="18" y2="10"/>
        <line x1="12" y1="20" x2="12" y2="4"/>
        <line x1="6" y1="20" x2="6" y2="14"/>
      </svg>
    </button>
  </Tooltip>
{/snippet}

<nav class="titlebar" class:suppress-hover={suppressTitlebarHover} data-tauri-drag-region>
  <!-- Left: on Windows/Linux action buttons go left; on macOS native traffic lights live here -->
  {#if !isMac}
    {@render settingsBtn()}
    {@render statsBtn()}
  {/if}

  <!-- Right: action buttons on macOS, window control buttons on Windows/Linux -->
  <div class="controls">
    {#if isMac}
      {@render statsBtn()}
      {@render settingsBtn()}
    {:else}
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
  }

  .controls {
    display: flex;
    gap: 6px;
    margin-left: auto;
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
