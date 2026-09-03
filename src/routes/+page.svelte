<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import Titlebar from '$lib/components/Titlebar.svelte';
  import Timer from '$lib/components/Timer.svelte';
  import StatsView from '$lib/components/views/StatsView.svelte';
  import SettingsView from '$lib/components/views/SettingsView.svelte';
  import { getSettings, getThemes, onSettingsChanged, onThemesChanged, resizeMainWindow } from '$lib/ipc';
  import { settings } from '$lib/stores/settings';
  import { applyTheme } from '$lib/stores/theme';
  import { resolveThemeName } from '$lib/utils/theme';
  import { isMac } from '$lib/utils/platform';
  import { setLocale } from '$lib/locale.svelte.js';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { LogicalSize } from '@tauri-apps/api/dpi';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { info, error as logError } from '@tauri-apps/plugin-log';
  import { createLocalShortcutHandler } from '$lib/utils/localShortcuts';

  // Navigation state & view-specific natural dimensions
  type View = 'timer' | 'stats' | 'settings';
  let currentView = $state<View>('timer');

  // One convenient, unified size for both Settings and Statistics
  const EXTENDED_VIEW_SIZE = { width: 750, height: 520 };

  const VIEW_DIMENSIONS: Record<View, { width: number; height: number }> = {
    timer: { width: 360, height: 478 },
    settings: EXTENDED_VIEW_SIZE,
    stats: EXTENDED_VIEW_SIZE,
  };

  let userTimerSize = $state<{ width: number; height: number } | null>(null);

  /** Automatically resize window to fit the contents of the target view */
  async function switchView(target: View) {
    if (target === currentView) return;
    const win = getCurrentWebviewWindow();

    // Preserve user's custom timer size if navigating away from the timer
    if (currentView === 'timer') {
      const currentW = Math.round(window.outerWidth || window.innerWidth);
      const currentH = Math.round(window.outerHeight || window.innerHeight);
      if (currentW > 0 && currentH > 0) {
        userTimerSize = { width: currentW, height: currentH };
      }
    }

    // Determine the ideal dimension for the destination view
    let targetDimensions = VIEW_DIMENSIONS[target];
    if (target === 'timer' && userTimerSize) {
      targetDimensions = userTimerSize;
    }

    // Trigger native window resize directly
    try {
      await resizeMainWindow(targetDimensions.width, targetDimensions.height);
      await win.setSize(new LogicalSize(targetDimensions.width, targetDimensions.height));
    } catch (e) {
      console.warn('[window] resize failed:', e);
    }

    currentView = target;
  }

  // Local shortcut state
  let localVolume = $state(1.0);
  let preMuteVolume = $state(0.5);
  let isFullscreen = $state(false);

  // Base window dimensions
  const BASE_W = 360;
  const BASE_H = 478;
  const TITLEBAR_H = 42;
  const COMPACT_THRESHOLD = 300;

  let uiScale = $state(1.0);
  let isCompact = $state(false);
  const COMPACT_BOTTOM_PAD = 48;

  $effect(() => {
    function update() {
      const w = window.innerWidth;
      const h = window.innerHeight;
      isCompact = w < COMPACT_THRESHOLD || h < COMPACT_THRESHOLD;
      if (isCompact) {
        const available = Math.min(w - 16, h - TITLEBAR_H - 16 - COMPACT_BOTTOM_PAD);
        uiScale = Math.max(0.4, Math.min(available / 220, 4));
      } else {
        uiScale = Math.max(0.5, Math.min(w / BASE_W, (h - TITLEBAR_H) / (BASE_H - TITLEBAR_H), 4));
      }
    }
    update();
    window.addEventListener('resize', update);
    return () => window.removeEventListener('resize', update);
  });

  async function startResize(direction: string) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    await getCurrentWebviewWindow().startResizeDragging(direction as any);
  }

  onMount(() => {
    const cleanups: UnlistenFn[] = [];

    // Mount local keyboard shortcut handler
    const shortcutHandler = createLocalShortcutHandler({
      getSettings: () => $settings,
      getVolume: () => localVolume,
      setVolume: (v) => {
        localVolume = v;
      },
      getPreMuteVolume: () => preMuteVolume,
      setPreMuteVolume: (v) => {
        preMuteVolume = v;
      },
      getFullscreen: () => isFullscreen,
      setFullscreen: (v) => {
        isFullscreen = v;
      },
    });

    const keyListener = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && currentView !== 'timer') {
        switchView('timer');
        return;
      }
      shortcutHandler(e);
    };

    document.addEventListener('keydown', keyListener);
    cleanups.push(() => document.removeEventListener('keydown', keyListener));

    (async () => {
      try {
        const s = await getSettings();
        settings.set(s);
        localVolume = s.volume;

        setLocale(s.language);
        await info(`[main] settings loaded, locale=${s.language}`);

        const themes = await getThemes();
        const osDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        const active = themes.find((t) => t.name === resolveThemeName(s, osDark)) ?? themes[0];
        if (active) applyTheme(active);
        await getCurrentWebviewWindow().show();
        await info(`[main] initialized, theme=${active?.name ?? 'none'}`);
      } catch (e) {
        await logError(`[main] initialization failed: ${e}`);
        throw e;
      }

      const mq = window.matchMedia('(prefers-color-scheme: dark)');
      const mqListener = async (e: MediaQueryListEvent) => {
        if ($settings.theme_mode !== 'auto') return;
        const allThemes = await getThemes();
        const t = allThemes.find((th) => th.name === resolveThemeName($settings, e.matches));
        if (t) applyTheme(t);
      };
      mq.addEventListener('change', mqListener);
      cleanups.push(() => mq.removeEventListener('change', mqListener));

      cleanups.push(
        await onSettingsChanged(async (updated) => {
          const prevMode = $settings.theme_mode;
          const prevLight = $settings.theme_light;
          const prevDark = $settings.theme_dark;
          const prevLanguage = $settings.language;
          settings.set(updated);
          localVolume = updated.volume;
          if (updated.language !== prevLanguage) {
            setLocale(updated.language);
          }
          if (
            updated.theme_mode !== prevMode ||
            updated.theme_light !== prevLight ||
            updated.theme_dark !== prevDark
          ) {
            const allThemes = await getThemes();
            const dark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            const t = allThemes.find((th) => th.name === resolveThemeName(updated, dark));
            if (t) applyTheme(t);
          }
        }),
        await onThemesChanged((updated) => {
          const dark = window.matchMedia('(prefers-color-scheme: dark)').matches;
          const current =
            updated.find((t) => t.name === resolveThemeName($settings, dark)) ?? updated[0];
          if (current) applyTheme(current);
        })
      );
    })();

    return () => {
      for (const fn of cleanups) fn();
    };
  });
