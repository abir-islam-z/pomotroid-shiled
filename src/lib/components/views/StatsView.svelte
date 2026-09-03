<script lang="ts">
  import { onMount } from 'svelte';
  import { statsGetDetailed, statsGetHeatmap, onRoundChange, onSessionsCleared } from '$lib/ipc';
  import type { DetailedStats, HeatmapStats } from '$lib/types';
  import DailyView from '$lib/components/stats/DailyView.svelte';
  import WeeklyView from '$lib/components/stats/WeeklyView.svelte';
  import YearlyView from '$lib/components/stats/YearlyView.svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { error as logError } from '@tauri-apps/plugin-log';

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

  onMount(() => {
    const cleanups: UnlistenFn[] = [];

    (async () => {
      try {
        detailed = await statsGetDetailed();
      } catch (e) {
        await logError(`[stats] initialization failed: ${e}`);
      }

      cleanups.push(
        await onRoundChange(async () => {
          try {
            detailed = await statsGetDetailed();
            if (heatmapLoaded) heatmap = await statsGetHeatmap();
          } catch (e) {
            await logError(`[stats] failed to refresh after round change: ${e}`);
          }
        }),
        await onSessionsCleared(async () => {
          try {
            detailed = await statsGetDetailed();
            if (heatmapLoaded) heatmap = await statsGetHeatmap();
          } catch (e) {
            await logError(`[stats] failed to refresh after sessions cleared: ${e}`);
          }
        })
      );
    })();

    return () => {
      for (const fn of cleanups) fn();
    };
  });
</script>

<div class="stats-view">
  <!-- Segmented Control Bar -->
  <div class="header-bar">
    <div class="segmented-control">
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
  </div>

  <!-- Scrollable Content -->
  <div class="stats-content">
    {#if activeTab === 'today'}
      <DailyView today={detailed?.today ?? null} />
    {:else if activeTab === 'week'}
      <WeeklyView week={detailed?.week ?? null} streak={detailed?.streak ?? null} />
    {:else}
      <YearlyView {heatmap} />
    {/if}
  </div>
</div>

<style>
  .stats-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    color: var(--color-foreground);
    animation: view-enter 0.25s cubic-bezier(0.16, 1, 0.3, 1) both;
  }

  @keyframes view-enter {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .header-bar {
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 8px 16px 12px;
    flex-shrink: 0;
  }

  .segmented-control {
    display: flex;
    background: rgba(0, 0, 0, 0.28);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 2px;
    gap: 2px;
  }

  .segment-btn {
    padding: 4px 18px;
    background: transparent;
    border: none;
    border-radius: 6px;
    font-size: 0.78rem;
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

  .stats-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 0 20px 20px;
  }
</style>
