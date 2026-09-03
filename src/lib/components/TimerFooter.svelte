<script lang="ts">
  // Round counter, reset button, and volume slider wrapped in a macOS glass pill.
  import type { TimerState } from '$lib/types';
  import { timerReset, setSetting } from '$lib/ipc';
  import { settings } from '$lib/stores/settings';
  import * as m from '$paraglide/messages.js';
  import Tooltip from './Tooltip.svelte';

  interface Props {
    snap: TimerState;
  }

  let { snap }: Props = $props();

  let showVolume = $state(false);

  let localVolume = $state($settings.volume);
  $effect(() => {
    localVolume = $settings.volume;
  });

  let premuteVolume = $state<number | null>(null);

  function handleVolumeChange(e: Event) {
    const val = (e.target as HTMLInputElement).valueAsNumber;
    localVolume = val;
    setSetting('volume', String(Math.round(val * 100)));
  }

  function toggleMute() {
    if (localVolume === 0) {
      const restore = premuteVolume ?? 1.0;
      premuteVolume = null;
      localVolume = restore;
      setSetting('volume', String(Math.round(restore * 100)));
    } else {
      premuteVolume = localVolume;
      localVolume = 0;
      setSetting('volume', '0');
    }
  }
</script>

<!-- Round counter -->
<Tooltip
  text={$settings.long_breaks_enabled
    ? m.tooltip_round_counter()
    : m.tooltip_round_counter_session()}
>
  <span class="rounds">
    {#if $settings.long_breaks_enabled}
      {snap.work_round_number} &nbsp;/&nbsp; {snap.work_rounds_total}
    {:else}
      {m.timer_session_round({ n: snap.session_work_count })}
    {/if}
  </span>
</Tooltip>

<!-- Reset -->
<Tooltip text={m.tooltip_reset()}>
  <button class="btn-text" onclick={timerReset} aria-label={m.timer_reset()}>
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 3px;">
      <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
      <path d="M3 3v5h5"/>
    </svg>
    {m.timer_reset()}
  </button>
</Tooltip>

<!-- Volume -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="volume-wrapper">
  <Tooltip text={localVolume === 0 ? m.tooltip_unmute() : m.tooltip_mute()}>
    <button
      class="btn-icon"
      onclick={toggleMute}
      aria-label={localVolume === 0 ? 'Unmute' : 'Mute'}
      onmouseenter={() => (showVolume = true)}
      onmouseleave={() => (showVolume = false)}
    >
      {#if localVolume === 0}
        <!-- Mute Icon -->
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/>
          <line x1="23" y1="9" x2="17" y2="15"/>
          <line x1="17" y1="9" x2="23" y2="15"/>
        </svg>
      {:else}
        <!-- Speaker Icon -->
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/>
          <path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>
          <path d="M19.07 4.93a10 10 0 0 1 0 14.14"/>
        </svg>
      {/if}
    </button>
  </Tooltip>

  {#if showVolume}
    <div class="volume-slider-wrapper">
      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={localVolume}
        oninput={handleVolumeChange}
        class="volume-slider"
        aria-label="Volume"
      />
    </div>
  {/if}
</div>

<style>
  .rounds {
    font-size: 0.76rem;
    font-weight: 500;
    color: var(--color-foreground-darker);
    min-width: 44px;
    text-align: center;
    cursor: default;
    letter-spacing: 0.04em;
  }

  .btn-text {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-foreground-darker);
    font-size: 0.76rem;
    font-weight: 500;
    padding: 3px 8px;
    border-radius: 6px;
    display: inline-flex;
    align-items: center;
    transition: all 0.15s ease;
  }

  .btn-text:hover {
    color: var(--color-foreground);
    background: rgba(255, 255, 255, 0.08);
  }

  .btn-icon {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-foreground-darker);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    transition: all 0.15s ease;
  }

  .btn-icon:hover {
    color: var(--color-foreground);
    background: rgba(255, 255, 255, 0.08);
  }

  .volume-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .volume-slider-wrapper {
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    padding: 10px 8px;
    background: rgba(26, 30, 42, 0.92);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
    width: 38px;
    height: 110px;
    margin-bottom: 6px;
  }

  .volume-slider {
    width: 86px;
    transform: rotate(-90deg);
    cursor: pointer;
    accent-color: var(--color-accent);
  }
</style>
