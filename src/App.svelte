<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let greetMsg = "";
  let name = "";

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="container">
  <h1>Welcome to Tauri + Svelte</h1>

  <div class="row">
    <a href="https://vite.dev" target="_blank">
      <img src="/vite.svg" class="logo vite" alt="Vite logo" />
    </a>
    <a href="https://tauri.app" target="_blank">
      <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
    </a>
    <a href="https://svelte.dev" target="_blank">
      <img src="https://svelte.dev/svelte-logo.svg" class="logo svelte" alt="Svelte logo" />
    </a>
  </div>
  <p>Click on the Tauri, Vite, and Svelte logos to learn more.</p>

  <form class="row" on:submit|preventDefault={greet}>
    <input id="greet-input" bind:value={name} placeholder="Enter a name..." />
    <button type="submit">Greet</button>
  </form>
  <p>{greetMsg}</p>
</main>

<style>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.svelte:hover {
  filter: drop-shadow(0 0 2em #ff3e00);
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
}
</style>
