<script lang="ts">
  import { settings } from '$lib/stores/settings';
  import { setSetting } from '$lib/ipc';
  import SettingsToggle from '$lib/components/settings/SettingsToggle.svelte';
  import * as m from '$paraglide/messages.js';

  const MIN_SECS = 60; // 1:00
  const MAX_SECS = 5400; // 90:00
  const MAX_ROUNDS = 12;

  let workMins = $derived(Math.round($settings.time_work_secs / 60));
  let shortMins = $derived(Math.round($settings.time_short_break_secs / 60));
  let longMins = $derived(Math.round($settings.time_long_break_secs / 60));
  let rounds = $derived($settings.long_break_interval);

  let workEdit = $state<string | null>(null);
  let shortEdit = $state<string | null>(null);
  let longEdit = $state<string | null>(null);

  function parseMMSS(input: string): number | null {
    const trimmed = input.trim();
    const colonIdx = trimmed.indexOf(':');
    if (colonIdx === -1) {
      const mins = parseInt(trimmed, 10);
      if (isNaN(mins) || trimmed === '') return null;
      return mins * 60;
    }
    const mm = parseInt(trimmed.slice(0, colonIdx), 10);
    const ss = parseInt(trimmed.slice(colonIdx + 1), 10);
    if (isNaN(mm) || isNaN(ss) || ss < 0 || ss > 59) return null;
    return mm * 60 + ss;
  }

  function formatMMSS(totalSecs: number): string {
    const mins = Math.floor(totalSecs / 60);
    const secs = totalSecs % 60;
    return `${mins}:${String(secs).padStart(2, '0')}`;
  }

  function barWidth(val: number, min: number, max: number): string {
    const frac = (val - min) / (max - min);
    return `calc(${frac} * (100% - 14px) + 7px)`;
  }

  async function handleChange(dbKey: string, rawValue: number) {
    const updated = await setSetting(dbKey, String(rawValue));
    settings.set(updated);
  }

  async function toggle(dbKey: string, current: boolean) {
    const updated = await setSetting(dbKey, current ? 'false' : 'true');
    settings.set(updated);
  }

  async function commitBadge(
    raw: string | null,
    currentSecs: number,
    dbKey: string,
    el: HTMLInputElement
  ): Promise<void> {
    if (raw === null) {
      el.value = formatMMSS(currentSecs);
      return;
    }
    const parsed = parseMMSS(raw);
    if (parsed === null) {
      el.value = formatMMSS(currentSecs);
      return;
    }
    const clamped = Math.max(MIN_SECS, Math.min(MAX_SECS, parsed));
    await handleChange(dbKey, clamped);
    el.value = formatMMSS(clamped);
  }
</script>

