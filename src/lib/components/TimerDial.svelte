<script lang="ts">
  // SVG arc dial showing timer progress.
  // Smoothly fills around the dial with subtle, classy Apple-like depth.
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import type { TimerState } from '$lib/types';

  interface Props {
    snap: TimerState;
    countdown?: boolean;
  }

  let { snap, countdown = false }: Props = $props();

  // SVG constants
  const CIRCUMFERENCE = 691.15; // 2π × 110 ≈ 691.15

  // Tweened offset: starts at full circumference (invisible), animates smoothly.
  const dashOffset = tweened(CIRCUMFERENCE, { duration: 750, easing: cubicOut });

  // Round-type → CSS color
  function strokeColor(rt: string): string {
    if (rt === 'work') return 'var(--color-focus-round)';
    if (rt === 'short-break') return 'var(--color-short-round)';
    return 'var(--color-long-round)';
  }

  let prevRound = $state<string>('');

  $effect(() => {
    const rt = snap.round_type;
    const progress = snap.total_secs > 0 ? snap.elapsed_secs / snap.total_secs : 0;
    const target = countdown ? CIRCUMFERENCE * progress : CIRCUMFERENCE * (1 - progress);
    const startOffset = countdown ? 0 : CIRCUMFERENCE;

    if (rt !== prevRound) {
      dashOffset.set(startOffset, { duration: 0 });
      prevRound = rt;
    } else {
      dashOffset.set(target);
    }
  });
</script>

<svg class="dial" viewBox="0 0 230 230" aria-hidden="true">
  <!-- Subtle glass background track -->
  <path
    class="track"
    d="M115,5c60.8,0,110,49.2,110,110s-49.2,110-110,110S5,175.8,5,115S54.2,5,115,5"
    fill="none"
    stroke="rgba(255, 255, 255, 0.07)"
    stroke-width="5"
  />
  <!-- Progress arc -->
  <path
    class="progress"
    d="M115,5c60.8,0,110,49.2,110,110s-49.2,110-110,110S5,175.8,5,115S54.2,5,115,5"
    fill="none"
    stroke={strokeColor(snap.round_type)}
    stroke-width="7"
    stroke-linecap="round"
    stroke-dasharray={CIRCUMFERENCE}
    stroke-dashoffset={$dashOffset}
    style="filter: drop-shadow(0 0 6px color-mix(in oklch, {strokeColor(snap.round_type)} 35%, transparent));"
  />
</svg>

<style>
  .dial {
    width: 220px;
    height: 220px;
    display: block;
  }
</style>
