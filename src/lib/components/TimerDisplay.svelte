<script lang="ts">
  // Displays the remaining time (MM:SS) in Apple Watch / SF Pro Rounded light numerals.
  import type { TimerState } from '$lib/types';

  interface Props {
    state: TimerState;
  }

  let { state }: Props = $props();

  let remaining = $derived(state.total_secs - state.elapsed_secs);
  let minutes = $derived(Math.floor(remaining / 60));
  let seconds = $derived(remaining % 60);
  let display = $derived(`${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`);
</script>

<div class="display">
  <span class="time">{display}</span>
</div>

<style>
  .display {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
  }

  .time {
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Rounded", "SF Pro Display", "SF Mono", "Mona Sans Mono", monospace;
    font-size: 3.15rem;
    font-weight: 250;
    letter-spacing: -0.04em;
    color: var(--color-foreground);
    font-variant-numeric: tabular-nums;
    text-shadow: 0 2px 16px rgba(0, 0, 0, 0.35);
  }
</style>