<div class="section">
  <!-- Work Session -->
  <div class="group-heading">Focus Duration</div>
  <div class="group-card">
    <div class="slider-row">
      <div class="slider-meta">
        <span class="slider-label">{m.timer_slider_focus()}</span>
        <input
          class="slider-value"
          type="text"
          value={workEdit ?? formatMMSS($settings.time_work_secs)}
          onfocus={(e) => {
            workEdit = (e.target as HTMLInputElement).value;
            (e.target as HTMLInputElement).select();
          }}
          oninput={(e) => {
            workEdit = (e.target as HTMLInputElement).value;
          }}
          onblur={async (e) => {
            await commitBadge(
              workEdit,
              $settings.time_work_secs,
              'time_work_secs',
              e.target as HTMLInputElement
            );
            workEdit = null;
          }}
          onkeydown={async (e) => {
            if (e.key === 'Enter') {
              await commitBadge(
                workEdit,
                $settings.time_work_secs,
                'time_work_secs',
                e.target as HTMLInputElement
              );
              workEdit = null;
              (e.target as HTMLInputElement).blur();
            } else if (e.key === 'Escape') {
              workEdit = null;
              (e.target as HTMLInputElement).value = formatMMSS($settings.time_work_secs);
              (e.target as HTMLInputElement).blur();
            }
          }}
        />
      </div>
      <div class="slider-wrap">
        <input
          type="range"
          min="1"
          max="90"
          step="1"
          value={workMins}
          class="slider"
          oninput={(e) =>
            handleChange('time_work_secs', (e.target as HTMLInputElement).valueAsNumber * 60)}
        />
        <div class="bar bar--focus" style="width: {barWidth(workMins, 1, 90)}"></div>
      </div>
    </div>
  </div>

  <!-- Short Break -->
  <div class="group-heading">Short Breaks</div>
  <div class="group-card">
    <SettingsToggle
      label={m.timer_toggle_short_breaks()}
      description={m.timer_toggle_short_breaks_desc()}
      checked={!$settings.short_breaks_enabled}
      onclick={() => toggle('short_breaks_enabled', $settings.short_breaks_enabled)}
    />
    <div class="break-body" class:disabled={!$settings.short_breaks_enabled}>
      <div class="slider-row">
        <div class="slider-meta">
          <span class="slider-label">{m.timer_slider_short_break()}</span>
          <input
            class="slider-value"
            type="text"
            value={shortEdit ?? formatMMSS($settings.time_short_break_secs)}
            onfocus={(e) => {
              shortEdit = (e.target as HTMLInputElement).value;
              (e.target as HTMLInputElement).select();
            }}
            oninput={(e) => {
              shortEdit = (e.target as HTMLInputElement).value;
            }}
            onblur={async (e) => {
              await commitBadge(
                shortEdit,
                $settings.time_short_break_secs,
                'time_short_break_secs',
                e.target as HTMLInputElement
              );
              shortEdit = null;
            }}
            onkeydown={async (e) => {
              if (e.key === 'Enter') {
                await commitBadge(
                  shortEdit,
                  $settings.time_short_break_secs,
                  'time_short_break_secs',
                  e.target as HTMLInputElement
                );
                shortEdit = null;
                (e.target as HTMLInputElement).blur();
              } else if (e.key === 'Escape') {
                shortEdit = null;
                (e.target as HTMLInputElement).value = formatMMSS($settings.time_short_break_secs);
                (e.target as HTMLInputElement).blur();
              }
            }}
          />
        </div>
        <div class="slider-wrap">
          <input
            type="range"
            min="1"
            max="90"
            step="1"
            value={shortMins}
            class="slider"
            oninput={(e) =>
              handleChange(
                'time_short_break_secs',
                (e.target as HTMLInputElement).valueAsNumber * 60
              )}
          />
          <div class="bar bar--short" style="width: {barWidth(shortMins, 1, 90)}"></div>
        </div>
      </div>
    </div>
  </div>

  <!-- Long Break -->
  <div class="group-heading">Long Breaks</div>
  <div class="group-card">
    <SettingsToggle
      label={m.timer_toggle_long_breaks()}
      description={m.timer_toggle_long_breaks_desc()}
      checked={!$settings.long_breaks_enabled}
      onclick={() => toggle('long_breaks_enabled', $settings.long_breaks_enabled)}
    />
    <div class="break-body" class:disabled={!$settings.long_breaks_enabled}>
      <div class="slider-row">
        <div class="slider-meta">
          <span class="slider-label">{m.timer_slider_long_break()}</span>
          <input
            class="slider-value"
            type="text"
            value={longEdit ?? formatMMSS($settings.time_long_break_secs)}
            onfocus={(e) => {
              longEdit = (e.target as HTMLInputElement).value;
              (e.target as HTMLInputElement).select();
            }}
            oninput={(e) => {
              longEdit = (e.target as HTMLInputElement).value;
            }}
            onblur={async (e) => {
              await commitBadge(
                longEdit,
                $settings.time_long_break_secs,
                'time_long_break_secs',
                e.target as HTMLInputElement
              );
              longEdit = null;
            }}
            onkeydown={async (e) => {
              if (e.key === 'Enter') {
                await commitBadge(
                  longEdit,
                  $settings.time_long_break_secs,
                  'time_long_break_secs',
                  e.target as HTMLInputElement
                );
                longEdit = null;
                (e.target as HTMLInputElement).blur();
              } else if (e.key === 'Escape') {
                longEdit = null;
                (e.target as HTMLInputElement).value = formatMMSS($settings.time_long_break_secs);
                (e.target as HTMLInputElement).blur();
              }
            }}
          />
        </div>
        <div class="slider-wrap">
          <input
            type="range"
            min="1"
            max="90"
            step="1"
            value={longMins}
            class="slider"
            oninput={(e) =>
              handleChange('time_long_break_secs', (e.target as HTMLInputElement).valueAsNumber * 60)}
          />
          <div class="bar bar--long" style="width: {barWidth(longMins, 1, 90)}"></div>
        </div>
      </div>

      <!-- Rounds -->
      <div class="slider-row">
        <div class="slider-meta">
          <span class="slider-label">{m.timer_slider_rounds()}</span>
          <span class="slider-value slider-value--static">{rounds}</span>
        </div>
        <div class="slider-wrap">
          <input
            type="range"
            min="1"
            max={MAX_ROUNDS}
            step="1"
            value={rounds}
            class="slider"
            oninput={(e) => handleChange('work_rounds', (e.target as HTMLInputElement).valueAsNumber)}
          />
          <div class="bar bar--rounds" style="width: {barWidth(rounds, 1, MAX_ROUNDS)}"></div>
        </div>
      </div>
    </div>
  </div>

  <!-- Automation & Display -->
  <div class="group-heading">Automation & Display</div>
  <div class="group-card">
    <SettingsToggle
      label={m.timer_toggle_auto_start_work()}
      description={m.timer_toggle_auto_start_work_desc()}
      checked={$settings.auto_start_work}
      onclick={() => toggle('auto_start_work', $settings.auto_start_work)}
    />
    <SettingsToggle
      label={m.timer_toggle_auto_start_break()}
      description={m.timer_toggle_auto_start_break_desc()}
      checked={$settings.auto_start_break}
      onclick={() => toggle('auto_start_break', $settings.auto_start_break)}
    />
    <SettingsToggle
      label={m.timer_toggle_countdown()}
      description={m.timer_toggle_countdown_desc()}
      checked={$settings.dial_countdown}
      onclick={() => toggle('dial_countdown', $settings.dial_countdown)}
    />
  </div>
