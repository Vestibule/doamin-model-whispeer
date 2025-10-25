<script lang="ts">
  import { orchestrate, type OrchestrateResult } from "./lib/tauri";
  import AudioInput from "./lib/AudioInput.svelte";
  import MarkdownViewer from "./lib/MarkdownViewer.svelte";
  import DomainModelInterview from "./lib/DomainModelInterview.svelte";
  import { Spinner, Button, ButtonGroup } from 'flowbite-svelte';
  import { FileCodeOutline, MessageDotsOutline } from 'flowbite-svelte-icons';

  let transcript = $state("");
  let markdown = $state("");
  let loading = $state(false);
  let error = $state("");
  let mode = $state<"transcript" | "interview">("interview"); // Default to interview mode

  async function handleSubmit(text: string) {
    if (!text.trim()) {
      error = "Please provide a transcript";
      return;
    }

    loading = true;
    error = "";

    try {
      const result = await orchestrate(text);
      markdown = result.markdown;
      // Clear the transcript after successful processing
      transcript = "";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<main class="h-screen w-screen flex flex-col overflow-hidden bg-gray-50 dark:bg-gray-900">
  <!-- Top section: Mode selector and Audio Input (only in transcript mode) -->
  <div class="flex-none border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-sm">
    <div class="px-6 py-4">
      <!-- Mode Selector -->
      <div class="flex items-center justify-between mb-4">
        <h1 class="text-xl font-semibold text-gray-900 dark:text-gray-100">
          Domain Model Note Taking
        </h1>
        <ButtonGroup>
          <Button 
            color={mode === "interview" ? "blue" : "light"}
            onclick={() => mode = "interview"}
          >
            <MessageDotsOutline class="w-4 h-4 mr-2" />
            Interview guidée
          </Button>
          <Button 
            color={mode === "transcript" ? "blue" : "light"}
            onclick={() => mode = "transcript"}
          >
            <FileCodeOutline class="w-4 h-4 mr-2" />
            Transcript libre
          </Button>
        </ButtonGroup>
      </div>

      {#if mode === "transcript"}
        <AudioInput bind:value={transcript} onSubmit={handleSubmit} />
        
        {#if loading}
          <div class="mt-3 flex items-center gap-2 text-blue-600 dark:text-blue-400">
            <Spinner size="4" />
            <span class="text-sm font-medium">Generating domain model documentation...</span>
          </div>
        {/if}
        
        {#if error}
          <div class="mt-3 text-sm text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 px-4 py-2 rounded-lg border border-red-200 dark:border-red-800">
            <span class="font-semibold">Error:</span> {error}
          </div>
        {/if}
      {/if}
    </div>
  </div>

  <!-- Bottom section: Content based on mode -->
  <div class="flex-1 overflow-hidden bg-white dark:bg-gray-800">
    {#if mode === "interview"}
      <DomainModelInterview />
    {:else}
      <MarkdownViewer content={markdown} />
    {/if}
  </div>
</main>

