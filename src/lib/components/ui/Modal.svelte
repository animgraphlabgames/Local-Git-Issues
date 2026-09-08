<script lang="ts">
  import type { Component, Snippet } from 'svelte';
  import { X } from '@lucide/svelte';

  let {
    isOpen,
    title,
    icon: Icon,
    maxWidth = 'max-w-md',
    onClose,
    children,
    footer
  }: {
    isOpen: boolean;
    title: string;
    icon?: Component;
    maxWidth?: string;
    onClose: () => void;
    children: Snippet;
    footer?: Snippet;
  } = $props();
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
    role="presentation"
  >
    <div class="bg-white border border-gh-border rounded-lg shadow-xl {maxWidth} w-full overflow-hidden flex flex-col animate-in fade-in zoom-in-95 duration-100 max-h-[85vh]">
      <div class="bg-gh-subtle border-b border-gh-border px-4 py-3 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-2 font-semibold text-sm text-gh-text">
          {#if Icon}
            <Icon class="w-4 h-4" />
          {/if}
          <span>{title}</span>
        </div>
        <button onclick={onClose} class="text-gh-muted hover:text-gh-text cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto">
        {@render children()}
      </div>

      {#if footer}
        <div class="bg-gh-subtle border-t border-gh-border px-4 py-2.5 text-right shrink-0">
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}