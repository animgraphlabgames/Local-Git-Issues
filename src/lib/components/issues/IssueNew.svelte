<script lang="ts">
  import { Tag, FolderKanban, Settings, Check, X, Search } from '@lucide/svelte';
  import MarkdownEditor from '$lib/components/markdown/MarkdownEditor.svelte';
  import LabelBadge from '$lib/components/ui/LabelBadge.svelte';
  import type { Issue, Label, Project, CreateIssuePayload } from '$lib/types';

  let {
    issues = [],
    labels,
    projects,
    onSubmit,
    onCancel,
    onSelectIssue
  }: {
    issues?: Issue[];
    labels: Label[];
    projects: Project[];
    onSubmit: (data: CreateIssuePayload) => void;
    onCancel: () => void;
    onSelectIssue?: (id: number) => void;
  } = $props();

  let title = $state('');
  let body = $state('');
  let selectedLabels = $state<number[]>([]);
  let selectedProject = $state<number | null>(null);

  let isLabelsOpen = $state(false);
  let isProjectsOpen = $state(false);
  let labelSearch = $state('');
  let projectSearch = $state('');

  let labelsMenuRef = $state<HTMLDivElement | null>(null);
  let projectsMenuRef = $state<HTMLDivElement | null>(null);

  let filteredLabels = $derived(
    labels.filter((l) => l.name.toLowerCase().includes(labelSearch.toLowerCase()))
  );

  let filteredProjects = $derived(
    projects.filter((p) => p.title.toLowerCase().includes(projectSearch.toLowerCase()))
  );

  let activeLabels = $derived(
    labels.filter((l) => selectedLabels.includes(l.id))
  );

  let currentProject = $derived(
    projects.find((p) => p.id === selectedProject)
  );

  function toggleLabel(id: number) {
    selectedLabels = selectedLabels.includes(id)
      ? selectedLabels.filter((i) => i !== id)
      : [...selectedLabels, id];
  }

  function handleSubmit(e?: Event) {
    e?.preventDefault();
    if (!title.trim()) return;
    onSubmit({
      title: title.trim(),
      body,
      projectId: selectedProject,
      labelIds: selectedLabels
    });
  }

  function handleWindowClick(e: MouseEvent) {
    const target = e.target as Node;
    if (isLabelsOpen && labelsMenuRef && !labelsMenuRef.contains(target)) {
      isLabelsOpen = false;
    }
    if (isProjectsOpen && projectsMenuRef && !projectsMenuRef.contains(target)) {
      isProjectsOpen = false;
    }
  }

  function handleWindowKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      isLabelsOpen = false;
      isProjectsOpen = false;
    }
  }
</script>

<svelte:window onclick={handleWindowClick} onkeydown={handleWindowKeydown} />

