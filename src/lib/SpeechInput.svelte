<script lang="ts">
  import { startRecording, stopRecording } from './tauri';
  import { listen } from '@tauri-apps/api/event';
  import type { Snippet } from "svelte";
  import type { TranscriptionResult } from './tauri';
  import { Button, Spinner } from 'flowbite-svelte';
  import { MicrophoneSolid } from 'flowbite-svelte-icons';

  interface Props {
    value?: string;
    onTranscript?: (text: string) => void;
    placeholder?: string;
    children?: Snippet;
  }

  let { value = $bindable(""), onTranscript, placeholder = "Click microphone to speak...", children }: Props = $props();

  let isRecording = $state(false);
  let error = $state("");
  let status = $state("idle");
  let isTauri = $state(false);
  let isSpacebarPressed = $state(false);

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
    console.log('[SpeechInput] Setting up event listeners...');
    
    const unlisten = listen<TranscriptionResult>('transcription-result', (event) => {
      console.log('[SpeechInput] Received transcription-result:', event.payload);
      const result = event.payload;
      value += result.text + " ";
      if (onTranscript) {
        onTranscript(value);
      }
    });

    const unlistenError = listen<string>('transcription-error', (event) => {
      console.error('[SpeechInput] Received transcription-error:', event.payload);
      error = `Transcription error: ${event.payload}`;
    });

    const unlistenState = listen<string>('recording-state-changed', (event) => {
      console.log('[SpeechInput] Received recording-state-changed:', event.payload);
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

  // Keyboard event handlers for push-to-talk with spacebar
  $effect(() => {
    if (!isTauri) return;

    const handleKeyDown = async (event: KeyboardEvent) => {
      // Only trigger on spacebar and ignore if already pressed (prevents key repeat)
      if (event.code === 'Space' && !isSpacebarPressed && !isRecording) {
        // Ignore if user is typing in an input field
        const target = event.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
          return;
        }
        
        event.preventDefault();
        isSpacebarPressed = true;
        error = "";
        
        try {
          console.log('[SpeechInput] Spacebar pressed - starting recording...');
          await startRecording();
          isRecording = true;
        } catch (e) {
          console.error('[SpeechInput] Error starting recording:', e);
          error = String(e);
          isSpacebarPressed = false;
        }
      }
    };

    const handleKeyUp = async (event: KeyboardEvent) => {
      // Only trigger on spacebar release
      if (event.code === 'Space' && isSpacebarPressed && isRecording) {
        // Ignore if user is typing in an input field
        const target = event.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
          return;
        }
        
        event.preventDefault();
        isSpacebarPressed = false;
        
        try {
          console.log('[SpeechInput] Spacebar released - stopping recording...');
          await stopRecording();
          isRecording = false;
        } catch (e) {
          console.error('[SpeechInput] Error stopping recording:', e);
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

  async function toggleRecording() {
    console.log('[SpeechInput] toggleRecording called, isTauri:', isTauri, 'isRecording:', isRecording);
    
    if (!isTauri) {
      error = "This feature requires running in Tauri app. Use: pnpm tauri dev";
      return;
    }
    
    if (isRecording) {
      try {
        console.log('[SpeechInput] Stopping recording...');
        await stopRecording();
        isRecording = false;
        console.log('[SpeechInput] Recording stopped');
      } catch (e) {
        console.error('[SpeechInput] Error stopping recording:', e);
        error = String(e);
      }
    } else {
      error = "";
      try {
        console.log('[SpeechInput] Starting recording...');
        await startRecording();
        isRecording = true;
        console.log('[SpeechInput] Recording started');
      } catch (e) {
        console.error('[SpeechInput] Error starting recording:', e);
        error = String(e);
      }
    }
  }
</script>

<div class="flex flex-col gap-2">
  <div class="flex items-center gap-2">
    <Button
      type="button"
      pill
      size="lg"
      color={isRecording ? "red" : "blue"}
      class="{isRecording ? 'animate-pulse shadow-lg' : ''} {status === 'processing' ? 'opacity-70' : ''} transition-all"
      onclick={toggleRecording}
      title={!isTauri ? "Requires Tauri app" : isRecording ? "Stop recording" : "Start recording"}
      disabled={!isTauri || status === 'processing'}
    >
      {#if status === 'processing'}
        <Spinner size="6" color="white" />
      {:else}
        <MicrophoneSolid class="w-5 h-5" />
      {/if}
    </Button>

    {#if isTauri && !isRecording && status !== 'processing'}
      <span class="text-sm text-gray-500 dark:text-gray-400">
        Hold <kbd class="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 border border-gray-200 rounded-lg dark:bg-gray-600 dark:text-gray-100 dark:border-gray-500">Space</kbd> to record
      </span>
    {/if}

    {#if isRecording}
      <span class="text-sm font-medium text-red-600 dark:text-red-400 animate-pulse">
        Recording... (release Space to transcribe)
      </span>
    {/if}
  </div>

  {#if error}
    <Alert color="red" class="max-w-md" border>
      {error}
    </Alert>
  {/if}

  {#if status === 'processing'}
    <Alert color="blue" class="max-w-md" border>
      <span class="flex items-center gap-2">
        <Spinner size="4" color="blue" />
        Processing audio...
      </span>
    </Alert>
  {/if}

  {#if children}
    {@render children()}
  {/if}
</div>

