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

      <div class="transport-controls" style="--active-theme-color: {roundColor(state.round_type)};">
        <!-- Back: restart current round -->
        <Tooltip text={m.tooltip_restart_round()}>
          <button class="btn-side" onclick={timerRestartRound} aria-label="Restart round">
            <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor">
              <polygon points="15,1 6,8 15,15" />
              <rect x="1" y="1" width="2.5" height="14" rx="0.5" />
            </svg>
          </button>
        </Tooltip>

        <!-- Play / Pause -->
        <button
          class="play-pause"
          class:running={state.is_running}
          onclick={timerToggle}
          aria-label={state.is_running ? 'Pause' : 'Play'}
        >
          {#key state.is_running}
            <span class="icon" in:fade={{ duration: 120 }}>
              {#if state.is_running}
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <rect x="6" y="3.5" width="4" height="17" rx="1.5"/>
                  <rect x="14" y="3.5" width="4" height="17" rx="1.5"/>
                </svg>
              {:else}
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" style="margin-left: 2px;">
                  <polygon points="5,3 21,12 5,21"/>
                </svg>
              {/if}
            </span>
          {/key}
        </button>

        <!-- Skip: advance to next round -->
        <Tooltip text={m.tooltip_skip()}>
          <button class="btn-side" onclick={timerSkip} aria-label="Skip round">
            <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor">
              <polygon points="1,1 10,8 1,15" />
              <rect x="12.5" y="1" width="2.5" height="14" rx="0.5" />
            </svg>
          </button>
        </Tooltip>
      </div>

      <TimerFooter snap={state} />
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

  .transport-controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 22px;
    margin-top: 6px;
  }

  .btn-side {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-foreground-darker);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    transition: all 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .btn-side:hover {
    color: var(--color-foreground);
    background: rgba(255, 255, 255, 0.08);
    transform: scale(1.08);
  }

  .btn-side:active {
    transform: scale(0.92);
  }

  .play-pause {
    background: transparent;
    border: 1.5px solid var(--color-foreground-darker, rgba(255, 255, 255, 0.35));
    border-radius: 50%;
    cursor: pointer;
    color: var(--color-foreground);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
    transition: all 0.22s cubic-bezier(0.16, 1, 0.3, 1);
    position: relative;
  }

  .play-pause:hover {
    color: var(--active-theme-color, var(--color-focus-round));
    border-color: var(--active-theme-color, var(--color-focus-round));
    background: color-mix(in srgb, var(--active-theme-color, var(--color-focus-round)) 10%, transparent);
    box-shadow: 0 0 18px color-mix(in srgb, var(--active-theme-color, var(--color-focus-round)) 22%, transparent);
    transform: scale(1.05);
  }

  .play-pause.running {
    border-color: var(--active-theme-color, var(--color-focus-round));
    color: var(--active-theme-color, var(--color-focus-round));
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
    font-weight: 650;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    padding: 3px 12px;
    border-radius: 20px;
    background: rgba(255, 255, 255, 0.04);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.07);
    margin-top: -6px;
    box-shadow: none;
  }
</style>