</script>

{#if !isMac}
  <div class="rh rh-n" onmousedown={() => startResize('North')} role="none"></div>
  <div class="rh rh-s" onmousedown={() => startResize('South')} role="none"></div>
  <div class="rh rh-e" onmousedown={() => startResize('East')} role="none"></div>
  <div class="rh rh-w" onmousedown={() => startResize('West')} role="none"></div>
  <div class="rh rh-ne" onmousedown={() => startResize('NorthEast')} role="none"></div>
  <div class="rh rh-nw" onmousedown={() => startResize('NorthWest')} role="none"></div>
  <div class="rh rh-se" onmousedown={() => startResize('SouthEast')} role="none"></div>
  <div class="rh rh-sw" onmousedown={() => startResize('SouthWest')} role="none"></div>
{/if}

<div class="app">
  <Titlebar {currentView} onnavigate={switchView} />

  <!-- Single-Window Stage with Smooth Page Transitions -->
  <div class="stage">
    {#if currentView === 'timer'}
      <main class="view timer-view" class:compact={isCompact}>
        <Timer {isCompact} {uiScale} />
      </main>
    {:else if currentView === 'stats'}
      <div class="view sub-view">
        <StatsView />
      </div>
    {:else if currentView === 'settings'}
      <div class="view sub-view">
        <SettingsView />
      </div>
    {/if}
  </div>
</div>

<style>
  .app {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--color-theme-glass, rgba(21, 24, 34, 0.76));
    backdrop-filter: blur(50px) saturate(200%);
    -webkit-backdrop-filter: blur(50px) saturate(200%);
    border: none;
    box-shadow: none;
    animation: app-fade-in 0.35s var(--transition-default) both;
  }

  .stage {
    flex: 1;
    position: relative;
    overflow: hidden;
    width: 100%;
    height: 100%;
    display: flex;
  }

  .view {
    flex: 1;
    width: 100%;
    height: 100%;
    overflow: hidden;
    animation: view-slide-fade 0.28s cubic-bezier(0.16, 1, 0.3, 1) both;
  }

  @keyframes view-slide-fade {
    from {
      opacity: 0;
      transform: scale(0.985) translateY(4px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .timer-view {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .timer-view.compact {
    padding-bottom: 8px;
  }

  .sub-view {
    display: flex;
    flex-direction: column;
  }

  :global(.rh) {
    position: fixed;
    z-index: 9999;
  }

  :global(.rh-n) {
    top: 0;
    left: 6px;
    right: 6px;
    height: 5px;
    cursor: n-resize;
  }
  :global(.rh-s) {
    bottom: 0;
    left: 6px;
    right: 6px;
    height: 5px;
    cursor: s-resize;
  }
  :global(.rh-e) {
    right: 0;
    top: 6px;
    bottom: 6px;
    width: 5px;
    cursor: e-resize;
  }
  :global(.rh-w) {
    left: 0;
    top: 6px;
    bottom: 6px;
    width: 5px;
    cursor: w-resize;
  }
  :global(.rh-ne) {
    top: 0;
    right: 0;
    width: 10px;
    height: 10px;
    cursor: ne-resize;
  }
  :global(.rh-nw) {
    top: 0;
    left: 0;
    width: 10px;
    height: 10px;
    cursor: nw-resize;
  }
  :global(.rh-se) {
    bottom: 0;
    right: 0;
    width: 10px;
    height: 10px;
    cursor: se-resize;
  }
  :global(.rh-sw) {
    bottom: 0;
    left: 0;
    width: 10px;
    height: 10px;
    cursor: sw-resize;
  }
</style>
