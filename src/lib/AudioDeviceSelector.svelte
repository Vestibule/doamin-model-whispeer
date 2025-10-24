<script lang="ts">
  import { listAudioDevices, setAudioDevice, type AudioDevice } from './tauri';

  let devices = $state<AudioDevice[]>([]);
  let selectedDevice = $state<string>("");
  let loading = $state(false);
  let error = $state("");
  let isTauri = $state(false);

  // Check if running in Tauri
  $effect(() => {
    if (typeof window !== 'undefined') {
      isTauri = '__TAURI_INTERNALS__' in window;
      if (isTauri) {
        loadDevices();
      }
    }
  });

  async function loadDevices() {
    try {
      loading = true;
      error = "";
      devices = await listAudioDevices();
      
      // Set default device as selected
      const defaultDevice = devices.find(d => d.is_default);
      if (defaultDevice) {
        selectedDevice = defaultDevice.name;
      }
    } catch (e) {
      error = `Failed to load audio devices: ${e}`;
      console.error('[AudioDeviceSelector] Error loading devices:', e);
    } finally {
      loading = false;
    }
  }

  async function handleDeviceChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const deviceName = target.value;
    
    try {
      error = "";
      const message = await setAudioDevice(deviceName);
      console.log('[AudioDeviceSelector]', message);
      selectedDevice = deviceName;
    } catch (e) {
      error = `Failed to set audio device: ${e}`;
      console.error('[AudioDeviceSelector] Error setting device:', e);
    }
  }
</script>

{#if isTauri}
  <div class="device-selector">
    <label for="audio-device">
      🎤 Audio Input:
    </label>
    
    {#if loading}
      <span class="loading">Loading devices...</span>
    {:else if devices.length > 0}
      <select 
        id="audio-device"
        value={selectedDevice}
        onchange={handleDeviceChange}
      >
        {#each devices as device}
          <option value={device.name}>
            {device.name} {device.is_default ? '(default)' : ''}
          </option>
        {/each}
      </select>
    {:else}
      <span class="no-devices">No audio devices found</span>
    {/if}

    <button 
      type="button"
      onclick={loadDevices}
      disabled={loading}
      title="Refresh device list"
    >
      🔄
    </button>

    {#if error}
      <div class="error">{error}</div>
    {/if}
  </div>
{/if}

<style>
  .device-selector {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    background-color: #f5f5f5;
    border-radius: 4px;
    font-size: 0.875rem;
    flex-wrap: wrap;
  }

  label {
    font-weight: 500;
    white-space: nowrap;
  }

  select {
    flex: 1;
    min-width: 200px;
    padding: 0.4rem 0.8rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    background-color: white;
    cursor: pointer;
    font-size: 0.875rem;
  }

  select:focus {
    outline: none;
    border-color: #396cd8;
  }

  button {
    padding: 0.4rem 0.8rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    background-color: white;
    cursor: pointer;
    font-size: 1rem;
    transition: all 0.2s;
  }

  button:hover:not(:disabled) {
    border-color: #396cd8;
    transform: scale(1.05);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .loading,
  .no-devices {
    color: #666;
    font-style: italic;
  }

  .error {
    width: 100%;
    padding: 0.5rem;
    background-color: #fee;
    border: 1px solid #fcc;
    border-radius: 4px;
    color: #c00;
    font-size: 0.875rem;
  }

  @media (prefers-color-scheme: dark) {
    .device-selector {
      background-color: #2a2a2a;
    }

    select,
    button {
      background-color: #1a1a1a;
      border-color: #555;
      color: #f6f6f6;
    }

    .loading,
    .no-devices {
      color: #999;
    }

    .error {
      background-color: #4a1f1f;
      border-color: #8b3a3a;
      color: #ffb3b3;
    }
  }
</style>
