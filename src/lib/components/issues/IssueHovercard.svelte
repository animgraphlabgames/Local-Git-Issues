<script lang="ts">
  import { CircleDot, CheckCircle2, FolderKanban } from '@lucide/svelte';
  import LabelBadge from '$lib/components/ui/LabelBadge.svelte';
  import { formatRelativeTime, getPlainTextExcerpt } from '$lib/utils/format';
  import type { Issue } from '$lib/types';

  let {
    issue,
    position,
    onMouseEnter,
    onMouseLeave,
    onSelectIssue
  }: {
    issue: Issue;
    position: { x: number; y: number; placeAbove: boolean };
    onMouseEnter: () => void;
    onMouseLeave: () => void;
    onSelectIssue?: (id: number) => void;
  } = $props();
</script>

<div
  class="fixed z-50 w-80 bg-white border border-gh-border rounded-lg shadow-xl p-3.5 flex flex-col gap-2.5 text-xs animate-in fade-in zoom-in-95 duration-100 select-none {position.placeAbove ? '-translate-y-full' : ''}"
  style="left: {position.x}px; top: {position.y}px;"
  onmouseenter={onMouseEnter}
  onmouseleave={onMouseLeave}
  role="tooltip"
>
  <div class="flex items-center gap-2">
    {#if issue.status === 'open'}
      <span class="flex items-center gap-1 text-[11px] font-semibold text-gh-green bg-green-50 border border-green-200 px-2 py-0.5 rounded-full">
        <CircleDot class="w-3 h-3" />
        Open
      </span>
    {:else}
      <span class="flex items-center gap-1 text-[11px] font-semibold text-gh-purple bg-purple-50 border border-purple-200 px-2 py-0.5 rounded-full">
        <CheckCircle2 class="w-3 h-3" />
        Closed
      </span>
    {/if}
    <span class="font-mono text-gh-muted font-medium">#{issue.id}</span>
    <span class="text-gh-muted ml-auto text-[11px]">{formatRelativeTime(issue.created_at)}</span>
  </div>

  <button
    type="button"
    onclick={() => onSelectIssue?.(issue.id)}
    class="text-left font-semibold text-sm text-gh-text hover:text-gh-link leading-snug line-clamp-2 cursor-pointer transition-colors"
  >
    {issue.title}
  </button>

  <p class="text-gh-muted line-clamp-3 leading-relaxed text-xs">
    {getPlainTextExcerpt(issue.body) || 'No description provided.'}
  </p>

  {#if issue.labels.length > 0 || issue.project_title}
    <div class="flex flex-wrap items-center gap-1.5 pt-2 border-t border-gh-border/60">
      {#each issue.labels as lbl (lbl.id)}
        <LabelBadge label={lbl} size="xs" />
      {/each}
      {#if issue.project_title}
        <span class="text-gh-muted flex items-center gap-1 text-[11px] ml-auto">
          <FolderKanban class="w-3 h-3" />
          <span class="truncate max-w-[120px]">{issue.project_title}</span>
        </span>
      {/if}
    </div>
  {/if}
</div>