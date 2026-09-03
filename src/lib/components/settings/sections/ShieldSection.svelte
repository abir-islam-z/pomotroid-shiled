<script lang="ts">
  import { settings } from '$lib/stores/settings';
  import { setSetting, systemBridgeTestBreakLock, systemBridgeCloseBreakLock } from '$lib/ipc';
  import SettingsToggle from '$lib/components/settings/SettingsToggle.svelte';

  let newDomain = $state('');
  let testingLock = $state(false);

  let domains = $derived(
    ($settings.system_blocked_domains || '')
      .split(',')
      .map((s) => s.trim().toLowerCase())
      .filter(Boolean)
  );

  async function toggle(key: string, current: boolean) {
    const updated = await setSetting(key, current ? 'false' : 'true');
    settings.set(updated);
  }

  async function saveDomains(list: string[]) {
    const updated = await setSetting('system_blocked_domains', list.join(', '));
    settings.set(updated);
  }

  function addDomain() {
    let d = newDomain.trim().toLowerCase();
    if (d.startsWith('https://')) d = d.slice(8);
    if (d.startsWith('http://')) d = d.slice(7);
    if (d.includes('/')) d = d.split('/')[0];
    if (d.includes(':')) d = d.split(':')[0];

    if (d && !domains.includes(d)) {
      saveDomains([...domains, d]);
    }
    newDomain = '';
  }

  function removeDomain(domain: string) {
    saveDomains(domains.filter((d) => d !== domain));
  }

  function resetDefaultDomains() {
    saveDomains([
      'twitter.com',
      'x.com',
      'facebook.com',
      'instagram.com',
      'youtube.com',
      'reddit.com',
      'tiktok.com',
      'linkedin.com',
      'netflix.com',
      'twitch.tv'
    ]);
  }

  async function triggerTestLock() {
    testingLock = true;
    await systemBridgeTestBreakLock();
    setTimeout(() => {
      testingLock = false;
    }, 10000);
  }
</script>

<div class="section">
  <div class="group">
    <div class="group-title">Focus & System Protection</div>

    <SettingsToggle
      label="System-Wide Focus Block"
      description="Block distracting websites system-wide across all browsers during focus sessions"
      checked={$settings.system_block_enabled}
      onclick={() => toggle('system_block_enabled', $settings.system_block_enabled)}
    />

    <SettingsToggle
      label="24/7 Adult Content Shield"
      description="Block adult and explicit websites 24/7 at the OS network level"
      checked={$settings.system_adult_shield_enabled}
      onclick={() => toggle('system_adult_shield_enabled', $settings.system_adult_shield_enabled)}
    />

    <SettingsToggle
      label="Break Screen Lock (Short Breaks)"
      description="Hardware-shielded full-screen overlay with countdown and mindfulness during short breaks"
      checked={$settings.system_break_lock_enabled}
      onclick={() => toggle('system_break_lock_enabled', $settings.system_break_lock_enabled)}
    />

    <SettingsToggle
      label="Desktop Media Auto-Pause"
      description="Automatically pause Spotify, Apple Music, and VLC on break or timer pause"
      checked={$settings.system_media_pause_enabled}
      onclick={() => toggle('system_media_pause_enabled', $settings.system_media_pause_enabled)}
    />
  </div>

  <div class="group">
    <div class="group-header">
      <div class="group-title">Blocked Domains ({domains.length})</div>
      <button class="btn-link" onclick={resetDefaultDomains}>Reset Defaults</button>
    </div>

    <div class="add-row">
      <input
        type="text"
        placeholder="Add domain (e.g. reddit.com)"
        bind:value={newDomain}
        onkeydown={(e) => {
          if (e.key === 'Enter') addDomain();
        }}
      />
      <button class="btn-add" onclick={addDomain}>Add</button>
    </div>

    <div class="tags-container">
      {#each domains as domain}
        <span class="tag">
          <span class="tag-name">{domain}</span>
          <button
            class="tag-remove"
            title="Remove {domain}"
            onclick={() => removeDomain(domain)}
          >×</button>
        </span>
      {/each}
    </div>
  </div>

  <div class="group test-group">
    <div class="test-desc">
      Test your full-screen Break Lock overlay. (Press ESC or wait 6 seconds to exit).
    </div>
    <button class="btn-test" onclick={triggerTestLock} disabled={testingLock}>
      {testingLock ? 'Preview Active (Press ESC)...' : 'Preview Break Lock Screen'}
    </button>
  </div>
</div>

<style>
  .section {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-bottom: 24px;
  }

  .group {
    display: flex;
    flex-direction: column;
  }

  .group-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 20px 6px;
  }

  .group-title {
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-foreground-darker, var(--color-foreground));
    opacity: 0.7;
    padding: 10px 20px 6px;
  }

  .group-header .group-title {
    padding: 0;
  }

  .btn-link {
    background: none;
    border: none;
    font-size: 0.72rem;
    color: var(--color-accent);
    cursor: pointer;
    text-decoration: underline;
    opacity: 0.85;
  }

  .btn-link:hover {
    opacity: 1;
  }

  .add-row {
    display: flex;
    gap: 8px;
    padding: 6px 20px 12px;
  }

  .add-row input {
    flex: 1;
    background: var(--color-background-darker, rgba(0, 0, 0, 0.15));
    border: 1px solid var(--color-separator);
    border-radius: 6px;
    color: var(--color-foreground);
    padding: 7px 12px;
    font-size: 0.8rem;
    outline: none;
  }

  .add-row input:focus {
    border-color: var(--color-accent);
  }

  .btn-add {
    background: var(--color-accent);
    color: var(--color-background);
    border: none;
    border-radius: 6px;
    padding: 0 16px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
  }

  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding: 0 20px 16px;
    max-height: 180px;
    overflow-y: auto;
  }

  .tag {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: color-mix(in oklch, var(--color-foreground) 8%, transparent);
    border: 1px solid var(--color-separator);
    border-radius: 12px;
    padding: 4px 10px;
    font-size: 0.75rem;
    color: var(--color-foreground);
  }

  .tag-remove {
    background: none;
    border: none;
    color: var(--color-foreground);
    opacity: 0.5;
    cursor: pointer;
    font-size: 0.9rem;
    line-height: 1;
    padding: 0 2px;
  }

  .tag-remove:hover {
    opacity: 1;
    color: #ef4444;
  }

  .test-group {
    padding: 12px 20px;
    gap: 10px;
    background: color-mix(in oklch, var(--color-foreground) 4%, transparent);
    border-radius: 8px;
    margin: 0 20px;
  }

  .test-desc {
    font-size: 0.75rem;
    color: var(--color-foreground);
    opacity: 0.75;
  }

  .btn-test {
    background: color-mix(in oklch, var(--color-accent) 15%, transparent);
    border: 1px solid var(--color-accent);
    color: var(--color-accent);
    border-radius: 6px;
    padding: 8px 16px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-test:hover:not(:disabled) {
    background: var(--color-accent);
    color: var(--color-background);
  }

  .btn-test:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
