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

<button class="row" {onclick}>
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
    padding: 10px 20px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--color-separator);
    cursor: pointer;
    text-align: left;
    gap: 16px;
    transition: background 0.12s;
  }

  .row:hover {
    background: var(--color-hover);
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .label {
    font-size: 0.85rem;
    color: var(--color-foreground);
    letter-spacing: 0.02em;
  }

  .desc {
    font-size: 0.72rem;
    color: var(--color-foreground-darker, var(--color-foreground));
    letter-spacing: 0.02em;
    opacity: 0.7;
  }

  /* Native macOS Pill toggle */
  .toggle {
    position: relative;
    width: 38px;
    height: 22px;
    border-radius: 11px;
    background: color-mix(in oklch, var(--color-foreground) 22%, transparent);
    flex-shrink: 0;
    transition: background 0.22s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.22s ease;
  }

  .toggle::after {
    content: '';
    position: absolute;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #ffffff;
    top: 2px;
    left: 2px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
    transition: transform 0.22s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .toggle.on {
    background: var(--color-accent);
    box-shadow: 0 0 10px color-mix(in oklch, var(--color-accent) 35%, transparent);
  }

  .toggle.on::after {
    transform: translateX(16px);
    background: #ffffff;
  }
</style>