</div>

<style>
  .section {
    display: flex;
    flex-direction: column;
    padding-bottom: 24px;
  }

  .group-heading {
    font-size: 0.74rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.55);
    margin: 16px 20px 6px;
    letter-spacing: -0.01em;
  }

  .group-card {
    margin: 0 20px 4px;
    background: rgba(255, 255, 255, 0.035);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 10px;
    overflow: hidden;
  }

  .slider-row {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .slider-row:last-child {
    border-bottom: none;
  }

  .slider-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .slider-label {
    font-size: 0.82rem;
    font-weight: 400;
    color: rgba(255, 255, 255, 0.85);
  }

  .slider-value {
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Rounded", monospace;
    font-size: 0.78rem;
    font-weight: 500;
    color: #ffffff;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    padding: 2px 8px;
    width: 52px;
    text-align: center;
    outline: none;
    transition: all 0.15s ease;
  }

  .slider-value:focus {
    border-color: rgba(255, 255, 255, 0.3);
    background: rgba(0, 0, 0, 0.35);
  }

  .slider-value--static {
    cursor: default;
    pointer-events: none;
    width: auto;
    min-width: 28px;
  }

  .slider-wrap {
    position: relative;
    height: 18px;
    display: flex;
    align-items: center;
  }

  .slider {
    position: relative;
    z-index: 2;
    width: 100%;
    -webkit-appearance: none;
    appearance: none;
    height: 4px;
    background: rgba(255, 255, 255, 0.12);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #ffffff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
    cursor: pointer;
    transition: transform 0.15s ease;
  }

  .slider::-webkit-slider-thumb:hover {
    transform: scale(1.1);
  }

  .bar {
    position: absolute;
    left: 0;
    height: 4px;
    border-radius: 2px;
    pointer-events: none;
    z-index: 1;
    transition: width 0.05s;
  }

  .bar--focus {
    background: var(--color-focus-round);
  }
  .bar--short {
    background: var(--color-short-round);
  }
  .bar--long {
    background: var(--color-long-round);
  }
  .bar--rounds {
    background: rgba(255, 255, 255, 0.6);
  }

  .break-body {
    transition: opacity 0.15s;
  }
  .break-body.disabled {
    opacity: 0.35;
    pointer-events: none;
  }
</style>
