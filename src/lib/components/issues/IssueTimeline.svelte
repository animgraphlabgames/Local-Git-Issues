<script lang="ts">
  import { CircleDot, CheckCircle2, Tag, FolderKanban, Pencil } from '@lucide/svelte';
  import LabelBadge from '$lib/components/ui/LabelBadge.svelte';
  import { formatRelativeTime } from '$lib/utils/format';
  import type { IssueEvent } from '$lib/types';

  let { events }: { events: IssueEvent[] } = $props();

  let visibleEvents = $derived.by(() => {
    const result: IssueEvent[] = [];
    for (const ev of events) {
      if (ev.event_type === 'label_added' || ev.event_type === 'label_removed') {
        const lastIdx = result.findLastIndex(
          (r) =>
            (r.event_type === 'label_added' || r.event_type === 'label_removed') &&
            r.new_value === ev.new_value
        );
        if (lastIdx !== -1) {
          const prev = result[lastIdx];
          const diff = Math.abs(
            (new Date(ev.created_at).getTime() - new Date(prev.created_at).getTime()) / 1000
          );
          if (diff <= 60) {
            if (prev.event_type !== ev.event_type) {
              result.splice(lastIdx, 1);
              continue;
            } else {
              continue;
            }
          }
        }
      } else if (ev.event_type === 'status_change') {
        const last = result[result.length - 1];
        if (last && last.event_type === 'status_change') {
          const diff = Math.abs(
            (new Date(ev.created_at).getTime() - new Date(last.created_at).getTime()) / 1000
          );
          if (diff <= 60) {
            if (last.old_value === ev.new_value) {
              result.pop();
              continue;
            } else {
              last.new_value = ev.new_value;
              last.created_at = ev.created_at;
              continue;
            }
          }
        }
      } else if (ev.event_type === 'project_change') {
        const last = result[result.length - 1];
        if (last && last.event_type === 'project_change') {
          const diff = Math.abs(
            (new Date(ev.created_at).getTime() - new Date(last.created_at).getTime()) / 1000
          );
          if (diff <= 60) {
            if (last.old_value === ev.new_value) {
              result.pop();
              continue;
            } else {
              last.new_value = ev.new_value;
              last.created_at = ev.created_at;
              continue;
            }
          }
        }
      } else if (ev.event_type === 'title_change') {
        const last = result[result.length - 1];
        if (last && last.event_type === 'title_change') {
          const diff = Math.abs(
            (new Date(ev.created_at).getTime() - new Date(last.created_at).getTime()) / 1000
          );
          if (diff <= 60) {
            if (last.old_value === ev.new_value) {
              result.pop();
              continue;
            } else {
              last.new_value = ev.new_value;
              last.created_at = ev.created_at;
              continue;
            }
          }
        }
      }
      result.push(ev);
    }
    return result;
  });
</script>

{#if visibleEvents.length > 0}
  <div class="relative flex flex-col gap-4 mt-6 before:content-[''] before:absolute before:left-[13px] before:top-2 before:bottom-2 before:w-0.5 before:bg-gh-border">
    {#each visibleEvents as ev (ev.id)}
      <div class="relative flex items-center gap-3 text-xs text-gh-muted">
        <div class="relative z-10 flex items-center justify-center w-7 h-7 rounded-full bg-white shrink-0">
          {#if ev.event_type === 'status_change'}
            {#if ev.new_value === 'closed'}
              <span class="w-7 h-7 rounded-full bg-gh-purple text-white flex items-center justify-center shadow-xs">
                <CheckCircle2 class="w-3.5 h-3.5" />
              </span>
            {:else}
              <span class="w-7 h-7 rounded-full bg-gh-green text-white flex items-center justify-center shadow-xs">
                <CircleDot class="w-3.5 h-3.5" />
              </span>
            {/if}
          {:else if ev.event_type === 'title_change'}
            <span class="w-7 h-7 rounded-full bg-gh-subtle border border-gh-border text-gh-muted flex items-center justify-center">
              <Pencil class="w-3.5 h-3.5" />
            </span>
          {:else if ev.event_type === 'label_added' || ev.event_type === 'label_removed'}
            <span class="w-7 h-7 rounded-full bg-gh-subtle border border-gh-border text-gh-muted flex items-center justify-center">
              <Tag class="w-3.5 h-3.5" />
            </span>
          {:else if ev.event_type === 'project_change'}
            <span class="w-7 h-7 rounded-full bg-gh-subtle border border-gh-border text-gh-muted flex items-center justify-center">
              <FolderKanban class="w-3.5 h-3.5" />
            </span>
          {/if}
        </div>

        <div class="flex-1 flex flex-wrap items-center gap-1.5 py-1">
          {#if ev.event_type === 'status_change'}
            {#if ev.new_value === 'closed'}
              <span class="font-semibold text-gh-text">closed this</span>
            {:else}
              <span class="font-semibold text-gh-text">reopened this</span>
            {/if}
          {:else if ev.event_type === 'title_change'}
            <span>changed title from</span>
            <span class="line-through font-semibold text-gh-text">{ev.old_value}</span>
            <span>to</span>
            <span class="font-semibold text-gh-text">{ev.new_value}</span>
          {:else if ev.event_type === 'label_added'}
            <span>added</span>
            <LabelBadge label={{ id: 0, name: ev.new_value ?? '', color: ev.metadata ?? '#0969da', description: '' }} size="xs" />
            <span>label</span>
          {:else if ev.event_type === 'label_removed'}
            <span>removed</span>
            <LabelBadge label={{ id: 0, name: ev.new_value ?? '', color: ev.metadata ?? '#0969da', description: '' }} size="xs" />
            <span>label</span>
          {:else if ev.event_type === 'project_change'}
            {#if !ev.old_value && ev.new_value}
              <span>added this to</span>
              <span class="font-semibold text-gh-text">{ev.new_value}</span>
            {:else if ev.old_value && !ev.new_value}
              <span>removed this from</span>
              <span class="font-semibold text-gh-text">{ev.old_value}</span>
            {:else if ev.old_value && ev.new_value}
              <span>moved from</span>
              <span class="font-semibold text-gh-text">{ev.old_value}</span>
              <span>to</span>
              <span class="font-semibold text-gh-text">{ev.new_value}</span>
            {/if}
          {/if}

          <span class="text-gh-muted ml-auto sm:ml-1">{formatRelativeTime(ev.created_at)}</span>
        </div>
      </div>
    {/each}
  </div>
{/if}