<script lang="ts">
  import { settings } from '$lib/stores/settings';
  import { setSetting, systemBridgeTestBreakLock } from '$lib/ipc';
  import SettingsToggle from '$lib/components/settings/SettingsToggle.svelte';

  let newFocusDomain = $state('');
  let newAdultDomain = $state('');
  let testingLock = $state(false);

  let focusDomains = $derived(
    ($settings.system_blocked_domains || '')
      .split(',')
      .map((s) => s.trim().toLowerCase())
      .filter(Boolean)
  );

  let adultDomains = $derived(
    ($settings.system_adult_domains || '')
      .split(',')
      .map((s) => s.trim().toLowerCase())
      .filter(Boolean)
  );

  async function toggle(key: string, current: boolean) {
    const updated = await setSetting(key, current ? 'false' : 'true');
    settings.set(updated);
  }

  function cleanDomain(input: string): string {
    let d = input.trim().toLowerCase();
    if (d.startsWith('https://')) d = d.slice(8);
    if (d.startsWith('http://')) d = d.slice(7);
    if (d.includes('/')) d = d.split('/')[0];
    if (d.includes(':')) d = d.split(':')[0];
    return d;
  }

  // --- Focus Domains Handlers ---
  async function saveFocusDomains(list: string[]) {
    const updated = await setSetting('system_blocked_domains', list.join(', '));
    settings.set(updated);
  }

  function addFocusDomain() {
    const d = cleanDomain(newFocusDomain);
    if (d && !focusDomains.includes(d)) {
      saveFocusDomains([...focusDomains, d]);
    }
    newFocusDomain = '';
  }

  function removeFocusDomain(domain: string) {
    saveFocusDomains(focusDomains.filter((d) => d !== domain));
  }

  function resetFocusDomains() {
    saveFocusDomains([
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

  // --- Adult Domains Handlers ---
  async function saveAdultDomains(list: string[]) {
    const updated = await setSetting('system_adult_domains', list.join(', '));
    settings.set(updated);
  }

  function addAdultDomain() {
    const d = cleanDomain(newAdultDomain);
    if (d && !adultDomains.includes(d)) {
      saveAdultDomains([...adultDomains, d]);
    }
    newAdultDomain = '';
  }

  function removeAdultDomain(domain: string) {
    saveAdultDomains(adultDomains.filter((d) => d !== domain));
  }

  function resetAdultDomains() {
    saveAdultDomains([
      'pornhub.com',
      'xvideos.com',
      'xnxx.com',
      'xhamster.com',
      'redtube.com',
      'youporn.com',
      'chaturbate.com',
      'onlyfans.com',
      'stripchat.com',
      'livejasmin.com',
      'cam4.com',
      'bongacams.com',
      'eporner.com',
      'spankbang.com',
      'tube8.com',
      'beeg.com',
      'kemono.party',
      'kemono.su',
      'coomer.party',
      'coomer.su',
      'faphouse.com',
      'brazzers.com',
      'bangbros.com',
      'naughtyamerica.com',
      'realitykings.com',
      'erome.com',
      'rule34.xxx',
      'nhentai.net',
      'hanime.tv'
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
  <!-- Protection Toggles -->
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
      description="Block explicit websites 24/7 via local network & browser tab filtering"
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

  <!-- Focus Session Blocklist -->
  <div class="group">
    <div class="group-header">
      <div class="group-title">Focus Blocklist ({focusDomains.length})</div>
      <button class="btn-link" onclick={resetFocusDomains}>Reset Defaults</button>
    </div>

    <div class="add-row">
      <input
        type="text"
        placeholder="Add focus domain (e.g. reddit.com)"
        bind:value={newFocusDomain}
        onkeydown={(e) => {
          if (e.key === 'Enter') addFocusDomain();
        }}
      />
      <button class="btn-add" onclick={addFocusDomain}>Add</button>
    </div>

    <div class="tags-container">
      {#each focusDomains as domain}
        <span class="tag">
          <span class="tag-name">{domain}</span>
          <button
            class="tag-remove"
            title="Remove {domain}"
            onclick={() => removeFocusDomain(domain)}
          >×</button>
        </span>
      {/each}
    </div>
  </div>

  <!-- 24/7 Adult Content Filter List -->
  <div class="group">
    <div class="group-header">
      <div class="group-title">24/7 Adult & Explicit Domains ({adultDomains.length})</div>
      <button class="btn-link" onclick={resetAdultDomains}>Reset Defaults</button>
    </div>

    <div class="add-row">
      <input
        type="text"
        placeholder="Add adult domain to block 24/7 (e.g. customsite.com)"
        bind:value={newAdultDomain}
        onkeydown={(e) => {
          if (e.key === 'Enter') addAdultDomain();
        }}
      />
      <button class="btn-add" onclick={addAdultDomain}>Add</button>
    </div>

    <div class="tags-container">
      {#each adultDomains as domain}
        <span class="tag tag-adult">
          <span class="tag-name">{domain}</span>
          <button
            class="tag-remove"
            title="Remove {domain}"
            onclick={() => removeAdultDomain(domain)}
          >×</button>
        </span>
      {/each}
    </div>
  </div>

  <!-- Test Lock Overlay -->
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
    gap: 16px;
    padding: 16px 20px 28px;
  }

  .group {
    display: flex;
    flex-direction: column;
    background: rgba(255, 255, 255, 0.03); backdrop-filter: blur(16px); -webkit-backdrop-filter: blur(16px);
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 10px;
    overflow: hidden;
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
    color: #ffffff;
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
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 6px;
    color: var(--color-foreground);
    padding: 7px 12px;
    font-size: 0.8rem;
    outline: none;
  }

  .add-row input:focus {
    border-color: #ffffff;
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
    max-height: 160px;
    overflow-y: auto;
  }

  .tag {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: color-mix(in oklch, var(--color-foreground) 8%, transparent);
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 12px;
    padding: 4px 10px;
    font-size: 0.75rem;
    color: var(--color-foreground);
  }

  .tag-adult {
    background: color-mix(in oklch, #ef4444 12%, transparent);
    border-color: rgba(239, 68, 68, 0.3);
    color: #fca5a5;
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
    padding: 14px 18px;
    gap: 10px;
    background: rgba(255, 255, 255, 0.03); backdrop-filter: blur(16px); -webkit-backdrop-filter: blur(16px);
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 10px;
    margin: 0;
  }

  .test-desc {
    font-size: 0.75rem;
    color: var(--color-foreground);
    opacity: 0.75;
  }

  .btn-test {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: #ffffff;
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