<div class="grid grid-cols-1 md:grid-cols-4 gap-6 items-start">
  <form onsubmit={handleSubmit} class="md:col-span-3 flex flex-col gap-4">
    <div>
      <input
        type="text"
        placeholder="Title"
        bind:value={title}
        class="w-full text-base font-normal px-3 py-2 bg-gh-subtle border border-gh-border rounded-md focus:outline-none focus:border-gh-link focus:bg-white"
        required
      />
    </div>

    <MarkdownEditor
      bind:value={body}
      rows={14}
      {issues}
      {onSelectIssue}
      placeholder="Leave a comment (supports Markdown, tables, #issue links, etc.)"
      onsubmit={handleSubmit}
    />

    <div class="flex items-center justify-end gap-3">
      <button
        type="button"
        onclick={onCancel}
        class="px-4 py-1.5 text-xs font-semibold rounded-md border border-gh-border bg-gh-subtle hover:bg-gray-200 cursor-pointer"
      >
        Cancel
      </button>
      <button
        type="submit"
        class="bg-gh-green-btn hover:bg-gh-green text-white text-xs font-semibold px-4 py-1.5 rounded-md cursor-pointer"
      >
        Submit new issue
      </button>
    </div>
  </form>

  <div class="md:col-span-1 border-t md:border-t-0 md:border-l border-gh-border md:pl-6 flex flex-col gap-6 text-xs">
    <div class="relative" bind:this={labelsMenuRef}>
      <div class="flex items-center justify-between mb-2">
        <button
          type="button"
          onclick={() => {
            isLabelsOpen = !isLabelsOpen;
            isProjectsOpen = false;
          }}
          class="font-semibold text-gh-text hover:text-gh-link flex items-center gap-1.5 cursor-pointer"
        >
          <Tag class="w-3.5 h-3.5 text-gh-muted" />
          <span>Labels</span>
        </button>
        <button
          type="button"
          onclick={() => {
            isLabelsOpen = !isLabelsOpen;
            isProjectsOpen = false;
          }}
          class="p-1 text-gh-muted hover:text-gh-text rounded hover:bg-gh-subtle cursor-pointer"
        >
          <Settings class="w-3.5 h-3.5" />
        </button>
      </div>

      <div class="flex flex-wrap gap-1.5">
        {#each activeLabels as lbl (lbl.id)}
          <LabelBadge label={lbl} size="xs" />
        {/each}
        {#if activeLabels.length === 0}
          <span class="text-gh-muted">None yet</span>
        {/if}
      </div>

      {#if isLabelsOpen}
        <div class="absolute left-0 md:right-0 md:left-auto mt-2 w-64 bg-white border border-gh-border rounded-md shadow-lg py-1 z-30 animate-in fade-in zoom-in-95 duration-100">
          <div class="px-3 py-1.5 text-[11px] font-semibold text-gh-muted border-b border-gh-border flex items-center justify-between">
            <span>Apply labels</span>
            <button onclick={() => (isLabelsOpen = false)} class="hover:text-gh-text cursor-pointer">
              <X class="w-3.5 h-3.5" />
            </button>
          </div>
          <div class="p-2 border-b border-gh-border">
            <div class="relative">
              <Search class="w-3 h-3 absolute left-2 top-2 text-gh-muted" />
              <input
                type="text"
                placeholder="Filter labels..."
                bind:value={labelSearch}
                class="w-full text-xs pl-6 pr-2 py-1 bg-gh-subtle border border-gh-border rounded focus:outline-none focus:border-gh-link focus:bg-white"
              />
            </div>
          </div>
          <div class="max-h-48 overflow-y-auto py-1">
            {#if filteredLabels.length === 0}
              <div class="px-3 py-2 text-xs text-gh-muted">No labels found</div>
            {:else}
              {#each filteredLabels as lbl (lbl.id)}
                {@const isChecked = selectedLabels.includes(lbl.id)}
                <button
                  type="button"
                  onclick={() => toggleLabel(lbl.id)}
                  class="w-full text-left px-3 py-1.5 text-xs flex items-center gap-2 hover:bg-gh-subtle cursor-pointer transition-colors"
                >
                  <span class="w-3.5 flex justify-center shrink-0">
                    {#if isChecked}
                      <Check class="w-3.5 h-3.5 text-gh-link" />
                    {/if}
                  </span>
                  <span class="w-2.5 h-2.5 rounded-full shrink-0" style="background-color: {lbl.color};"></span>
                  <span class="truncate flex-1 {isChecked ? 'font-semibold text-gh-text' : 'text-gh-text'}">{lbl.name}</span>
                </button>
              {/each}
            {/if}
          </div>
        </div>
      {/if}
    </div>

    <div class="border-t border-gh-border pt-4 relative" bind:this={projectsMenuRef}>
      <div class="flex items-center justify-between mb-2">
        <button
          type="button"
          onclick={() => {
            isProjectsOpen = !isProjectsOpen;
            isLabelsOpen = false;
          }}
          class="font-semibold text-gh-text hover:text-gh-link flex items-center gap-1.5 cursor-pointer"
        >
          <FolderKanban class="w-3.5 h-3.5 text-gh-muted" />
          <span>Project</span>
        </button>
        <button
          type="button"
          onclick={() => {
            isProjectsOpen = !isProjectsOpen;
            isLabelsOpen = false;
          }}
          class="p-1 text-gh-muted hover:text-gh-text rounded hover:bg-gh-subtle cursor-pointer"
        >
          <Settings class="w-3.5 h-3.5" />
        </button>
      </div>

      <div>
        {#if currentProject}
          <span class="font-medium text-gh-text leading-snug">{currentProject.title}</span>
        {:else}
          <span class="text-gh-muted">None yet</span>
        {/if}
      </div>

      {#if isProjectsOpen}
        <div class="absolute left-0 md:right-0 md:left-auto mt-2 w-64 bg-white border border-gh-border rounded-md shadow-lg py-1 z-30 animate-in fade-in zoom-in-95 duration-100">
          <div class="px-3 py-1.5 text-[11px] font-semibold text-gh-muted border-b border-gh-border flex items-center justify-between">
            <span>Assign project</span>
            <button onclick={() => (isProjectsOpen = false)} class="hover:text-gh-text cursor-pointer">
              <X class="w-3.5 h-3.5" />
            </button>
          </div>
          <div class="p-2 border-b border-gh-border">
            <div class="relative">
              <Search class="w-3 h-3 absolute left-2 top-2 text-gh-muted" />
              <input
                type="text"
                placeholder="Filter projects..."
                bind:value={projectSearch}
                class="w-full text-xs pl-6 pr-2 py-1 bg-gh-subtle border border-gh-border rounded focus:outline-none focus:border-gh-link focus:bg-white"
              />
            </div>
          </div>
          <div class="max-h-48 overflow-y-auto py-1">
            <button
              type="button"
              onclick={() => {
                selectedProject = null;
                isProjectsOpen = false;
              }}
              class="w-full text-left px-3 py-1.5 text-xs flex items-center gap-2 hover:bg-gh-subtle cursor-pointer transition-colors text-gh-muted hover:text-gh-text"
            >
              <span class="w-3.5 flex justify-center shrink-0">
                {#if selectedProject === null}
                  <Check class="w-3.5 h-3.5 text-gh-link" />
                {/if}
              </span>
              <span>No project</span>
            </button>
            {#each filteredProjects as p (p.id)}
              {@const isSelected = selectedProject === p.id}
              <button
                type="button"
                onclick={() => {
                  selectedProject = p.id;
                  isProjectsOpen = false;
                }}
                class="w-full text-left px-3 py-1.5 text-xs flex items-center gap-2 hover:bg-gh-subtle cursor-pointer transition-colors"
              >
                <span class="w-3.5 flex justify-center shrink-0">
                  {#if isSelected}
                    <Check class="w-3.5 h-3.5 text-gh-link" />
                  {/if}
                </span>
                <span class="truncate flex-1 {isSelected ? 'font-semibold text-gh-text' : 'text-gh-text'}">{p.title}</span>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>