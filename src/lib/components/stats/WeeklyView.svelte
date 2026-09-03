<script lang="ts">
  import type { DayStat, StreakInfo } from '$lib/types';
  import * as m from '$paraglide/messages.js';
  import { getLocale } from '$paraglide/runtime.js';

  let { week, streak }: { week: DayStat[] | null; streak: StreakInfo | null } = $props();

  const CHART_H = 140;
  const BAR_W = 46;
  const BAR_GAP = 14;
  const CHART_W = 7 * (BAR_W + BAR_GAP) - BAR_GAP; // ~406px

  const shortFmt = $derived(new Intl.DateTimeFormat(getLocale(), { weekday: 'short' }));
  const narrowFmt = $derived(new Intl.DateTimeFormat(getLocale(), { weekday: 'narrow' }));

  const days = $derived.by(() => {
    const countByDate = new Map((week ?? []).map((d) => [d.date, d.rounds]));

    const today = new Date();
    today.setHours(0, 0, 0, 0);

    return Array.from({ length: 7 }, (_, i) => {
      const d = new Date(today);
      d.setDate(today.getDate() - (6 - i));
      const dateStr = [
        d.getFullYear(),
        String(d.getMonth() + 1).padStart(2, '0'),
        String(d.getDate()).padStart(2, '0'),
      ].join('-');
      return {
        date: dateStr,
        label: shortFmt.format(d),
        short: narrowFmt.format(d),
        rounds: countByDate.get(dateStr) ?? 0,
        isToday: i === 6,
      };
    });
  });

  const maxRounds = $derived(Math.max(1, ...days.map((d) => d.rounds)));
  const totalWeek = $derived(days.reduce((s, d) => s + d.rounds, 0));
  const hasData = $derived(totalWeek > 0);
</script>

<div class="view">
  <!-- Inset Frosted Stat Cards Row -->
  <div class="cards-grid">
    <div class="stat-card">
      <div class="card-header">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="card-icon">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 6v6l4 2"/>
        </svg>
        <span class="card-label">This Week's Rounds</span>
      </div>
      <span class="card-value">{totalWeek}</span>
    </div>

    <div class="stat-card">
      <div class="card-header">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="card-icon">
          <path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"/>
        </svg>
        <span class="card-label">Focus Streak</span>
      </div>
      <div class="card-streak-val">
        {#if streak && streak.current > 0}
          <span class="card-value">{streak.current}</span>
          <span class="card-unit">{streak.current === 1 ? 'day' : 'days'}</span>
        {:else}
          <span class="card-value-muted">No active streak</span>
        {/if}
      </div>
    </div>
  </div>

  <!-- Weekly Activity Chart Card -->
  <div class="chart-card">
    <div class="section-header">
      <span class="section-title">Weekly Focus Activity</span>
      {#if !hasData}
        <span class="empty-hint">No sessions completed this week</span>
      {/if}
    </div>

    <div class="chart-wrap">
      <svg
        width="100%" style="max-width: {CHART_W}px"
        height={CHART_H + 34}
        viewBox="0 0 {CHART_W} {CHART_H + 34}"
        class="chart"
      >
        {#each days as day, i}
          {@const barH = Math.max(day.rounds > 0 ? 6 : 2, Math.round((day.rounds / maxRounds) * CHART_H))}
          {@const x = i * (BAR_W + BAR_GAP)}
          {@const y = CHART_H - barH}

          <!-- Bar -->
          <rect
            {x}
            {y}
            width={BAR_W}
            height={barH}
            rx="4"
            class="bar"
            class:bar-today={day.isToday}
            class:bar-empty={day.rounds === 0}
            style="--bar-delay: {i * 35}ms"
          />

          <!-- Round count label -->
          {#if day.rounds > 0}
            <text x={x + BAR_W / 2} y={y - 6} text-anchor="middle" class="count-label">
              {day.rounds}
            </text>
          {/if}

          <!-- Day label -->
          <text
            x={x + BAR_W / 2}
            y={CHART_H + 20}
            text-anchor="middle"
            class="day-label"
            class:day-label-today={day.isToday}
          >
            {day.label}
          </text>
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

  .cards-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
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
    transition: all 0.2s ease;
  }

  .stat-card:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.12);
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

  .card-streak-val {
    display: flex;
    align-items: baseline;
    gap: 6px;
  }

  .card-unit {
    font-size: 0.9rem;
    font-weight: 400;
    color: rgba(255, 255, 255, 0.6);
  }

  .card-value-muted {
    font-size: 1.1rem;
    font-weight: 400;
    color: rgba(255, 255, 255, 0.35);
    padding-top: 6px;
  }

  .chart-card {
    display: flex;
    flex-direction: column;
    padding: 20px 24px 18px;
    gap: 16px;
    background: rgba(255, 255, 255, 0.035);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 12px;
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
    display: flex;
    justify-content: center;
    overflow-x: auto;
  }

  .bar {
    fill: var(--color-focus-round);
    opacity: 0.85;
    transition: all 0.15s ease;
  }

  .bar-today {
    fill: color-mix(in srgb, var(--color-focus-round) 85%, white 15%);
    filter: drop-shadow(0 0 6px color-mix(in oklch, var(--color-focus-round) 30%, transparent));
  }

  .bar-empty {
    fill: rgba(255, 255, 255, 0.06);
  }

  .count-label {
    fill: rgba(255, 255, 255, 0.75);
    font-size: 11px;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }

  .day-label {
    fill: rgba(255, 255, 255, 0.45);
    font-size: 10.5px;
    font-weight: 500;
  }

  .day-label-today {
    fill: #ffffff;
    font-weight: 600;
  }

  .baseline {
    stroke: rgba(255, 255, 255, 0.06);
    stroke-width: 1;
  }
</style>
