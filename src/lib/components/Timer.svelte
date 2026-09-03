<script lang="ts">
  // Orchestrator component. Subscribes to timer events, owns keyboard listener,
  // and renders TimerDial + TimerDisplay + TimerFooter.
  import { onMount } from 'svelte';
  import {
    timerToggle,
    timerRestartRound,
    timerSkip,
    getTimerState,
    onTimerTick,
    onTimerPaused,
    onTimerResumed,
    onRoundChange,
    onTimerReset,
  } from '$lib/ipc';
  import { timerState } from '$lib/stores/timer';
  import { settings } from '$lib/stores/settings';
  import { fade } from 'svelte/transition';
  import TimerDial from './TimerDial.svelte';
  import TimerDisplay from './TimerDisplay.svelte';
  import TimerFooter from './TimerFooter.svelte';
  import MiniControls from './MiniControls.svelte';
  import Tooltip from './Tooltip.svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import * as m from '$paraglide/messages.js';

  interface Props {
    isCompact?: boolean;
    uiScale?: number;
  }

  let { isCompact = false, uiScale = 1 }: Props = $props();

  let state = $derived($timerState);

  function roundColor(rt: string): string {
    if (rt === 'work') return 'var(--color-focus-round)';
    if (rt === 'short-break') return 'var(--color-short-round)';
    return 'var(--color-long-round)';
  }

  function roundLabel(rt: string): string {
    if (rt === 'work') return m.round_label_work();
    if (rt === 'short-break') return m.round_label_short_break();
    return m.round_label_long_break();
  }

  onMount(() => {
    const cleanups: UnlistenFn[] = [];

    (async () => {
      const initial = await getTimerState();
      timerState.set(initial);

      cleanups.push(
        await onTimerTick(({ elapsed_secs, total_secs }) => {
          timerState.update((s) => ({
            ...s,
            elapsed_secs,
            total_secs,
            is_running: true,
            is_paused: false,
          }));
        }),
        await onTimerPaused(({ elapsed_secs }) => {
          timerState.update((s) => ({
            ...s,
            elapsed_secs,
            is_running: false,
            is_paused: true,
          }));
        }),
        await onTimerResumed(({ elapsed_secs }) => {
          timerState.update((s) => ({
            ...s,
            elapsed_secs,
            is_running: true,
            is_paused: false,
          }));
        }),
        await onRoundChange((snap) => {
          timerState.set(snap);
        }),
        await onTimerReset(() => {
          getTimerState().then((snap) => timerState.set(snap));
        })
      );
    })();

    return () => {
      for (const fn of cleanups) fn();
    };
  });
</script>

<div
  class="timer-outer"
  style="--ui-scale: {uiScale}; transform: scale(var(--ui-scale)); transform-origin: top center;"
>
  <div class="timer">
    <div class="dial-stack">
      <TimerDial snap={state} countdown={$settings.dial_countdown} />
      <TimerDisplay {state} />
    </div>

    {#if !isCompact}
      <!-- Refined Micro-capsule Round Badge -->
      <div class="round-label" style="color: {roundColor(state.round_type)};">
        {roundLabel(state.round_type)}
      </div>

      <div class="controls-wrapper">
        <!-- Back: restart current round (Apple Music style rewind) -->
        <Tooltip text={m.tooltip_restart_round()}>
          <button class="btn-side" onclick={timerRestartRound} aria-label="Restart round">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 12.75l7.65 5.1a1 1 0 0 0 1.55-.83V6.98a1 1 0 0 0-1.55-.83L12 11.25V6.98a1 1 0 0 0-1.55-.83L2.8 11.27a1 1 0 0 0 0 1.66l7.65 5.12a1 1 0 0 0 1.55-.83v-4.47z"/>
            </svg>
          </button>
        </Tooltip>

        <!-- Play / Pause — Translucent Apple Glass Button with smooth icon fade -->
        <button
          class="play-pause"
          onclick={timerToggle}
          aria-label={state.is_running ? 'Pause' : 'Play'}
        >
          {#key state.is_running}
            <span class="icon" in:fade={{ duration: 140 }}>
              {#if state.is_running}
                <!-- Apple Style Smooth Rounded Dual Bars -->
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <rect x="5.5" y="4" width="4.5" height="16" rx="2.25"/>
                  <rect x="14" y="4" width="4.5" height="16" rx="2.25"/>
                </svg>
              {:else}
                <!-- Apple Style Smooth Rounded Play Triangle -->
                <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" style="margin-left: 2px;">
                  <path d="M7.05 4.45a1.5 1.5 0 0 1 2.27-1.28l11.5 6.64a1.5 1.5 0 0 1 0 2.6l-11.5 6.64a1.5 1.5 0 0 1-2.27-1.28V4.45z"/>
                </svg>
              {/if}
            </span>
          {/key}
        </button>

        <!-- Skip: advance to next round (Apple Music style fast-forward) -->
        <Tooltip text={m.tooltip_skip()}>
          <button class="btn-side" onclick={timerSkip} aria-label="Skip round">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 11.25L4.35 6.15A1 1 0 0 0 2.8 6.98v10.04a1 1 0 0 0 1.55.83L12 12.75v4.27a1 1 0 0 0 1.55.83l7.65-5.12a1 1 0 0 0 0-1.66l-7.65-5.12a1 1 0 0 0-1.55.83v4.27z"/>
            </svg>
          </button>
        </Tooltip>

        <TimerFooter snap={state} />
      </div>
    {/if}
  </div>

  {#if isCompact}
    <MiniControls />
  {/if}
</div>

<style>
  .timer-outer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .timer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
  }

  .dial-stack {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .controls-wrapper {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 4px 14px;
    align-items: center;
  }
  .controls-wrapper > :global(*) {
    aspect-ratio: 1;
  }

  .btn-side {
    background: rgba(255, 255, 255, 0.05);
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    border: 1px solid rgba(255, 255, 255, 0.09);
    cursor: pointer;
    color: var(--color-foreground-darker);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .btn-side:hover {
    color: var(--color-foreground);
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.2);
    transform: scale(1.06);
  }

  .btn-side:active {
    transform: scale(0.94);
  }

  .play-pause {
    background: rgba(255, 255, 255, 0.08);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border: 1px solid rgba(255, 255, 255, 0.16);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25), inset 0 1px 0 rgba(255, 255, 255, 0.2);
    cursor: pointer;
    color: var(--color-foreground);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 52px;
    height: 52px;
    border-radius: 50%;
    transition: all 0.22s cubic-bezier(0.16, 1, 0.3, 1);
    overflow: hidden;
  }

  .play-pause:hover {
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.3);
    box-shadow: 0 6px 22px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.3);
    transform: scale(1.06);
  }

  .play-pause:active {
    transform: scale(0.94);
  }

  .icon {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .round-label {
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    padding: 3px 12px;
    border-radius: 20px;
    background: rgba(255, 255, 255, 0.05);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    margin-top: -6px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.18), inset 0 0.5px 0 rgba(255, 255, 255, 0.1);
  }
</style>
