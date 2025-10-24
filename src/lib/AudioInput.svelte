<script lang="ts">
  import { startRecording, stopRecording } from './tauri';
  import { listen } from '@tauri-apps/api/event';
  import type { TranscriptionResult } from './tauri';
  import { Input, Spinner } from 'flowbite-svelte';
  import { MicrophoneSolid, PenSolid, CheckCircleSolid } from 'flowbite-svelte-icons';

  interface Props {
    value?: string;
    onSubmit?: (text: string) => void;
  }

  let { value = $bindable(""), onSubmit }: Props = $props();

  let isRecording = $state(false);
  let error = $state("");
  let status = $state("idle");
  let isTauri = $state(false);
  let isSpacebarPressed = $state(false);
  let isEditMode = $state(false);

  // Check if running in Tauri environment
  $effect(() => {
    if (typeof window !== 'undefined') {
      isTauri = '__TAURI_INTERNALS__' in window;
      if (!isTauri) {
        error = "This feature requires running in Tauri app. Use: pnpm tauri dev";
      }
    }
  });

  // Listen for transcription results from backend
  $effect(() => {
    if (!isTauri) return;
    console.log('[AudioInput] Setting up event listeners...');
    
    const unlisten = listen<TranscriptionResult>('transcription-result', (event) => {
      console.log('[AudioInput] Received transcription-result:', event.payload);
      const result = event.payload;
      // Append to existing value with a space
      value += (value ? " " : "") + result.text;
    });

    const unlistenError = listen<string>('transcription-error', (event) => {
      console.error('[AudioInput] Received transcription-error:', event.payload);
      error = `Transcription error: ${event.payload}`;
    });

    const unlistenState = listen<string>('recording-state-changed', (event) => {
      console.log('[AudioInput] Received recording-state-changed:', event.payload);
      status = event.payload;
      if (event.payload === 'idle') {
        isRecording = false;
      }
    });

    return () => {
      unlisten.then(fn => fn());
      unlistenError.then(fn => fn());
      unlistenState.then(fn => fn());
    };
  });

  // Keyboard event handlers for push-to-talk with spacebar and edit mode toggle
  $effect(() => {
    if (!isTauri) return;

    const handleKeyDown = async (event: KeyboardEvent) => {
      // Handle CMD+E to toggle edit mode
      if (event.metaKey && event.code === 'KeyE') {
        event.preventDefault();
        isEditMode = !isEditMode;
        return;
      }

      // Handle Enter key to submit
      if (event.code === 'Enter' && !isRecording && value.trim()) {
        event.preventDefault();
        if (onSubmit) {
          onSubmit(value);
        }
        return;
      }

      // Only trigger on spacebar and ignore if already pressed (prevents key repeat)
      if (event.code === 'Space' && !isSpacebarPressed && !isRecording && !isEditMode) {
        // Ignore if user is typing in the input field
        const target = event.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
          return;
        }
        
        event.preventDefault();
        isSpacebarPressed = true;
        error = "";
        
        try {
          console.log('[AudioInput] Spacebar pressed - starting recording...');
          await startRecording();
          isRecording = true;
        } catch (e) {
          console.error('[AudioInput] Error starting recording:', e);
          error = String(e);
          isSpacebarPressed = false;
        }
      }
    };

    const handleKeyUp = async (event: KeyboardEvent) => {
      // Only trigger on spacebar release
      if (event.code === 'Space' && isSpacebarPressed && isRecording) {
        // Ignore if user is typing in the input field
        const target = event.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
          return;
        }
        
        event.preventDefault();
        isSpacebarPressed = false;
        
        try {
          console.log('[AudioInput] Spacebar released - stopping recording...');
          await stopRecording();
          isRecording = false;
        } catch (e) {
          console.error('[AudioInput] Error stopping recording:', e);
          error = String(e);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  });
</script>

<div class="flex flex-col gap-3">
  <div class="relative flex-1">
    <textarea
      bind:value
      placeholder={isEditMode ? "Edit your text..." : "Hold spacebar to record, then press Enter to process..."}
      readonly={!isEditMode}
      class="w-full h-32 p-4 text-base border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 resize-none overflow-y-auto focus:outline-none focus:ring-2 {isRecording ? 'ring-2 ring-red-500 border-red-500' : ''} {isEditMode ? 'ring-2 ring-blue-500 border-blue-500' : ''} {!isEditMode ? 'cursor-default' : ''}"
    ></textarea>
  </div>
  
  <div class="flex items-center gap-3">
    <div class="flex items-center gap-2">
      {#if isRecording}
        <div class="flex items-center gap-2 px-3 py-2 bg-red-100 dark:bg-red-900 rounded-lg">
          <div class="w-3 h-3 bg-red-600 rounded-full animate-pulse"></div>
          <span class="text-sm font-medium text-red-600 dark:text-red-400">Recording...</span>
        </div>
      {:else if status === 'processing'}
        <div class="flex items-center gap-2 px-3 py-2 bg-blue-100 dark:bg-blue-900 rounded-lg">
          <Spinner size="4" color="blue" />
          <span class="text-sm font-medium text-blue-600 dark:text-blue-400">Processing...</span>
        </div>
      {:else if isEditMode}
        <div class="flex items-center gap-2 text-blue-600 dark:text-blue-400">
          <PenSolid class="w-5 h-5" />
          <span class="text-sm">Edit mode</span>
        </div>
      {:else if isTauri}
        <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
          <MicrophoneSolid class="w-5 h-5" />
          <span class="text-sm">Ready to record</span>
        </div>
      {/if}
    </div>
    
    <div class="flex gap-2 ml-auto">
      <!-- Toggle edit mode button -->
      <button
        type="button"
        title={isEditMode ? "Exit edit mode (Cmd+E)" : "Enter edit mode (Cmd+E)"}
        class="p-2 rounded-lg transition-colors {isEditMode ? 'text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20 hover:bg-blue-100 dark:hover:bg-blue-900/40' : 'text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'}"
        onclick={() => isEditMode = !isEditMode}
      >
        <PenSolid class="w-5 h-5" />
      </button>
      
      <!-- Submit button -->
      {#if value.trim()}
        <button
          type="button"
          title="Process text (Enter)"
          class="p-2 rounded-lg text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-900/20 hover:bg-green-100 dark:hover:bg-green-900/40 transition-colors"
          onclick={() => {
            if (onSubmit) {
              onSubmit(value);
            }
          }}
        >
          <CheckCircleSolid class="w-5 h-5" />
        </button>
      {/if}
    </div>
  </div>
</div>

{#if error}
  <div class="mt-2 text-sm text-red-600 dark:text-red-400">
    {error}
  </div>
{/if}
