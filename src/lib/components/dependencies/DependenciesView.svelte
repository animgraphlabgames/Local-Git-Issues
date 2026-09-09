<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { Package, RefreshCw, Plus, ExternalLink, Check, Trash2, X, Loader2 } from '@lucide/svelte';
  import { formatRelativeTime } from '$lib/utils/format';
  import type { Dependency } from '$lib/types';

  let {
    dependencies,
    onCheckAll,
    onAcknowledge,
    onAdd,
    onDelete
  }: {
    dependencies: Dependency[];
    onCheckAll: () => Promise<void>;
    onAcknowledge: (id: number) => Promise<void>;
    onAdd: (name: string, target: string) => Promise<void>;
    onDelete: (id: number) => Promise<void>;
  } = $props();

  let isChecking = $state(false);
  let isAdding = $state(false);
  let newName = $state('');
  let newTarget = $state('');

  async function handleCheck() {
    isChecking = true;
    try {
      await onCheckAll();
    } finally {
      isChecking = false;
    }
  }

  async function handleAddSubmit(e: Event) {
    e.preventDefault();
    if (!newName.trim() || !newTarget.trim()) return;
    await onAdd(newName.trim(), newTarget.trim());
    newName = '';
    newTarget = '';
    isAdding = false;
  }
</script>

<div class="flex flex-col gap-4">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <button
      onclick={() => (isAdding = !isAdding)}
      class="bg-gh-green-btn hover:bg-gh-green text-white text-xs font-semibold px-3 py-1.5 rounded-md cursor-pointer flex items-center gap-1.5 transition-colors"
    >
      <Plus class="w-3.5 h-3.5" />
      <span>Track repository</span>
    </button>

    <button
      onclick={handleCheck}
      disabled={isChecking}
      class="border border-gh-border bg-white hover:bg-gh-subtle text-gh-text text-xs font-semibold px-3 py-1.5 rounded-md cursor-pointer flex items-center gap-1.5 transition-colors disabled:opacity-50"
    >
      {#if isChecking}
        <Loader2 class="w-3.5 h-3.5 animate-spin text-gh-muted" />
        <span>Checking GitHub...</span>
      {:else}
        <RefreshCw class="w-3.5 h-3.5 text-gh-muted" />
        <span>Check for releases</span>
      {/if}
    </button>
  </div>

  {#if isAdding}
    <form
      onsubmit={handleAddSubmit}
      class="p-4 border border-gh-border rounded-md bg-gh-subtle flex flex-col gap-3 animate-in fade-in duration-100"
    >
      <div class="flex items-center justify-between border-b border-gh-border pb-2.5">
        <div class="flex items-center gap-1.5 text-xs font-semibold text-gh-text">
          <Package class="w-4 h-4 text-gh-muted" />
          <span>Track new dependency release</span>
        </div>
        <button type="button" onclick={() => (isAdding = false)} class="text-gh-muted hover:text-gh-text cursor-pointer">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div class="flex flex-col gap-1">
          <label for="dep-name" class="text-xs font-semibold text-gh-text">Display name</label>
          <input
            id="dep-name"
            type="text"
            placeholder="Blender, Godot, etc."
            bind:value={newName}
            class="text-xs px-3 py-1.5 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link"
            required
          />
        </div>

        <div class="flex flex-col gap-1">
          <label for="dep-target" class="text-xs font-semibold text-gh-text">GitHub URL</label>
          <input
            id="dep-target"
            type="text"
            placeholder="Format: https://github.com/USER/REPO/releases"
            bind:value={newTarget}
            class="text-xs px-3 py-1.5 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link"
            required
          />
        </div>
      </div>

      <div class="flex justify-end gap-2 pt-1">
        <button
          type="button"
          onclick={() => (isAdding = false)}
          class="px-3 py-1.5 text-xs font-semibold rounded-md border border-gh-border bg-white hover:bg-gray-100 cursor-pointer"
        >
          Cancel
        </button>
        <button
          type="submit"
          class="bg-gh-green-btn hover:bg-gh-green text-white text-xs font-semibold px-3 py-1.5 rounded-md cursor-pointer flex items-center gap-1"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>Add repository</span>
        </button>
      </div>
    </form>
  {/if}

  <div class="border border-gh-border rounded-md overflow-hidden bg-white divide-y divide-gh-border">
    <div class="bg-gh-subtle px-4 py-2.5 text-xs font-semibold text-gh-muted flex items-center justify-between">
      <span>{dependencies.length} tracked dependencies</span>
    </div>

    {#if dependencies.length === 0}
      <div class="p-8 text-center text-xs text-gh-muted">
        No tracked dependencies.
      </div>
    {:else}
      {#each dependencies as dep (dep.id)}
        <div class="p-4 flex flex-wrap items-center justify-between gap-3 hover:bg-gh-subtle/50 transition-colors {dep.has_update ? 'bg-purple-50/40' : ''}">
          <div class="flex items-start gap-3 min-w-0">
            <Package class="w-4 h-4 text-gh-muted shrink-0 mt-1" />
            <div class="flex flex-col gap-1 min-w-0">
              <div class="flex items-center gap-2 flex-wrap">
                <span class="font-semibold text-sm text-gh-text">{dep.name}</span>
                <span class="text-xs text-gh-muted font-mono">{dep.repo_owner}/{dep.repo_name}</span>

                {#if dep.has_update}
                  <span class="text-[11px] font-semibold text-gh-purple bg-purple-100 border border-purple-200 px-2 py-0.5 rounded-full">
                    New: {dep.latest_tag}
                  </span>
                {:else if dep.latest_tag}
                  <span class="text-[11px] font-medium text-gh-muted bg-gray-100 px-2 py-0.5 rounded-full">
                    {dep.latest_tag}
                  </span>
                {/if}
              </div>

              <div class="text-xs text-gh-muted flex items-center gap-3 flex-wrap">
                <span>{dep.release_name || 'No releases recorded yet'}</span>
                <span>•</span>
                <span>Checked {dep.last_checked_at ? formatRelativeTime(dep.last_checked_at) : 'never'}</span>
              </div>
            </div>
          </div>

          <div class="flex items-center gap-2 shrink-0">
            {#if dep.has_update}
              <button
                onclick={() => onAcknowledge(dep.id)}
                class="bg-white hover:bg-gh-subtle border border-gh-border text-gh-text text-xs font-semibold px-2.5 py-1 rounded-md cursor-pointer flex items-center gap-1 transition-colors"
                title="Mark as seen"
              >
                <Check class="w-3.5 h-3.5 text-gh-green" />
                <span>Mark seen</span>
              </button>
            {/if}

            {#if dep.release_url}
              <button
                onclick={() => openUrl(dep.release_url!)}
                class="bg-gh-subtle hover:bg-gray-200 border border-gh-border text-gh-text text-xs font-semibold px-2.5 py-1 rounded-md cursor-pointer flex items-center gap-1 transition-colors"
              >
                <ExternalLink class="w-3.5 h-3.5 text-gh-muted" />
                <span>Releases</span>
              </button>
            {/if}

            <button
              onclick={() => onDelete(dep.id)}
              class="text-gh-muted hover:text-gh-red p-1 cursor-pointer transition-colors"
              title="Delete dependency"
            >
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>