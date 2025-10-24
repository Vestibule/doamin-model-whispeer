<script lang="ts">
  import { listAudioDevices, setAudioDevice, type AudioDevice } from './tauri';
  import { Select, Button, Alert, Label, Spinner } from 'flowbite-svelte';
  import { RefreshOutline } from 'flowbite-svelte-icons';

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
  <div class="flex items-center gap-3 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg mb-4 flex-wrap border border-blue-200 dark:border-blue-800">
    <Label class="font-semibold whitespace-nowrap text-blue-900 dark:text-blue-100">
      🎤 Audio Input:
    </Label>
    
    {#if loading}
      <div class="flex items-center gap-2">
        <Spinner size="4" color="blue" />
        <span class="text-sm text-blue-700 dark:text-blue-300 italic">Loading devices...</span>
      </div>
    {:else if devices.length > 0}
      <Select
        id="audio-device"
        bind:value={selectedDevice}
        on:change={handleDeviceChange}
        class="flex-1 min-w-[200px]"
      >
        {#each devices as device}
          <option value={device.name}>
            {device.name} {device.is_default ? '(default)' : ''}
          </option>
        {/each}
      </Select>
    {:else}
      <span class="text-sm text-blue-700 dark:text-blue-300 italic">No audio devices found</span>
    {/if}

    <Button 
      size="sm"
      color="blue"
      onclick={loadDevices}
      disabled={loading}
      title="Refresh device list"
      class="!p-2"
    >
      <RefreshOutline class="w-4 h-4" />
    </Button>

    {#if error}
      <Alert color="red" class="w-full mt-2" border>
        {error}
      </Alert>
    {/if}
  </div>
{/if}

