<script lang="ts">
  import { Keyboard } from '@lucide/svelte';
  import Modal from '$lib/components/ui/Modal.svelte';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  let activeTab = $state<'navigation' | 'editor'>('navigation');

  const globalShortcuts = [
    { key: 'Ctrl + N', desc: 'Create new issue' },
    { key: 'Ctrl + K', desc: 'Close / Reopen current issue' },
    { key: 'Ctrl + L', desc: 'Focus issue search bar' },
    { key: 'Ctrl + ,', desc: 'Open Settings' },
    { key: 'Tab', desc: 'Toggle Open / Closed issues tab' },
    { key: '↑ / ↓', desc: 'Navigate issues in list' },
    { key: 'Enter', desc: 'Open highlighted issue' },
    { key: 'Ctrl + /', desc: 'Toggle keyboard shortcuts help' },
    { key: 'Esc', desc: 'Close modal / Back to list' }
  ];

  const editorShortcuts = [
    { key: 'Ctrl + B', desc: 'Bold' },
    { key: 'Ctrl + I', desc: 'Italic' },
    { key: 'Ctrl + K', desc: 'Insert link' },
    { key: 'Ctrl + E', desc: 'Inline code' },
    { key: 'Ctrl + Shift + C', desc: 'Code block' },
    { key: 'Ctrl + Shift + X', desc: 'Strikethrough' },
    { key: 'Ctrl + Shift + 8', desc: 'Unordered list' },
    { key: 'Ctrl + Shift + 7', desc: 'Ordered list' },
    { key: 'Ctrl + Shift + >', desc: 'Blockquote' },
    { key: 'Ctrl + Shift + P', desc: 'Toggle Write / Preview' },
    { key: 'Ctrl + Enter', desc: 'Submit / Save comment' },
    { key: 'Tab / Shift + Tab', desc: 'Indent / Outdent' }
  ];
</script>

<Modal {isOpen} {onClose} title="Keyboard Shortcuts" icon={Keyboard}>
  <div class="flex border-b border-gh-border px-4 bg-gh-subtle gap-4">
    <button
      type="button"
      onclick={() => (activeTab = 'navigation')}
      class="py-2 text-xs font-semibold cursor-pointer border-b-2 transition-colors {activeTab === 'navigation' ? 'border-gh-link text-gh-text' : 'border-transparent text-gh-muted hover:text-gh-text'}"
    >
      Navigation
    </button>
    <button
      type="button"
      onclick={() => (activeTab = 'editor')}
      class="py-2 text-xs font-semibold cursor-pointer border-b-2 transition-colors {activeTab === 'editor' ? 'border-gh-link text-gh-text' : 'border-transparent text-gh-muted hover:text-gh-text'}"
    >
      Editor
    </button>
  </div>

  <div class="p-4 divide-y divide-gh-border max-h-[60vh] overflow-y-auto">
    {#if activeTab === 'navigation'}
      {#each globalShortcuts as item}
        <div class="py-2 flex items-center justify-between text-xs">
          <span class="text-gh-muted">{item.desc}</span>
          <kbd class="px-2 py-1 bg-gh-subtle border border-gh-border rounded font-mono font-semibold text-gh-text shadow-xs">
            {item.key}
          </kbd>
        </div>
      {/each}
    {:else}
      {#each editorShortcuts as item}
        <div class="py-2 flex items-center justify-between text-xs">
          <span class="text-gh-muted">{item.desc}</span>
          <kbd class="px-2 py-1 bg-gh-subtle border border-gh-border rounded font-mono font-semibold text-gh-text shadow-xs">
            {item.key}
          </kbd>
        </div>
      {/each}
    {/if}
  </div>

  {#snippet footer()}
    <button
      onclick={onClose}
      class="bg-gh-green-btn hover:bg-gh-green text-white text-xs font-semibold px-3 py-1.5 rounded-md cursor-pointer"
    >
      Got it
    </button>
  {/snippet}
</Modal>