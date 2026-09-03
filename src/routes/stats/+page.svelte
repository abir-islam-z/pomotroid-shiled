<script lang="ts">
  import '../../app.css';
  import { onMount } from 'svelte';
  import {
    getSettings,
    getThemes,
    onSettingsChanged,
    onThemesChanged,
    onRoundChange,
    onSessionsCleared,
    statsGetDetailed,
    statsGetHeatmap,
  } from '$lib/ipc';
  import { settings } from '$lib/stores/settings';
  import { applyTheme } from '$lib/stores/theme';
  import { setLocale } from '$lib/locale.svelte.js';
  import { resolveThemeName } from '$lib/utils/theme';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { isMac } from '$lib/utils/platform';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import type { DetailedStats, HeatmapStats } from '$lib/types';
  import * as m from '$paraglide/messages.js';
  import { info, error as logError } from '@tauri-apps/plugin-log';

  import DailyView from '$lib/components/stats/DailyView.svelte';
  import WeeklyView from '$lib/components/stats/WeeklyView.svelte';
  import YearlyView from '$lib/components/stats/YearlyView.svelte';

  type Tab = 'today' | 'week' | 'alltime';

  let activeTab = $state<Tab>('today');
  let detailed = $state<DetailedStats | null>(null);
  let heatmap = $state<HeatmapStats | null>(null);
  let heatmapLoaded = $state(false);

  async function switchTab(tab: Tab) {
    activeTab = tab;
    if (tab === 'alltime' && !heatmapLoaded) {
      try {
        heatmap = await statsGetHeatmap();
        heatmapLoaded = true;
      } catch (e) {
        await logError(`[stats] failed to load heatmap: ${e}`);
      }
    }
  }

  function close() {
    getCurrentWebviewWindow().close();
  }

  onMount(() => {
    const cleanups: UnlistenFn[] = [];

    (async () => {
      try {
        const s = await getSettings();
        settings.set(s);
        setLocale(s.language);
        await info(`[stats] settings loaded, locale=${s.language}`);

        const themes = await getThemes();
        const osDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        const activeTheme = themes.find((t) => t.name === resolveThemeName(s, osDark)) ?? themes[0];
        if (activeTheme) applyTheme(activeTheme);

        await getCurrentWebviewWindow().show();

        detailed = await statsGetDetailed();
        await info(`[stats] initialized, theme=${activeTheme?.name ?? 'none'}`);
      } catch (e) {
        await logError(`[stats] initialization failed: ${e}`);
        throw e;
      }

      cleanups.push(
        await onRoundChange(async () => {
          try {
            detailed = await statsGetDetailed();
            if (heatmapLoaded) heatmap = await statsGetHeatmap();
          } catch (e) {
            await logError(`[stats] failed to refresh stats after round change: ${e}`);
          }
        }),
        await onSessionsCleared(async () => {
          try {
            detailed = await statsGetDetailed();
            if (heatmapLoaded) heatmap = await statsGetHeatmap();
          } catch (e) {
            await logError(`[stats] failed to refresh stats after session clear: ${e}`);
          }
        }),
        await onSettingsChanged(async (updated) => {
          const prev = {
            mode: $settings.theme_mode,
            light: $settings.theme_light,
            dark: $settings.theme_dark,
            language: $settings.language,
          };
          settings.set(updated);
          if (updated.language !== prev.language) {
            setLocale(updated.language);
          }
          if (
            updated.theme_mode !== prev.mode ||
            updated.theme_light !== prev.light ||
            updated.theme_dark !== prev.dark
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

<div class="window">
  <!-- Clean macOS Titlebar with Segmented Control -->
  <nav class="titlebar" class:macos={isMac} data-tauri-drag-region>
    <div class="titlebar-left">
      <span class="titlebar-label">Statistics</span>
    </div>

    <!-- Apple Native Segmented Control for Timeframe -->
    <div class="timeframe-selector" data-tauri-drag-region="false">
      <button
        class="segment-btn"
        class:active={activeTab === 'today'}
        onclick={() => switchTab('today')}
      >
        Today
      </button>
      <button
        class="segment-btn"
        class:active={activeTab === 'week'}
        onclick={() => switchTab('week')}
      >
        This Week
      </button>
      <button
        class="segment-btn"
        class:active={activeTab === 'alltime'}
        onclick={() => switchTab('alltime')}
      >
        All Time
      </button>
    </div>

    <div class="titlebar-right">
      {#if !isMac}
        <button class="btn-close" onclick={close} aria-label="Close">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <line x1="1" y1="1" x2="11" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="11" y1="1" x2="1" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      {/if}
    </div>
  </nav>

  <!-- Content Container -->
  <main class="content">
    {#if activeTab === 'today'}
      <DailyView today={detailed?.today ?? null} />
    {:else if activeTab === 'week'}
      <WeeklyView week={detailed?.week ?? null} streak={detailed?.streak ?? null} />
    {:else}
      <YearlyView {heatmap} />
    {/if}
  </main>
</div>

<style>
  .window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: rgba(22, 25, 36, 0.88);
    backdrop-filter: blur(48px) saturate(180%);
    -webkit-backdrop-filter: blur(48px) saturate(180%);
    color: var(--color-foreground);
    animation: app-fade-in 0.25s var(--transition-default);
    overflow: hidden;
    cursor: default;
    border: none;
    box-shadow: none;
  }

  /* ── Titlebar with Inset Controls ──────────────────────────────── */
  .titlebar {
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    position: relative;
    flex-shrink: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .macos {
    padding-left: 78px;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    min-width: 100px;
  }

  .titlebar-label {
    font-size: 0.82rem;
    font-weight: 500;
    letter-spacing: -0.01em;
    color: rgba(255, 255, 255, 0.7);
    pointer-events: none;
  }

  .titlebar-right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    min-width: 100px;
  }

  /* ── Apple Native Segmented Control ─────────────────────────────── */
  .timeframe-selector {
    display: flex;
    background: rgba(0, 0, 0, 0.28);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 2px;
    gap: 2px;
  }

  .segment-btn {
    padding: 4px 16px;
    background: transparent;
    border: none;
    border-radius: 6px;
    font-size: 0.76rem;
    font-weight: 400;
    color: rgba(255, 255, 255, 0.65);
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
  }

  .segment-btn:hover {
    color: rgba(255, 255, 255, 0.9);
  }

  .segment-btn.active {
    background: rgba(255, 255, 255, 0.14);
    color: #ffffff;
    font-weight: 500;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }

  .btn-close {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-foreground-darker);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 4px;
    transition: all 0.15s;
  }

  .btn-close:hover {
    color: #ffffff;
    background: #FF453A;
  }

  /* ── Content ─────────────────────────────────────────────── */
  .content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 20px 24px 24px;
  }
</style>
