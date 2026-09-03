<script lang="ts">
  // A labelled toggle-switch row. Click anywhere on the row to toggle.
  import TooltipInfo from '$lib/components/TooltipInfo.svelte';

  interface Props {
    label: string;
    checked: boolean;
    description?: string;
    tooltip?: string;
    onclick: () => void;
  }

  let { label, checked, description, tooltip, onclick }: Props = $props();
</script>

<button class="row" {onclick} type="button">
  <span class="text">
    <span class="label"
      >{label}{#if tooltip}<TooltipInfo text={tooltip} />{/if}</span
    >
    {#if description}
      <span class="desc">{description}</span>
    {/if}
  </span>
  <span class="toggle" class:on={checked} aria-checked={checked} role="switch"></span>
</button>

<style>
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 11px 16px;
    background: transparent;
    border: none;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    cursor: pointer;
    text-align: left;
    gap: 16px;
    transition: background 0.14s ease;
  }

  .row:last-child {
    border-bottom: none;
  }

  .row:hover {
    background: rgba(255, 255, 255, 0.035);
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .label {
    font-size: 0.83rem;
    font-weight: 450;
    color: rgba(255, 255, 255, 0.9);
    letter-spacing: -0.01em;
  }

  .desc {
    font-size: 0.72rem;
    color: rgba(255, 255, 255, 0.52);
    line-height: 1.35;
    letter-spacing: -0.005em;
  }

  /* ---------------------------------------------------------------------------
     Apple macOS Glassmorphic Switch Toggle
     --------------------------------------------------------------------------- */
  .toggle {
    position: relative;
    width: 38px;
    height: 22px;
    border-radius: 11px;
    flex-shrink: 0;
    cursor: pointer;
    box-sizing: border-box;

    /* Off state: Recessed frosted glass track */
    background: rgba(0, 0, 0, 0.32);
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.35);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);

    transition:
      background 0.25s cubic-bezier(0.16, 1, 0.3, 1),
      border-color 0.25s cubic-bezier(0.16, 1, 0.3, 1),
      box-shadow 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  }

  /* Tactile white specular thumb disc */
  .toggle::after {
    content: '';
    position: absolute;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    top: 1px;
    left: 1px;
    background: linear-gradient(180deg, #FFFFFF 0%, #ECEFF4 100%);
    box-shadow:
      0 2px 4px rgba(0, 0, 0, 0.32),
      0 1px 1px rgba(0, 0, 0, 0.2),
      inset 0 1px 0.5px #FFFFFF;
    transition:
      transform 0.24s cubic-bezier(0.175, 0.885, 0.32, 1.25),
      width 0.18s ease;
  }

  .row:active .toggle::after {
    width: 20px; /* Apple spring squish on active click */
  }

  /* On state: Radiant glassmorphic accent track with specular highlight */
  .toggle.on {
    background: linear-gradient(
      180deg,
      color-mix(in srgb, var(--color-accent) 82%, rgba(255, 255, 255, 0.32)) 0%,
      color-mix(in srgb, var(--color-accent) 90%, rgba(0, 0, 0, 0.18)) 100%
    );
    border: 1px solid rgba(255, 255, 255, 0.22);
    box-shadow:
      inset 0 1px 1px rgba(255, 255, 255, 0.45),
      inset 0 -1px 2px rgba(0, 0, 0, 0.25),
      0 2px 10px color-mix(in srgb, var(--color-accent) 30%, transparent);
  }

  .toggle.on::after {
    transform: translateX(16px);
  }

  .row:active .toggle.on::after {
    transform: translateX(14px);
    width: 20px;
  }
</style>
