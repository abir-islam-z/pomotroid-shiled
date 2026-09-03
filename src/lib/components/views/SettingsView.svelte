<script lang="ts">
  import TimerSection from '$lib/components/settings/sections/TimerSection.svelte';
  import AppearanceSection from '$lib/components/settings/sections/AppearanceSection.svelte';
  import NotificationsSection from '$lib/components/settings/sections/NotificationsSection.svelte';
  import ShortcutsSection from '$lib/components/settings/sections/ShortcutsSection.svelte';
  import SystemSection from '$lib/components/settings/sections/SystemSection.svelte';
  import ShieldSection from '$lib/components/settings/sections/ShieldSection.svelte';
  import AboutSection from '$lib/components/settings/sections/AboutSection.svelte';

  import * as m from '$paraglide/messages.js';

  type Section = 'timer' | 'appearance' | 'notifications' | 'shortcuts' | 'shield' | 'system' | 'about';

  const SECTIONS: { id: Section; label: () => string }[] = [
    { id: 'timer', label: m.nav_timer },
    { id: 'appearance', label: m.nav_appearance },
    { id: 'notifications', label: m.nav_notifications },
    { id: 'shortcuts', label: m.nav_shortcuts },
    { id: 'shield', label: () => 'Shield & Focus' },
    { id: 'system', label: m.nav_system },
    { id: 'about', label: m.nav_about },
  ];

  let active = $state<Section>('timer');
</script>

<div class="settings-view">
  <!-- Left Inset Sidebar Navigation with Apple Squircle Badges -->
  <aside class="sidebar">
    <nav>
      {#each SECTIONS as section}
        <button
          class="nav-item"
          class:active={active === section.id}
          onclick={() => {
            active = section.id;
          }}
        >
          <span class="nav-badge badge-{section.id}">
            {#if section.id === 'timer'}
              <!-- Clock -->
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"/>
                <polyline points="12 6 12 12 16 14"/>
              </svg>
            {:else if section.id === 'appearance'}
              <!-- Sun -->
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="4"/>
                <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>
              </svg>
            {:else if section.id === 'notifications'}
              <!-- Bell -->
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/>
                <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>
              </svg>
            {:else if section.id === 'shortcuts'}
              <!-- Keyboard -->
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <rect width="20" height="16" x="2" y="4" rx="2"/>
                <path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M8 12h8"/>
              </svg>
            {:else if section.id === 'shield'}
              <!-- Shield -->
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
              </svg>
            {:else if section.id === 'system'}
              <!-- Gear -->
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3"/>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
              </svg>
            {:else}
              <!-- Info -->
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" y1="16" x2="12" y2="12"/>
                <line x1="12" y1="8" x2="12.01" y2="8"/>
              </svg>
            {/if}
          </span>
          <span class="nav-label">{section.label()}</span>
        </button>
      {/each}
    </nav>
  </aside>

  <!-- Right Content Area -->
  <main class="content">
    {#if active === 'timer'}
      <TimerSection />
    {:else if active === 'appearance'}
      <AppearanceSection />
    {:else if active === 'notifications'}
      <NotificationsSection />
    {:else if active === 'shortcuts'}
      <ShortcutsSection />
    {:else if active === 'shield'}
      <ShieldSection />
    {:else if active === 'system'}
      <SystemSection />
    {:else if active === 'about'}
      <AboutSection />
    {/if}
  </main>
</div>

<style>
  .settings-view {
    display: flex;
    height: 100%;
    width: 100%;
    overflow: hidden;
    color: var(--color-foreground);
    animation: view-enter 0.25s cubic-bezier(0.16, 1, 0.3, 1) both;
  }

  @keyframes view-enter {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* macOS System Settings Inset Sidebar */
  .sidebar {
    width: 215px;
    flex-shrink: 0;
    border-right: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(0, 0, 0, 0.12);
    backdrop-filter: blur(24px);
    -webkit-backdrop-filter: blur(24px);
    overflow-y: auto;
    padding: 12px 10px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 6px 9px;
    margin-bottom: 2px;
    background: transparent;
    border: none;
    border-radius: 8px;
    text-align: left;
    font-size: 0.82rem;
    font-weight: 400;
    color: rgba(255, 255, 255, 0.72);
    cursor: pointer;
    transition: all 0.14s ease;
  }

  /* Apple System Settings Squircle Badges */
  .nav-badge {
    width: 21px;
    height: 21px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #ffffff;
    flex-shrink: 0;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.28);
    transition: transform 0.15s ease;
  }

  .badge-timer {
    background: linear-gradient(135deg, #FF6B4A, #FF453A);
  }
  .badge-appearance {
    background: linear-gradient(135deg, #5E5CE6, #7B79FF);
  }
  .badge-notifications {
    background: linear-gradient(135deg, #FF375F, #E02440);
  }
  .badge-shortcuts {
    background: linear-gradient(135deg, #636366, #48484A);
  }
  .badge-shield {
    background: linear-gradient(135deg, #0A84FF, #0066CC);
  }
  .badge-system {
    background: linear-gradient(135deg, #8E8E93, #636366);
  }
  .badge-about {
    background: linear-gradient(135deg, #30D158, #248A3D);
  }

  .nav-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .nav-item:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.06);
  }

  .nav-item:hover .nav-badge {
    transform: scale(1.05);
  }

  .nav-item.active {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.13);
    font-weight: 500;
  }

  /* Content */
  .content {
    flex: 1;
    overflow-y: auto;
    min-width: 0;
    background: rgba(255, 255, 255, 0.015);
  }
</style>
