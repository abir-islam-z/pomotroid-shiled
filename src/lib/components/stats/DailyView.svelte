<script lang="ts">
  import type { DailyStats } from '$lib/types';
  import * as m from '$paraglide/messages.js';

  let { today }: { today: DailyStats | null } = $props();

  const CHART_H = 86; // px, max bar height in the hourly chart
  const CHART_W = 744; // px, total SVG width for 24 bars
  const BAR_W = 22; // px per bar
  const BAR_GAP = 9; // px between bars

  function fmtTime(mins: number): string {
    if (mins < 60) return `${mins}m`;
    const h = Math.floor(mins / 60);
    const m = mins % 60;
    return m === 0 ? `${h}h` : `${h}h ${m}m`;
  }

  function fmtRate(rate: number | null): string {
    if (rate === null) return '—';
    return `${Math.round(rate * 100)}%`;
  }

  const byHour = $derived(today?.by_hour ?? Array(24).fill(0));
  const maxHour = $derived(Math.max(1, ...byHour));
  const hasData = $derived(today !== null && today.rounds > 0);

  const hourLabels = [0, 6, 12, 18];
</script>

<div class="view">
  <!-- 3 Floating Inset Glass Stat Cards -->
  <div class="cards-grid">
    <!-- Rounds -->
    <div class="stat-card" style="--delay: 0ms">
      <div class="card-header">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="card-icon">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 6v6l4 2"/>
        </svg>
        <span class="card-label">Rounds</span>
      </div>
      <span class="card-value">{today?.rounds ?? '0'}</span>
    </div>

    <!-- Focus Time -->
    <div class="stat-card" style="--delay: 50ms">
      <div class="card-header">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="card-icon">
          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
        </svg>
        <span class="card-label">Focus Time</span>
      </div>
      <span class="card-value">{today ? fmtTime(today.focus_mins) : '0m'}</span>
    </div>

    <!-- Completion Rate -->
    <div class="stat-card" style="--delay: 100ms">
      <div class="card-header">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="card-icon">
          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
          <polyline points="22 4 12 14.01 9 11.01"/>
        </svg>
        <span class="card-label">Completion</span>
      </div>
      <span class="card-value">{today ? fmtRate(today.completion_rate) : '—'}</span>
    </div>
  </div>

  <!-- Activity by Hour Card -->
  <div class="chart-card">
    <div class="section-header">
      <span class="section-title">Activity by Hour</span>
      {#if !hasData}
        <span class="empty-hint">No focus sessions recorded today</span>
      {/if}
    </div>

    <div class="chart-wrap">
      <svg
        width="100%"
        height={CHART_H + 26}
        viewBox="0 0 {CHART_W} {CHART_H + 26}"
        class="chart"
      >
        {#each byHour as count, h}
          {@const barH = Math.max(count > 0 ? 4 : 2, Math.round((count / maxHour) * CHART_H))}
          {@const x = h * (BAR_W + BAR_GAP)}
          {@const y = CHART_H - barH}

          <!-- Bar -->
          <rect
            {x}
            {y}
            width={BAR_W}
            height={barH}
            rx="3"
            class="bar"
            class:bar-empty={count === 0}
            style="--bar-delay: {h * 15}ms"
          />

          <!-- Hour label -->
          {#if hourLabels.includes(h)}
            <text x={x + BAR_W / 2} y={CHART_H + 18} text-anchor="middle" class="hour-label">
              {h === 0 ? '12 AM' : h === 12 ? '12 PM' : h < 12 ? `${h} AM` : `${h - 12} PM`}
            </text>
          {/if}
        {/each}

        <!-- Baseline -->
        <line x1="0" y1={CHART_H} x2={CHART_W} y2={CHART_H} class="baseline" />
      </svg>
    </div>
  </div>
</div>

<style>
  .view {
    display: flex;
    flex-direction: column;
    gap: 16px;
    animation: app-fade-in 0.2s ease;
  }

  /* ── 3 Inset Frosted Stat Cards ──────────────────────────────── */
  .cards-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }

  .stat-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 16px 20px 18px;
    background: rgba(255, 255, 255, 0.035);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 12px;
    animation: card-rise 0.3s cubic-bezier(0.22, 1, 0.36, 1) both;
    animation-delay: var(--delay, 0ms);
    transition: all 0.2s ease;
  }

  .stat-card:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.12);
  }

  @keyframes card-rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 7px;
  }

  .card-icon {
    color: rgba(255, 255, 255, 0.5);
  }

  .card-label {
    font-size: 0.74rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.55);
    letter-spacing: -0.01em;
  }

  .card-value {
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Rounded", system-ui, sans-serif;
    font-size: 2.15rem;
    font-weight: 250;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.03em;
    color: #ffffff;
    line-height: 1;
  }

  /* ── Activity Chart Card ─────────────────────────────────────── */
  .chart-card {
    display: flex;
    flex-direction: column;
    padding: 18px 22px 16px;
    gap: 14px;
    background: rgba(255, 255, 255, 0.035);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 12px;
    overflow: hidden;
  }

  .section-header {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }

  .section-title {
    font-size: 0.76rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.7);
    letter-spacing: -0.01em;
  }

  .empty-hint {
    font-size: 0.72rem;
    color: rgba(255, 255, 255, 0.35);
    font-style: italic;
  }

  .chart-wrap {
    overflow-x: auto;
    overflow-y: hidden;
  }

  .chart {
    display: block;
  }

  .bar {
    fill: var(--color-focus-round);
    opacity: 0.85;
    transform-origin: bottom;
    transform-box: fill-box;
    animation: bar-rise 0.35s cubic-bezier(0.22, 1, 0.36, 1) both;
    animation-delay: var(--bar-delay, 0ms);
    transition: opacity 0.15s ease;
  }

  .bar:hover {
    opacity: 1;
    fill: color-mix(in srgb, var(--color-focus-round) 85%, white 15%);
  }

  @keyframes bar-rise {
    from {
      transform: scaleY(0.05);
      opacity: 0;
    }
    to {
      transform: scaleY(1);
      opacity: 0.85;
    }
  }

  .bar-empty {
    fill: rgba(255, 255, 255, 0.06);
    animation: none;
  }

  .hour-label {
    fill: rgba(255, 255, 255, 0.45);
    font-size: 9.5px;
    font-weight: 500;
    letter-spacing: 0.02em;
    cursor: default;
  }

  .baseline {
    stroke: rgba(255, 255, 255, 0.06);
    stroke-width: 1;
  }
</style>
