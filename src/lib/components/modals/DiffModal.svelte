<script lang="ts">
  import { History } from '@lucide/svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Dropdown from '$lib/components/ui/Dropdown.svelte';
  import { computeLineDiff } from '$lib/utils/diff';
  import { formatRelativeTime } from '$lib/utils/format';
  import type { IssueRevision } from '$lib/types';

  let {
    isOpen,
    revisions,
    onClose
  }: {
    isOpen: boolean;
    revisions: IssueRevision[];
    onClose: () => void;
  } = $props();

  let selectedIndex = $state(0);

  $effect(() => {
    if (revisions.length > 1) {
      selectedIndex = revisions.length - 1;
    }
  });

  let revisionItems = $derived(
    revisions.map((rev, index) => ({
      value: index,
      label: index === 0
        ? `Revision 1 (Initial) - ${formatRelativeTime(rev.created_at)}`
        : `Revision ${index + 1} - ${formatRelativeTime(rev.created_at)}`
    }))
  );

  let prevRevision = $derived(
    selectedIndex > 0 ? revisions[selectedIndex - 1] : null
  );

  let currentRevision = $derived(
    revisions[selectedIndex] ?? null
  );

  let diffLines = $derived(
    computeLineDiff(
      prevRevision ? prevRevision.body : '',
      currentRevision ? currentRevision.body : ''
    )
  );

  let additions = $derived(diffLines.filter((l) => l.type === 'added').length);
  let deletions = $derived(diffLines.filter((l) => l.type === 'removed').length);

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;
    const isInputActive = ['INPUT', 'TEXTAREA', 'SELECT'].includes(document.activeElement?.tagName || '');
    if (isInputActive) return;

    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      if (selectedIndex > 0) {
        selectedIndex -= 1;
      }
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      if (selectedIndex < revisions.length - 1) {
        selectedIndex += 1;
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<Modal {isOpen} {onClose} title="Edit History" icon={History} maxWidth="max-w-4xl">
  <div class="flex flex-col gap-4 p-4 text-xs">
    <div class="flex flex-wrap items-center justify-between gap-3 bg-gh-subtle p-3 border border-gh-border rounded-md relative z-20">
      <div class="flex items-center gap-2">
        <span class="font-semibold text-gh-text">Revision:</span>
        <Dropdown
          items={revisionItems}
          value={selectedIndex}
          onselect={(val) => {
            if (val !== null) selectedIndex = Number(val);
          }}
        />
      </div>

      <div class="flex items-center gap-2 font-mono font-semibold">
        <span class="text-gh-green">+{additions}</span>
        <span class="text-gh-red">-{deletions}</span>
      </div>
    </div>

    <div class="border border-gh-border rounded-md overflow-hidden bg-white relative z-0">
      <div class="bg-gh-subtle border-b border-gh-border px-3 py-1.5 font-mono text-[11px] text-gh-muted flex items-center justify-between">
        <span>Unified Diff</span>
        {#if prevRevision && currentRevision}
          <span>Comparing #{selectedIndex} → #{selectedIndex + 1}</span>
        {/if}
      </div>

      <div class="max-h-[55vh] overflow-auto font-mono text-xs leading-relaxed divide-y divide-gh-border/20">
        {#each diffLines as line}
          <div
            class="flex items-start {line.type === 'added' ? 'bg-green-50 text-green-950' : line.type === 'removed' ? 'bg-red-50 text-red-950' : 'bg-white text-gh-text'}"
          >
            <span class="w-10 px-2 py-0.5 text-right select-none text-gh-muted border-r border-gh-border/40 text-[11px] shrink-0">
              {line.oldLineNumber ?? ''}
            </span>
            <span class="w-10 px-2 py-0.5 text-right select-none text-gh-muted border-r border-gh-border/40 text-[11px] shrink-0">
              {line.newLineNumber ?? ''}
            </span>
            <span class="w-6 px-1.5 py-0.5 text-center font-bold select-none shrink-0 {line.type === 'added' ? 'text-gh-green' : line.type === 'removed' ? 'text-gh-red' : 'text-transparent'}">
              {#if line.type === 'added'}+{/if}
              {#if line.type === 'removed'}-{/if}
            </span>
            <span class="flex-1 px-2 py-0.5 whitespace-pre-wrap break-words">{line.text || ' '}</span>
          </div>
        {/each}
      </div>
    </div>
  </div>

  {#snippet footer()}
    <div class="flex items-center justify-between w-full">
      <div class="flex items-center gap-1.5 text-xs text-gh-muted select-none">
        <span>Navigate revisions:</span>
        <kbd class="px-1.5 py-0.5 bg-white border border-gh-border rounded font-mono text-[10px] text-gh-text shadow-xs">←</kbd>
        <kbd class="px-1.5 py-0.5 bg-white border border-gh-border rounded font-mono text-[10px] text-gh-text shadow-xs">→</kbd>
      </div>

      <button
        onclick={onClose}
        class="bg-gh-subtle hover:bg-gray-200 border border-gh-border text-gh-text text-xs font-semibold px-3 py-1.5 rounded-md cursor-pointer"
      >
        Close
      </button>
    </div>
  {/snippet}
</Modal>