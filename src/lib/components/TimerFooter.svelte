<script lang="ts">
  // Round counter, reset button, and native macOS glassmorphic volume control.
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
  let isDragging = $state(false);

  let localVolume = $state($settings.volume);
  $effect(() => {
    localVolume = $settings.volume;
  });

  let premuteVolume = $state<number | null>(null);

  let activeColor = $derived.by(() => {
    if (snap.round_type === 'short-break') return 'var(--color-short-round)';
    if (snap.round_type === 'long-break') return 'var(--color-long-round)';
    return 'var(--color-focus-round)';
  });

  function setVolumeClamped(val: number) {
    const clamped = Math.max(0, Math.min(1, Math.round(val * 100) / 100));
    localVolume = clamped;
    setSetting('volume', String(Math.round(clamped * 100)));
  }

  function toggleMute() {
    if (localVolume === 0) {
      const restore = premuteVolume ?? 1.0;
      premuteVolume = null;
      setVolumeClamped(restore);
    } else {
      premuteVolume = localVolume;
      setVolumeClamped(0);
    }
  }

  function updateVolumeFromY(clientY: number, trackRect: DOMRect) {
    const trackHeight = trackRect.height;
    const offsetY = trackRect.bottom - clientY;
    setVolumeClamped(offsetY / trackHeight);
  }

  function handleTrackPointerDown(e: PointerEvent) {
    const track = e.currentTarget as HTMLElement;
    track.setPointerCapture(e.pointerId);
    isDragging = true;
    const rect = track.getBoundingClientRect();
    updateVolumeFromY(e.clientY, rect);

    const onPointerMove = (ev: PointerEvent) => {
      if (!isDragging) return;
      updateVolumeFromY(ev.clientY, rect);
    };

    const onPointerUp = (ev: PointerEvent) => {
      isDragging = false;
      try {
        track.releasePointerCapture(ev.pointerId);
      } catch {}
      window.removeEventListener('pointermove', onPointerMove);
      window.removeEventListener('pointerup', onPointerUp);
    };

    window.addEventListener('pointermove', onPointerMove);
    window.addEventListener('pointerup', onPointerUp);
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const step = e.deltaY < 0 ? 0.05 : -0.05;
    setVolumeClamped(localVolume + step);
  }
</script>

<div class="timer-footer" style="--active-theme-color: {activeColor};">
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
      {m.timer_reset()}
    </button>
  </Tooltip>

  <!-- Native macOS Glassmorphic Sound Control -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="volume-wrapper"
    onmouseenter={() => (showVolume = true)}
    onmouseleave={() => { if (!isDragging) showVolume = false; }}
    onwheel={handleWheel}
  >
    {#if showVolume || isDragging}
      <div class="volume-popover">
        <!-- Interactive Vertical Pill Track -->
        <div
          class="volume-track"
          role="slider"
          aria-label="Volume"
          aria-valuenow={Math.round(localVolume * 100)}
          aria-valuemin="0"
          aria-valuemax="100"
          tabindex="0"
          onpointerdown={handleTrackPointerDown}
        >
          <div
            class="track-fill"
            style="height: {localVolume * 100}%;"
          ></div>
          <div
            class="track-thumb"
            style="bottom: calc({localVolume * 100}% - 2px);"
          ></div>
        </div>

        <span class="volume-label">
          {localVolume === 0 ? '0%' : `${Math.round(localVolume * 100)}%`}
        </span>
      </div>
    {/if}

    <button
      class="btn-icon"
      class:active={showVolume || isDragging}
      onclick={toggleMute}
      aria-label={localVolume === 0 ? 'Unmute' : 'Mute'}
    >
      {#if localVolume === 0}
        <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor">
          <polygon points="1,5 5,5 10,1 10,15 5,11 1,11" />
          <line x1="12" y1="5" x2="16" y2="11" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
          <line x1="16" y1="5" x2="12" y2="11" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      {:else}
        <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor">
          <polygon points="1,5 5,5 10,1 10,15 5,11 1,11" />
          <path d="M12,5 Q15,8 12,11" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      {/if}
    </button>
  </div>
</div>

<style>
  .timer-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 190px;
    margin-top: 6px;
    user-select: none;
  }

  .rounds {
    font-size: 0.8rem;
    font-weight: 450;
    color: var(--color-foreground-darker);
    min-width: 48px;
    text-align: left;
    cursor: default;
    letter-spacing: 0.04em;
    font-variant-numeric: tabular-nums;
  }

  .btn-text {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-foreground-darker);
    font-size: 0.8rem;
    font-weight: 450;
    padding: 3px 8px;
    border-radius: 5px;
    transition: all 0.15s ease;
    letter-spacing: 0.02em;
  }

  .btn-text:hover {
    color: var(--color-foreground);
    background: rgba(255, 255, 255, 0.06);
  }

  .btn-text:active {
    transform: scale(0.96);
  }

  .volume-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    min-width: 48px;
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
    border-radius: 50%;
    transition: all 0.18s ease;
  }

  .btn-icon:hover,
  .btn-icon.active {
    color: var(--color-foreground);
    background: rgba(255, 255, 255, 0.08);
  }

  .btn-icon:active {
    transform: scale(0.92);
  }

  /* macOS Control Center Style Vertical Volume Popover */
  .volume-popover {
    position: absolute;
    bottom: calc(100% + 8px);
    right: 0;
    background: rgba(22, 26, 36, 0.88);
    backdrop-filter: blur(28px) saturate(180%);
    -webkit-backdrop-filter: blur(28px) saturate(180%);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 14px;
    box-shadow: 0 10px 32px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.12);
    padding: 10px 8px 7px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    z-index: 50;
    user-select: none;
    animation: popover-fade 0.14s cubic-bezier(0.16, 1, 0.3, 1);
  }

  /* Invisible hover bridge to prevent mouseout when moving between icon and slider */
  .volume-popover::after {
    content: '';
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    height: 12px;
  }

  @keyframes popover-fade {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* Vertical Pill Track */
  .volume-track {
    width: 20px;
    height: 84px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    position: relative;
    cursor: pointer;
    overflow: hidden;
    touch-action: none;
    transition: background 0.15s, border-color 0.15s;
  }

  .volume-track:hover {
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .volume-track:focus-visible {
    outline: 2px solid var(--active-theme-color, var(--color-focus-round));
    outline-offset: 1px;
  }

  /* Filled level smoothly colored by the active theme round */
  .track-fill {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: var(--active-theme-color, var(--color-focus-round));
    border-radius: 0 0 9px 9px;
    transition: height 0.04s ease-out;
    pointer-events: none;
  }

  /* Sleek Apple-style white indicator bar at current level */
  .track-thumb {
    position: absolute;
    left: 2px;
    right: 2px;
    height: 4px;
    background: #ffffff;
    border-radius: 2px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.45);
    pointer-events: none;
    transition: bottom 0.04s ease-out;
  }

  .volume-label {
    font-size: 0.62rem;
    font-weight: 600;
    letter-spacing: -0.01em;
    font-variant-numeric: tabular-nums;
    color: var(--color-foreground-darker);
    font-family: var(--font-system);
    text-align: center;
    min-width: 28px;
    line-height: 1;
  }
</style>
