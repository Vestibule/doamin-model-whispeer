<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { orchestrate, type OrchestrateResult } from "./lib/tauri";
  import SpeechInput from "./lib/SpeechInput.svelte";
  import AudioDeviceSelector from "./lib/AudioDeviceSelector.svelte";
  import { Card, Button, Textarea, Input, Alert, Heading, Hr } from 'flowbite-svelte';

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

<main class="min-h-screen bg-gray-50 dark:bg-gray-900">
  <div class="container mx-auto px-4 py-8 max-w-6xl">
    <Heading tag="h1" class="text-center mb-8 text-gray-900 dark:text-white">Domain Model Note Taking</Heading>

    <div class="mb-8">
      <Card size="xl" class="w-full">
        <Heading tag="h2" class="mb-4">Orchestrate Transcript</Heading>
      
        <AudioDeviceSelector />
        
        <form onsubmit={(e) => { e.preventDefault(); handleOrchestrate(); }} class="space-y-4">
          <div class="relative">
            <Textarea
              bind:value={transcript}
              placeholder="Enter your transcript here or click the microphone..."
              rows="8"
              disabled={loading}
              class="pr-16"
            />
            <div class="absolute right-2 top-2">
              <SpeechInput bind:value={transcript} />
            </div>
          </div>
          <Button type="submit" disabled={loading} color="blue" size="lg" class="w-full">
            {loading ? "Processing..." : "Orchestrate"}
          </Button>
        </form>

        {#if error}
          <Alert color="red" class="mt-4" border>
            <span class="font-medium">Error:</span> {error}
          </Alert>
        {/if}

        {#if result}
          <div class="mt-6 space-y-4">
            <Heading tag="h3" class="text-lg">Results</Heading>
            
            <Card size="lg">
              <Heading tag="h4" class="mb-3 text-base">Domain Model</Heading>
              <pre class="bg-gray-100 dark:bg-gray-800 p-4 rounded-lg overflow-x-auto text-sm font-mono border border-gray-200 dark:border-gray-700">{JSON.stringify(result.model, null, 2)}</pre>
            </Card>

            <Card size="lg">
              <Heading tag="h4" class="mb-3 text-base">Mermaid Diagram</Heading>
              <pre class="bg-gray-100 dark:bg-gray-800 p-4 rounded-lg overflow-x-auto text-sm font-mono border border-gray-200 dark:border-gray-700">{result.mermaid}</pre>
            </Card>

            <Card size="lg">
              <Heading tag="h4" class="mb-3 text-base">Markdown Documentation</Heading>
              <pre class="bg-gray-100 dark:bg-gray-800 p-4 rounded-lg overflow-x-auto text-sm font-mono border border-gray-200 dark:border-gray-700 whitespace-pre-wrap">{result.markdown}</pre>
            </Card>
          </div>
        {/if}
      </Card>
    </div>

    <Hr class="my-8" />

    <Card size="xl" class="w-full">
      <Heading tag="h2" class="mb-4">Test Greet Command</Heading>
      <form onsubmit={(e) => { e.preventDefault(); greet(); }} class="space-y-4">
        <Input bind:value={name} placeholder="Enter a name..." size="lg" />
        <Button type="submit" color="blue">Greet</Button>
      </form>
      {#if greetMsg}
        <Alert color="green" class="mt-4">{greetMsg}</Alert>
      {/if}
    </Card>
  </div>
</main>

