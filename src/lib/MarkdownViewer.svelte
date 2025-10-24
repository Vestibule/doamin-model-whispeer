<script lang="ts">
  import { marked } from 'marked';

  interface Props {
    content: string;
  }

  let { content }: Props = $props();

  // Configure marked for better rendering
  marked.setOptions({
    breaks: true,
    gfm: true,
  });

  let html = $derived(content ? marked(content) : '<p class="text-gray-400 dark:text-gray-500 italic">No content yet. Start recording to generate domain model documentation...</p>');
</script>

<div class="prose prose-sm md:prose-base lg:prose-lg dark:prose-invert max-w-none p-6 overflow-auto h-full">
  {@html html}
</div>

<style>
  :global(.prose) {
    @apply text-gray-900 dark:text-gray-100;
  }
  
  :global(.prose h1) {
    @apply text-3xl font-bold mb-4 text-gray-900 dark:text-white;
  }
  
  :global(.prose h2) {
    @apply text-2xl font-bold mb-3 mt-6 text-gray-900 dark:text-white;
  }
  
  :global(.prose h3) {
    @apply text-xl font-semibold mb-2 mt-4 text-gray-900 dark:text-white;
  }
  
  :global(.prose p) {
    @apply mb-4 leading-relaxed;
  }
  
  :global(.prose ul, .prose ol) {
    @apply mb-4 ml-6;
  }
  
  :global(.prose li) {
    @apply mb-2;
  }
  
  :global(.prose code) {
    @apply bg-gray-100 dark:bg-gray-800 px-1.5 py-0.5 rounded text-sm font-mono;
  }
  
  :global(.prose pre) {
    @apply bg-gray-100 dark:bg-gray-800 p-4 rounded-lg overflow-x-auto mb-4;
  }
  
  :global(.prose pre code) {
    @apply bg-transparent p-0;
  }
  
  :global(.prose blockquote) {
    @apply border-l-4 border-blue-500 pl-4 italic my-4;
  }
  
  :global(.prose table) {
    @apply w-full border-collapse mb-4;
  }
  
  :global(.prose th) {
    @apply bg-gray-100 dark:bg-gray-800 border border-gray-300 dark:border-gray-700 px-4 py-2 text-left font-semibold;
  }
  
  :global(.prose td) {
    @apply border border-gray-300 dark:border-gray-700 px-4 py-2;
  }
</style>
