<script lang="ts">
  import { marked } from 'marked';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { X } from '@lucide/svelte';
  import { resolveAttachmentsInMarkdown } from '$lib/utils/attachments';
  import IssueHovercard from '$lib/components/issues/IssueHovercard.svelte';
  import type { Issue } from '$lib/types';

  let {
    content,
    issues = [],
    onSelectIssue
  }: {
    content: string;
    issues?: Issue[];
    onSelectIssue?: (id: number) => void;
  } = $props();

  marked.use({
    gfm: true,
    breaks: true,
    extensions: [
      {
        name: 'issueRef',
        level: 'inline',
        start(src: string) {
          const match = /(?:^|[\s(\[])(#\d+\b|\[#\d+\](?!\())/.exec(src);
          return match ? match.index + (match[0].length - match[1].length) : -1;
        },
        tokenizer(src: string) {
          const match = /^(?:\[#(\d+)\](?!\()|#(\d+))\b/.exec(src);
          if (match) {
            return {
              type: 'issueRef',
              raw: match[0],
              issueId: match[1] || match[2]
            };
          }
        },
        renderer(token: any) {
          return `<a href="#issue-${token.issueId}" class="issue-mention font-semibold text-gh-link hover:underline inline-flex items-center cursor-pointer" data-issue-id="${token.issueId}">#${token.issueId}</a>`;
        }
      }
    ]
  });

  let resolvedContent = $state('');
  let activeImageSrc = $state<string | null>(null);
  let hoveredIssue = $state<Issue | null>(null);
  let cardPosition = $state<{ x: number; y: number; placeAbove: boolean }>({ x: 0, y: 0, placeAbove: false });
  let hoverTimeout = $state<ReturnType<typeof setTimeout> | null>(null);

  $effect(() => {
    let active = true;
    resolveAttachmentsInMarkdown(content || '').then((res) => {
      if (active) resolvedContent = res;
    });
    return () => {
      active = false;
    };
  });

  let parsed = $derived(marked.parse(resolvedContent || ''));

  function cancelClose() {
    if (hoverTimeout) {
      clearTimeout(hoverTimeout);
      hoverTimeout = null;
    }
  }

  function scheduleClose() {
    cancelClose();
    hoverTimeout = setTimeout(() => {
      hoveredIssue = null;
    }, 150);
  }

  function updateHovercard(target: HTMLElement) {
    const issueId = Number(target.getAttribute('data-issue-id'));
    const match = issues.find((i) => i.id === issueId);
    if (!match) {
      hoveredIssue = null;
      return;
    }

    const rect = target.getBoundingClientRect();
    const placeAbove = rect.top > 210;
    cardPosition = {
      x: Math.max(12, Math.min(window.innerWidth - 332, rect.left)),
      y: placeAbove ? rect.top - 8 : rect.bottom + 8,
      placeAbove
    };
    hoveredIssue = match;
  }

  async function handleClick(e: MouseEvent) {
    const issueTarget = (e.target as HTMLElement)?.closest('[data-issue-id]') as HTMLElement | null;
    if (issueTarget) {
      e.preventDefault();
      const issueId = Number(issueTarget.getAttribute('data-issue-id'));
      if (issueId && onSelectIssue) {
        hoveredIssue = null;
        onSelectIssue(issueId);
      }
      return;
    }

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

  function handleMouseOver(e: MouseEvent) {
    const target = (e.target as HTMLElement)?.closest('[data-issue-id]') as HTMLElement | null;
    if (target) {
      cancelClose();
      updateHovercard(target);
    }
  }

  function handleMouseOut(e: MouseEvent) {
    const target = (e.target as HTMLElement)?.closest('[data-issue-id]');
    if (target) { scheduleClose(); }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (activeImageSrc) { activeImageSrc = null; }
      if (hoveredIssue) { hoveredIssue = null; }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="gh-markdown"
  onclick={handleClick}
  onmouseover={handleMouseOver}
  onmouseout={handleMouseOut}
  role="presentation"
>
  {@html parsed}
</div>

{#if hoveredIssue}
  <IssueHovercard
    issue={hoveredIssue}
    position={cardPosition}
    onMouseEnter={cancelClose}
    onMouseLeave={scheduleClose}
    onSelectIssue={(id) => {
      hoveredIssue = null;
      onSelectIssue?.(id);
    }}
  />
{/if}

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