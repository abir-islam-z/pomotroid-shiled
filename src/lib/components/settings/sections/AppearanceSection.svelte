<script lang="ts">
  import { settings } from '$lib/stores/settings';
  import { setSetting } from '$lib/ipc';
  import type { Theme } from '$lib/types';
  import { getThemes } from '$lib/ipc';
  import { applyTheme } from '$lib/stores/theme';
  import { resolveThemeName } from '$lib/utils/theme';
  import { onMount } from 'svelte';
  import * as m from '$paraglide/messages.js';

  let themes = $state<Theme[]>([]);
  let openPicker = $state<'light' | 'dark' | null>(null);

  onMount(async () => {
    themes = await getThemes();
  });

  const osDark = $derived(
    typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches
  );

  const activeThemeName = $derived(resolveThemeName($settings, osDark));
  const lightIsActive = $derived(
    $settings.theme_mode === 'light' || ($settings.theme_mode === 'auto' && !osDark)
  );
  const darkIsActive = $derived(
    $settings.theme_mode === 'dark' || ($settings.theme_mode === 'auto' && osDark)
  );

  const selectedLightTheme = $derived(themes.find((t) => t.name === $settings.theme_light));
  const selectedDarkTheme = $derived(themes.find((t) => t.name === $settings.theme_dark));

  async function setMode(mode: 'auto' | 'light' | 'dark') {
    const updated = await setSetting('theme_mode', mode);
    settings.set(updated);
    const target = resolveThemeName(updated, osDark);
    const t = themes.find((th) => th.name === target);
    if (t) applyTheme(t);
  }

  async function selectLight(theme: Theme) {
    const updated = await setSetting('theme_light', theme.name);
    settings.set(updated);
    if (lightIsActive) applyTheme(theme);
    openPicker = null;
  }

  async function selectDark(theme: Theme) {
    const updated = await setSetting('theme_dark', theme.name);
    settings.set(updated);
    if (darkIsActive) applyTheme(theme);
    openPicker = null;
  }

  function togglePicker(which: 'light' | 'dark') {
    openPicker = openPicker === which ? null : which;
  }
</script>

