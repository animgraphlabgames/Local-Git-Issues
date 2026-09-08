<script lang="ts">
  import { Tag, Plus, Pencil, Trash2, Search, X, Check, RefreshCw } from '@lucide/svelte';
  import LabelBadge from '$lib/components/ui/LabelBadge.svelte';
  import type { Label } from '$lib/types';

  let {
    labels,
    onCreateLabel,
    onUpdateLabel,
    onDeleteLabel
  }: {
    labels: Label[];
    onCreateLabel: (name: string, color: string, description: string) => void;
    onUpdateLabel: (id: number, name: string, color: string, description: string) => void;
    onDeleteLabel: (id: number) => void;
  } = $props();

  const PRESET_COLORS = [
    '#d73a4a', '#0075ca', '#a2eeef', '#0e8a16',
    '#fbca04', '#d93f0b', '#5319e7', '#e99695'
  ];

  let searchQuery = $state('');
  let isCreating = $state(false);

  let newName = $state('');
  let newColor = $state('#0075ca');
  let newDescription = $state('');

  let editingId = $state<number | null>(null);
  let editName = $state('');
  let editColor = $state('');
  let editDescription = $state('');

  let filteredLabels = $derived(
    labels.filter((l) =>
      l.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      l.description.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );

  function randomColor(): string {
    return '#' + Math.floor(Math.random() * 16777215).toString(16).padStart(6, '0');
  }

  function handleCreateSubmit(e: Event) {
    e.preventDefault();
    if (!newName.trim()) return;
    onCreateLabel(newName.trim(), newColor, newDescription.trim());
    newName = '';
    newDescription = '';
    newColor = randomColor();
    isCreating = false;
  }

  function startEditing(lbl: Label) {
    editingId = lbl.id;
    editName = lbl.name;
    editColor = lbl.color;
    editDescription = lbl.description;
  }

  function handleUpdateSubmit(e: Event, id: number) {
    e.preventDefault();
    if (!editName.trim()) return;
    onUpdateLabel(id, editName.trim(), editColor, editDescription.trim());
    editingId = null;
  }
</script>

<div class="flex flex-col gap-4">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <div class="relative w-64">
      <Search class="w-3.5 h-3.5 absolute left-2.5 top-2.5 text-gh-muted" />
      <input
        type="text"
        placeholder="Filter labels..."
        bind:value={searchQuery}
        class="w-full text-xs pl-8 pr-3 py-1.5 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link"
      />
    </div>

    <button
      onclick={() => {
        isCreating = !isCreating;
        if (isCreating) newColor = randomColor();
      }}
      class="bg-gh-green-btn hover:bg-gh-green text-white text-xs font-semibold px-3 py-1.5 rounded-md cursor-pointer flex items-center gap-1.5 transition-colors"
    >
      <Plus class="w-3.5 h-3.5" />
      <span>New label</span>
    </button>
  </div>

  {#if isCreating}
    <form
      onsubmit={handleCreateSubmit}
      class="p-4 border border-gh-border rounded-md bg-gh-subtle flex flex-col gap-4 animate-in fade-in duration-100"
    >
      <div class="flex items-center justify-between border-b border-gh-border pb-3">
        <div class="flex items-center gap-2">
          <span class="text-xs text-gh-muted font-medium">Preview:</span>
          <span
            class="text-xs font-semibold px-2.5 py-1 rounded-full border border-black/10 inline-flex items-center"
            style="background-color: {newColor}; color: #000;"
          >
            {newName.trim() || 'Label preview'}
          </span>
        </div>
        <button
          type="button"
          onclick={() => (isCreating = false)}
          class="text-gh-muted hover:text-gh-text cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-12 gap-3 items-end">
        <div class="md:col-span-4 flex flex-col gap-1">
          <label for="create-label-name" class="text-xs font-semibold text-gh-text">Label name</label>
          <input
            id="create-label-name"
            type="text"
            placeholder="Label name"
            bind:value={newName}
            class="text-xs px-3 py-1.5 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link"
            required
          />
        </div>

        <div class="md:col-span-4 flex flex-col gap-1">
          <label for="create-label-desc" class="text-xs font-semibold text-gh-text">Description</label>
          <input
            id="create-label-desc"
            type="text"
            placeholder="Description (optional)"
            bind:value={newDescription}
            class="text-xs px-3 py-1.5 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link"
          />
        </div>

        <div class="md:col-span-4 flex flex-col gap-1">
          <span class="text-xs font-semibold text-gh-text">Color</span>
          <div class="flex items-center gap-1.5">
            <button
              type="button"
              onclick={() => (newColor = randomColor())}
              title="Generate random color"
              class="p-1.5 border border-gh-border rounded-md bg-white hover:bg-gray-100 text-gh-muted cursor-pointer shrink-0"
            >
              <RefreshCw class="w-3.5 h-3.5" />
            </button>
            <input
              type="color"
              bind:value={newColor}
              class="h-7 w-9 border border-gh-border rounded cursor-pointer shrink-0"
            />
            <div class="flex items-center gap-1 overflow-x-auto">
              {#each PRESET_COLORS as hex}
                <button
                  type="button"
                  onclick={() => (newColor = hex)}
                  class="w-5 h-5 rounded-full border border-black/20 shrink-0 cursor-pointer transition-transform hover:scale-110"
                  style="background-color: {hex};"
                ></button>
              {/each}
            </div>
          </div>
        </div>
      </div>

      <div class="flex justify-end gap-2 pt-1">
        <button
          type="button"
          onclick={() => (isCreating = false)}
          class="px-3 py-1.5 text-xs font-semibold rounded-md border border-gh-border bg-white hover:bg-gray-100 cursor-pointer"
        >
          Cancel
        </button>
        <button
          type="submit"
          class="bg-gh-green-btn hover:bg-gh-green text-white text-xs font-semibold px-3 py-1.5 rounded-md cursor-pointer flex items-center gap-1"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>Create label</span>
        </button>
      </div>
    </form>
  {/if}

  <div class="border border-gh-border rounded-md overflow-hidden bg-white divide-y divide-gh-border">
    <div class="bg-gh-subtle px-4 py-2.5 text-xs font-semibold text-gh-muted flex items-center justify-between">
      <span>{filteredLabels.length} labels</span>
    </div>

    {#if filteredLabels.length === 0}
      <div class="p-8 text-center text-xs text-gh-muted">
        No labels found.
      </div>
    {:else}
      {#each filteredLabels as lbl (lbl.id)}
        {#if editingId === lbl.id}
          <form
            onsubmit={(e) => handleUpdateSubmit(e, lbl.id)}
            class="p-4 bg-gh-subtle flex flex-col gap-3"
          >
            <div class="flex items-center gap-2">
              <span class="text-xs text-gh-muted font-medium">Preview:</span>
              <span
                class="text-xs font-semibold px-2.5 py-1 rounded-full border border-black/10 inline-flex items-center"
                style="background-color: {editColor}; color: #000;"
              >
                {editName.trim() || 'Label preview'}
              </span>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-12 gap-3 items-end">
              <div class="md:col-span-4 flex flex-col gap-1">
                <label for="edit-label-name-{lbl.id}" class="text-xs font-semibold text-gh-text">Label name</label>
                <input
                  id="edit-label-name-{lbl.id}"
                  type="text"
                  bind:value={editName}
                  class="text-xs px-3 py-1.5 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link"
                  required
                />
              </div>

              <div class="md:col-span-4 flex flex-col gap-1">
                <label for="edit-label-desc-{lbl.id}" class="text-xs font-semibold text-gh-text">Description</label>
                <input
                  id="edit-label-desc-{lbl.id}"
                  type="text"
                  bind:value={editDescription}
                  class="text-xs px-3 py-1.5 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link"
                />
              </div>

              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold text-gh-text">Color</span>
                <div class="flex items-center gap-1.5">
                  <button
                    type="button"
                    onclick={() => (editColor = randomColor())}
                    title="Generate random color"
                    class="p-1.5 border border-gh-border rounded-md bg-white hover:bg-gray-100 text-gh-muted cursor-pointer shrink-0"
                  >
                    <RefreshCw class="w-3.5 h-3.5" />
                  </button>
                  <input
                    type="color"
                    bind:value={editColor}
                    class="h-7 w-9 border border-gh-border rounded cursor-pointer shrink-0"
                  />
                  <div class="flex items-center gap-1 overflow-x-auto">
                    {#each PRESET_COLORS as hex}
                      <button
                        type="button"
                        onclick={() => (editColor = hex)}
                        class="w-5 h-5 rounded-full border border-black/20 shrink-0 cursor-pointer transition-transform hover:scale-110"
                        style="background-color: {hex};"
                      ></button>
                    {/each}
                  </div>
                </div>
              </div>
            </div>

            <div class="flex justify-end gap-2 pt-1">
              <button
                type="button"
                onclick={() => (editingId = null)}
                class="px-3 py-1.5 text-xs font-semibold rounded-md border border-gh-border bg-white hover:bg-gray-100 cursor-pointer"
              >
                Cancel
              </button>
              <button
                type="submit"
                class="bg-gh-green-btn hover:bg-gh-green text-white text-xs font-semibold px-3 py-1.5 rounded-md cursor-pointer flex items-center gap-1"
              >
                <Check class="w-3.5 h-3.5" />
                <span>Save changes</span>
              </button>
            </div>
          </form>
        {:else}
          <div class="px-4 py-3 flex items-center justify-between hover:bg-gh-subtle transition-colors">
            <div class="flex items-center gap-4 flex-1 min-w-0 pr-4">
              <div class="shrink-0">
                <LabelBadge label={lbl} size="sm" />
              </div>
              <span class="text-xs text-gh-muted truncate">{lbl.description || 'No description'}</span>
            </div>

            <div class="flex items-center gap-3 shrink-0">
              <button
                onclick={() => startEditing(lbl)}
                class="text-xs text-gh-muted hover:text-gh-link cursor-pointer flex items-center gap-1"
              >
                <Pencil class="w-3.5 h-3.5" />
                <span>Edit</span>
              </button>
              <button
                onclick={() => onDeleteLabel(lbl.id)}
                class="text-xs text-gh-muted hover:text-gh-red cursor-pointer flex items-center gap-1"
              >
                <Trash2 class="w-3.5 h-3.5" />
                <span>Delete</span>
              </button>
            </div>
          </div>
        {/if}
      {/each}
    {/if}
  </div>
</div>