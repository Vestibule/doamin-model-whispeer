<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { orchestrate, type OrchestrateResult } from "./lib/tauri";
  import SpeechInput from "./lib/SpeechInput.svelte";
  import AudioDeviceSelector from "./lib/AudioDeviceSelector.svelte";

  let greetMsg = $state("");
  let name = $state("");
  let transcript = $state("");
  let result = $state<OrchestrateResult | null>(null);
  let loading = $state(false);
  let error = $state("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }

  async function handleOrchestrate() {
    if (!transcript.trim()) {
      error = "Please enter a transcript";
      return;
    }

    loading = true;
    error = "";
    result = null;

    try {
      result = await orchestrate(transcript);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<main class="container">
  <h1>Domain Model Note Taking</h1>

  <div class="orchestrate-section">
    <h2>Orchestrate Transcript</h2>
    
    <AudioDeviceSelector />
    
    <form onsubmit={(e) => { e.preventDefault(); handleOrchestrate(); }}>
      <div class="textarea-with-mic">
        <textarea
          bind:value={transcript}
          placeholder="Enter your transcript here or click the microphone..."
          rows="5"
          disabled={loading}
        ></textarea>
        <div class="mic-container">
          <SpeechInput bind:value={transcript} />
        </div>
      </div>
      <button type="submit" disabled={loading}>
        {loading ? "Processing..." : "Orchestrate"}
      </button>
    </form>

    {#if error}
      <div class="error">
        <strong>Error:</strong> {error}
      </div>
    {/if}

    {#if result}
      <div class="result">
        <h3>Results</h3>
        
        <div class="result-section">
          <h4>Domain Model</h4>
          <pre>{JSON.stringify(result.model, null, 2)}</pre>
        </div>

        <div class="result-section">
          <h4>Mermaid Diagram</h4>
          <pre>{result.mermaid}</pre>
        </div>

        <div class="result-section">
          <h4>Markdown Documentation</h4>
          <pre>{result.markdown}</pre>
        </div>
      </div>
    {/if}
  </div>

  <hr />

  <div class="greet-section">
    <h2>Test Greet Command</h2>
    <form class="row" onsubmit={(e) => { e.preventDefault(); greet(); }}>
      <input id="greet-input" bind:value={name} placeholder="Enter a name..." />
      <button type="submit">Greet</button>
    </form>
    <p>{greetMsg}</p>
  </div>
</main>

<style>
.orchestrate-section,
.greet-section {
  margin: 2rem 0;
  width: 100%;
  max-width: 800px;
}

.textarea-with-mic {
  position: relative;
  margin-bottom: 1rem;
}

textarea {
  width: 100%;
  padding: 0.8em;
  padding-right: 60px; /* Space for microphone button */
  border-radius: 8px;
  border: 1px solid #ccc;
  font-family: inherit;
  font-size: 1em;
  resize: vertical;
}

.mic-container {
  position: absolute;
  right: 8px;
  top: 8px;
}

textarea:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error {
  background-color: #fee;
  border: 1px solid #fcc;
  border-radius: 8px;
  padding: 1rem;
  margin: 1rem 0;
  color: #c00;
}

.result {
  margin-top: 2rem;
  text-align: left;
}

.result-section {
  margin: 1.5rem 0;
  background-color: #f9f9f9;
  border-radius: 8px;
  padding: 1rem;
}

.result-section h4 {
  margin-top: 0;
  margin-bottom: 0.5rem;
}

.result-section pre {
  background-color: #fff;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 1rem;
  overflow-x: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
}

hr {
  border: none;
  border-top: 1px solid #ddd;
  margin: 2rem 0;
}

:global(:root) {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

:global(.container) {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

:global(.logo) {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

:global(.logo.tauri:hover) {
  filter: drop-shadow(0 0 2em #24c8db);
}

:global(.row) {
  display: flex;
  justify-content: center;
}

:global(a) {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

:global(a:hover) {
  color: #535bf2;
}

:global(h1) {
  text-align: center;
}

:global(input),
:global(button) {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

:global(button) {
  cursor: pointer;
}

:global(button:hover) {
  border-color: #396cd8;
}
:global(button:active) {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

:global(input),
:global(button) {
  outline: none;
}

:global(#greet-input) {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :global(:root) {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  :global(a:hover) {
    color: #24c8db;
  }

  :global(input),
  :global(button) {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  :global(button:active) {
    background-color: #0f0f0f69;
  }

  textarea {
    color: #ffffff;
    background-color: #0f0f0f98;
    border-color: #555;
  }

  .error {
    background-color: #4a1f1f;
    border-color: #8b3a3a;
    color: #ffb3b3;
  }

  .result-section {
    background-color: #1a1a1a;
  }

  .result-section pre {
    background-color: #0f0f0f;
    border-color: #444;
    color: #f6f6f6;
  }
}
</style>
