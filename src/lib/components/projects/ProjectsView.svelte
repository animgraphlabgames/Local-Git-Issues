<script lang="ts">
  import { FolderKanban, Plus, Pencil, Trash2, Search, X, Check } from '@lucide/svelte';
  import type { Project } from '$lib/types';

  let {
    projects,
    onCreateProject,
    onUpdateProject,
    onDeleteProject
  }: {
    projects: Project[];
    onCreateProject: (title: string, description: string) => void;
    onUpdateProject: (id: number, title: string, description: string) => void;
    onDeleteProject: (id: number) => void;
  } = $props();

  let searchQuery = $state('');
  let isCreating = $state(false);

  let newTitle = $state('');
  let newDescription = $state('');

  let editingId = $state<number | null>(null);
  let editTitle = $state('');
  let editDescription = $state('');

  let filteredProjects = $derived(
    projects.filter((p) =>
      p.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.description.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );

  function handleCreateSubmit(e: Event) {
    e.preventDefault();
    if (!newTitle.trim()) return;
    onCreateProject(newTitle.trim(), newDescription.trim());
    newTitle = '';
    newDescription = '';
    isCreating = false;
  }

  function startEditing(p: Project) {
    editingId = p.id;
    editTitle = p.title;
    editDescription = p.description;
  }

  function handleUpdateSubmit(e: Event, id: number) {
    e.preventDefault();
    if (!editTitle.trim()) return;
    onUpdateProject(id, editTitle.trim(), editDescription.trim());
    editingId = null;
  }
</script>

<div class="flex flex-col gap-4">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <div class="relative w-64">
      <Search class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-gh-muted pointer-events-none" />
      <input
        type="text"
        placeholder="Filter projects..."
        bind:value={searchQuery}
        class="w-full text-xs pl-8 pr-3 py-1.5 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link"
      />
    </div>

    <button
      onclick={() => (isCreating = !isCreating)}
      class="bg-gh-green-btn hover:bg-gh-green text-white text-xs font-semibold px-3 py-1.5 rounded-md cursor-pointer flex items-center gap-1.5 transition-colors"
    >
      <Plus class="w-3.5 h-3.5" />
      <span>New project</span>
    </button>
  </div>

  {#if isCreating}
    <form
      onsubmit={handleCreateSubmit}
      class="p-4 border border-gh-border rounded-md bg-gh-subtle flex flex-col gap-3 animate-in fade-in duration-100"
    >
      <div class="flex items-center justify-between border-b border-gh-border pb-2.5">
        <div class="flex items-center gap-1.5 text-xs font-semibold text-gh-text">
          <FolderKanban class="w-4 h-4 text-gh-muted" />
          <span>Create project</span>
        </div>
        <button
          type="button"
          onclick={() => (isCreating = false)}
          class="text-gh-muted hover:text-gh-text cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="flex flex-col gap-3">
        <div class="flex flex-col gap-1">
          <label for="create-project-title" class="text-xs font-semibold text-gh-text">Project title</label>
          <input
            id="create-project-title"
            type="text"
            placeholder="Project title"
            bind:value={newTitle}
            class="text-xs px-3 py-1.5 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link"
            required
          />
        </div>

        <div class="flex flex-col gap-1">
          <label for="create-project-desc" class="text-xs font-semibold text-gh-text">Description</label>
          <textarea
            id="create-project-desc"
            placeholder="Description (optional)"
            bind:value={newDescription}
            rows="2"
            class="text-xs p-2 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link resize-y"
          ></textarea>
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
          <span>Create project</span>
        </button>
      </div>
    </form>
  {/if}

  <div class="border border-gh-border rounded-md overflow-hidden bg-white divide-y divide-gh-border">
    <div class="bg-gh-subtle px-4 py-2.5 text-xs font-semibold text-gh-muted flex items-center justify-between">
      <span>{filteredProjects.length} projects</span>
    </div>

    {#if filteredProjects.length === 0}
      <div class="p-8 text-center text-xs text-gh-muted">
        No projects found.
      </div>
    {:else}
      {#each filteredProjects as p (p.id)}
        {#if editingId === p.id}
          <form
            onsubmit={(e) => handleUpdateSubmit(e, p.id)}
            class="p-4 bg-gh-subtle flex flex-col gap-3"
          >
            <div class="flex flex-col gap-1">
              <label for="edit-project-title-{p.id}" class="text-xs font-semibold text-gh-text">Project title</label>
              <input
                id="edit-project-title-{p.id}"
                type="text"
                bind:value={editTitle}
                class="text-xs px-3 py-1.5 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link"
                required
              />
            </div>

            <div class="flex flex-col gap-1">
              <label for="edit-project-desc-{p.id}" class="text-xs font-semibold text-gh-text">Description</label>
              <textarea
                id="edit-project-desc-{p.id}"
                bind:value={editDescription}
                rows="2"
                class="text-xs p-2 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link resize-y"
              ></textarea>
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
            <div class="flex items-start gap-3 flex-1 min-w-0 pr-4">
              <FolderKanban class="w-4 h-4 text-gh-muted shrink-0 mt-0.5" />
              <div class="min-w-0">
                <h3 class="text-sm font-semibold text-gh-text leading-snug">{p.title}</h3>
                <p class="text-xs text-gh-muted mt-0.5 leading-relaxed">{p.description || 'No description provided.'}</p>
              </div>
            </div>

            <div class="flex items-center gap-3 shrink-0 self-center">
              <button
                onclick={() => startEditing(p)}
                class="text-xs text-gh-muted hover:text-gh-link cursor-pointer flex items-center gap-1"
              >
                <Pencil class="w-3.5 h-3.5" />
                <span>Edit</span>
              </button>
              <button
                onclick={() => onDeleteProject(p.id)}
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