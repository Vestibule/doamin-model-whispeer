<script lang="ts">
  import { startRecording, stopRecording } from './tauri';
  import { listen } from '@tauri-apps/api/event';
  import type { Snippet } from "svelte";
  import type { TranscriptionResult } from './tauri';

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

<div class="speech-input">
  <button
    type="button"
    class="mic-button"
    class:recording={isRecording}
    class:processing={status === 'processing'}
    onclick={toggleRecording}
    title={!isTauri ? "Requires Tauri app" : isRecording ? "Stop recording" : "Start recording"}
    disabled={!isTauri || status === 'processing'}
  >
    {#if status === 'processing'}
      ⏳
    {:else if isRecording}
      🔴
    {:else}
      🎤
    {/if}
  </button>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if status === 'processing'}
    <div class="status-message">Processing audio...</div>
  {/if}

  {#if children}
    {@render children()}
  {/if}
</div>

<style>
  .speech-input {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .mic-button {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    border: 2px solid #ccc;
    background-color: white;
    cursor: pointer;
    font-size: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.3s ease;
  }

  .mic-button:hover {
    border-color: #396cd8;
    transform: scale(1.05);
  }

  .mic-button.recording {
    background-color: #fee;
    border-color: #f00;
    animation: pulse 1.5s infinite;
  }

  .mic-button.processing {
    background-color: #ffa;
    border-color: #fa0;
    opacity: 0.7;
    cursor: not-allowed;
  }

  @keyframes pulse {
    0%, 100% {
      box-shadow: 0 0 0 0 rgba(255, 0, 0, 0.7);
    }
    50% {
      box-shadow: 0 0 0 10px rgba(255, 0, 0, 0);
    }
  }

  .status-message {
    padding: 0.5rem 1rem;
    background-color: #e0f7ff;
    border: 1px solid #17a2b8;
    border-radius: 4px;
    color: #0c5460;
    font-size: 0.875rem;
  }

  .error-message {
    padding: 0.5rem 1rem;
    background-color: #fee;
    border: 1px solid #fcc;
    border-radius: 4px;
    color: #c00;
    font-size: 0.875rem;
  }

  @media (prefers-color-scheme: dark) {
    .mic-button {
      background-color: #1a1a1a;
      border-color: #555;
    }

    .mic-button.recording {
      background-color: #4a1f1f;
      border-color: #f00;
    }

    .mic-button.processing {
      background-color: #3d3d1a;
      border-color: #fa0;
    }

    .status-message {
      background-color: #1a3d3d;
      border-color: #17a2b8;
      color: #7dd3e0;
    }

    .error-message {
      background-color: #4a1f1f;
      border-color: #8b3a3a;
      color: #ffb3b3;
    }
  }
</style>