<div class="section">
  <!-- Mode label -->
  <div class="group-label">Theme Mode</div>

  <!-- Apple Native Segmented Control -->
  <div class="mode-selector">
    <button
      class="mode-btn"
      class:active={$settings.theme_mode === 'auto'}
      onclick={() => setMode('auto')}
    >
      {m.appearance_mode_auto()}
    </button>
    <button
      class="mode-btn"
      class:active={$settings.theme_mode === 'light'}
      onclick={() => setMode('light')}
    >
      {m.appearance_mode_light()}
    </button>
    <button
      class="mode-btn"
      class:active={$settings.theme_mode === 'dark'}
      onclick={() => setMode('dark')}
    >
      {m.appearance_mode_dark()}
    </button>
  </div>

  <!-- Light theme picker -->
  <div class="picker-group">
    <button
      class="picker-trigger"
      class:open={openPicker === 'light'}
      onclick={() => togglePicker('light')}
    >
      <span class="trigger-label">
        Light Theme
        {#if lightIsActive}<span class="active-badge">Active</span>{/if}
      </span>
      <span class="trigger-preview">
        <span class="preview-name">{$settings.theme_light}</span>
        {#if selectedLightTheme}
          {@const bg = selectedLightTheme.colors['--color-background'] ?? '#2f384b'}
          {@const focusRound = selectedLightTheme.colors['--color-focus-round'] ?? '#e25d60'}
          {@const shortRound = selectedLightTheme.colors['--color-short-round'] ?? '#3baf82'}
          {@const longRound = selectedLightTheme.colors['--color-long-round'] ?? '#3d85c8'}
          <span class="preview-swatches" style="background:{bg}">
            <span class="swatch" style="background:{focusRound}"></span>
            <span class="swatch" style="background:{shortRound}"></span>
            <span class="swatch" style="background:{longRound}"></span>
          </span>
        {/if}
      </span>
      <svg
        class="chevron"
        class:rotated={openPicker === 'light'}
        width="12"
        height="12"
        viewBox="0 0 24 24"
        fill="currentColor"
      >
        <path d="M7 10l5 5 5-5z" />
      </svg>
    </button>
    {#if openPicker === 'light'}
      <div class="theme-list">
        {#each themes as theme (theme.name)}
          {@const bg = theme.colors['--color-background'] ?? '#2f384b'}
          {@const fg = theme.colors['--color-foreground'] ?? '#d7e1f4'}
          {@const accent = theme.colors['--color-accent'] ?? '#e25d60'}
          {@const focusRound = theme.colors['--color-focus-round'] ?? '#e25d60'}
          {@const shortRound = theme.colors['--color-short-round'] ?? '#3baf82'}
          {@const longRound = theme.colors['--color-long-round'] ?? '#3d85c8'}
          {@const isSelected = theme.name === $settings.theme_light}
          <button
            class="card"
            class:selected={isSelected}
            class:highlighted={isSelected && lightIsActive}
            style="--card-bg:{bg}; --card-fg:{fg}; --card-accent:{accent};"
            onclick={() => selectLight(theme)}
          >
            <span class="swatches">
              <span class="swatch" style="background:{focusRound}"></span>
              <span class="swatch" style="background:{shortRound}"></span>
              <span class="swatch" style="background:{longRound}"></span>
            </span>
            <span class="card-name" style="color:{fg}">{theme.name}</span>
            <span class="card-right">
              {#if theme.is_custom}
                <span class="badge" style="color:{accent}">{m.appearance_badge_custom()}</span>
              {/if}
              {#if isSelected}
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  style="fill:{accent}; flex-shrink:0;"
                >
                  <path d="M9 16.2L4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4L9 16.2z" />
                </svg>
              {/if}
            </span>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Dark theme picker -->
  <div class="picker-group">
    <button
      class="picker-trigger"
      class:open={openPicker === 'dark'}
      onclick={() => togglePicker('dark')}
    >
      <span class="trigger-label">
        Dark Theme
        {#if darkIsActive}<span class="active-badge">Active</span>{/if}
      </span>
      <span class="trigger-preview">
        <span class="preview-name">{$settings.theme_dark}</span>
        {#if selectedDarkTheme}
          {@const bg = selectedDarkTheme.colors['--color-background'] ?? '#2f384b'}
          {@const focusRound = selectedDarkTheme.colors['--color-focus-round'] ?? '#e25d60'}
          {@const shortRound = selectedDarkTheme.colors['--color-short-round'] ?? '#3baf82'}
          {@const longRound = selectedDarkTheme.colors['--color-long-round'] ?? '#3d85c8'}
          <span class="preview-swatches" style="background:{bg}">
            <span class="swatch" style="background:{focusRound}"></span>
            <span class="swatch" style="background:{shortRound}"></span>
            <span class="swatch" style="background:{longRound}"></span>
          </span>
        {/if}
      </span>
      <svg
        class="chevron"
        class:rotated={openPicker === 'dark'}
        width="12"
        height="12"
        viewBox="0 0 24 24"
        fill="currentColor"
      >
        <path d="M7 10l5 5 5-5z" />
      </svg>
    </button>
    {#if openPicker === 'dark'}
      <div class="theme-list">
        {#each themes as theme (theme.name)}
          {@const bg = theme.colors['--color-background'] ?? '#2f384b'}
          {@const fg = theme.colors['--color-foreground'] ?? '#d7e1f4'}
          {@const accent = theme.colors['--color-accent'] ?? '#e25d60'}
          {@const focusRound = theme.colors['--color-focus-round'] ?? '#e25d60'}
          {@const shortRound = theme.colors['--color-short-round'] ?? '#3baf82'}
          {@const longRound = theme.colors['--color-long-round'] ?? '#3d85c8'}
          {@const isSelected = theme.name === $settings.theme_dark}
          <button
            class="card"
            class:selected={isSelected}
            class:highlighted={isSelected && darkIsActive}
            style="--card-bg:{bg}; --card-fg:{fg}; --card-accent:{accent};"
            onclick={() => selectDark(theme)}
          >
            <span class="swatches">
              <span class="swatch" style="background:{focusRound}"></span>
              <span class="swatch" style="background:{shortRound}"></span>
              <span class="swatch" style="background:{longRound}"></span>
            </span>
            <span class="card-name" style="color:{fg}">{theme.name}</span>
            <span class="card-right">
              {#if theme.is_custom}
                <span class="badge" style="color:{accent}">{m.appearance_badge_custom()}</span>
              {/if}
              {#if isSelected}
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  style="fill:{accent}; flex-shrink:0;"
                >
                  <path d="M9 16.2L4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4L9 16.2z" />
                </svg>
              {/if}
            </span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .section {
    display: flex;
    flex-direction: column;
    padding-bottom: 20px;
  }

  .group-label {
    font-size: 0.74rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.55);
    margin: 0;
    padding: 18px 20px 8px;
    letter-spacing: -0.01em;
  }

  .active-badge {
    font-size: 0.58rem;
    font-weight: 500;
    letter-spacing: 0.02em;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.08);
    color: rgba(255, 255, 255, 0.65);
  }

  /* Native Apple Segmented Control */
  .mode-selector {
    display: flex;
    margin: 0 20px 10px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 2px;
    gap: 2px;
  }

  .mode-btn {
    flex: 1;
    padding: 5px 0;
    background: transparent;
    border: none;
    border-radius: 6px;
    font-size: 0.78rem;
    font-weight: 400;
    color: rgba(255, 255, 255, 0.65);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .mode-btn:hover {
    color: rgba(255, 255, 255, 0.9);
  }

  .mode-btn.active {
    background: rgba(255, 255, 255, 0.14);
    color: #ffffff;
    font-weight: 500;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }

  /* Inset grouped rows */
  .picker-group {
    margin: 8px 20px 0;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    overflow: hidden;
  }

  .picker-trigger {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 9px 12px;
    background: none;
    border: none;
    cursor: pointer;
    gap: 8px;
    transition: background 0.12s;
  }

  .picker-trigger:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .picker-trigger.open {
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .trigger-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.8rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.75);
    flex-shrink: 0;
  }

  .trigger-preview {
    display: flex;
    align-items: center;
    gap: 7px;
    flex: 1;
    justify-content: flex-end;
    min-width: 0;
  }

  .preview-swatches {
    display: flex;
    gap: 3px;
    flex-shrink: 0;
    padding: 3px 5px;
    border-radius: 4px;
  }

  .preview-name {
    font-size: 0.78rem;
    color: rgba(255, 255, 255, 0.6);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  .chevron {
    flex-shrink: 0;
    color: rgba(255, 255, 255, 0.45);
    transition: transform 0.15s;
  }

  .chevron.rotated {
    transform: rotate(180deg);
  }

  /* Theme list */
  .theme-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 6px 8px 8px;
    max-height: 220px;
    overflow-y: auto;
  }

  .card {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-radius: 6px;
    border: 1px solid transparent;
    background: var(--card-bg);
    cursor: pointer;
    width: 100%;
    text-align: left;
    transition: all 0.15s ease;
  }

  .card:hover {
    background: color-mix(in srgb, var(--card-bg) 85%, white 15%);
  }

  .card.selected {
    border-color: rgba(255, 255, 255, 0.25);
  }

  .card.highlighted {
    border-color: rgba(255, 255, 255, 0.4);
  }

  .swatches {
    display: flex;
    gap: 3px;
    flex-shrink: 0;
  }

  .swatch {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.12);
  }

  .card-name {
    font-size: 0.8rem;
    font-weight: 400;
    flex: 1;
  }

  .card-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .badge {
    font-size: 0.62rem;
    text-transform: uppercase;
    opacity: 0.7;
  }
</style>
