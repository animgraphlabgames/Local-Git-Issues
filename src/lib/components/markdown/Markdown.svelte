<script lang="ts">
  import { marked } from 'marked';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { X } from '@lucide/svelte';
  import { resolveAttachmentsInMarkdown } from '$lib/utils/attachments';

  let { content }: { content: string } = $props();

  marked.setOptions({
    gfm: true,
    breaks: true
  });

  let resolvedContent = $state('');
  let activeImageSrc = $state<string | null>(null);

  $effect(() => {
    let active = true;
    resolveAttachmentsInMarkdown(content || '').then((res) => {
      if (active) {
        resolvedContent = res;
      }
    });
    return () => {
      active = false;
    };
  });

  let parsed = $derived(marked.parse(resolvedContent || ''));

  async function handleClick(e: MouseEvent) {
    const linkTarget = (e.target as HTMLElement)?.closest('a');
    if (linkTarget && linkTarget.href) {
      e.preventDefault();
      try {
        await openUrl(linkTarget.href);
      } catch {
        window.open(linkTarget.href, '_blank');
      }
      return;
    }

    const imgTarget = (e.target as HTMLElement)?.closest('img');
    if (imgTarget && imgTarget.src) {
      e.preventDefault();
      activeImageSrc = imgTarget.src;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && activeImageSrc) {
      activeImageSrc = null;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="gh-markdown" onclick={handleClick} role="presentation">
  {@html parsed}
</div>

{#if activeImageSrc}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-4"
    onclick={() => (activeImageSrc = null)}
    role="presentation"
  >
    <div class="relative max-w-5xl max-h-[90vh] flex flex-col items-center">
      <button
        type="button"
        onclick={() => (activeImageSrc = null)}
        class="absolute -top-10 right-0 text-white/80 hover:text-white p-1 cursor-pointer"
      >
        <X class="w-6 h-6" />
      </button>
      <img
        src={activeImageSrc}
        alt="Full preview"
        class="max-h-[85vh] max-w-full object-contain rounded shadow-2xl border border-white/20"
      />
    </div>
  </div>
{/if}