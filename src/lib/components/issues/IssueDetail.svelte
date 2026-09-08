<script lang="ts">
  import {
    CircleDot,
    CheckCircle2,
    Tag,
    FolderKanban,
    Pencil,
    Check,
    X,
    Settings,
    Search,
    History
  } from '@lucide/svelte';
  import Markdown from '$lib/components/markdown/Markdown.svelte';
  import MarkdownEditor from '$lib/components/markdown/MarkdownEditor.svelte';
  import LabelBadge from '$lib/components/ui/LabelBadge.svelte';
  import IssueTimeline from '$lib/components/issues/IssueTimeline.svelte';
  import DiffModal from '$lib/components/modals/DiffModal.svelte';
  import { formatRelativeTime } from '$lib/utils/format';
  import { issuesApi } from '$lib/api';
  import type { Issue, IssueEvent, IssueRevision, Label, Project } from '$lib/types';

  let {
    issue,
    labels,
    projects,
    onToggleStatus,
    onUpdateMeta,
    onUpdateContent
  }: {
    issue: Issue;
    labels: Label[];
    projects: Project[];
    onToggleStatus: (id: number) => void;
    onUpdateMeta: (issue: Issue) => void;
    onUpdateContent: (id: number, title: string, body: string) => void;
  } = $props();

  let isEditingTitle = $state(false);
  let editTitleValue = $state('');

  let isEditingBody = $state(false);
  let editBodyValue = $state('');

  let isLabelsOpen = $state(false);
  let isProjectsOpen = $state(false);
  let isDiffModalOpen = $state(false);

  let labelSearch = $state('');
  let projectSearch = $state('');

  let labelsMenuRef = $state<HTMLDivElement | null>(null);
  let projectsMenuRef = $state<HTMLDivElement | null>(null);

  let events = $state<IssueEvent[]>([]);
  let revisions = $state<IssueRevision[]>([]);

  let filteredLabels = $derived(
    labels.filter((l) => l.name.toLowerCase().includes(labelSearch.toLowerCase()))
  );

  let filteredProjects = $derived(
    projects.filter((p) => p.title.toLowerCase().includes(projectSearch.toLowerCase()))
  );

  let currentProject = $derived(projects.find((p) => p.id === issue.project_id));

  async function loadData() {
    try {
      const [evs, revs] = await Promise.all([
        issuesApi.getEvents(issue.id),
        issuesApi.getRevisions(issue.id)
      ]);
      events = evs;
      revisions = revs;
    } catch (e) {
      console.error(e);
    }
  }

  $effect(() => {
    if (issue.id) {
      loadData();
    }
  });

  function startEditTitle() {
    editTitleValue = issue.title;
    isEditingTitle = true;
  }

  async function saveTitle() {
    if (!editTitleValue.trim()) return;
    await onUpdateContent(issue.id, editTitleValue.trim(), issue.body);
    isEditingTitle = false;
    await loadData();
  }

  function startEditBody() {
    editBodyValue = issue.body;
    isEditingBody = true;
  }

  async function saveBody() {
    await onUpdateContent(issue.id, issue.title, editBodyValue);
    isEditingBody = false;
    await loadData();
  }

  async function handleToggleStatus() {
    await onToggleStatus(issue.id);
    await loadData();
  }

  async function toggleLabel(lbl: Label) {
    const exists = issue.labels.some((l) => l.id === lbl.id);
    if (exists) {
      issue.labels = issue.labels.filter((l) => l.id !== lbl.id);
    } else {
      issue.labels = [...issue.labels, lbl];
    }
    await onUpdateMeta(issue);
    await loadData();
  }

  async function selectProject(projectId: number | null) {
    issue.project_id = projectId;
    await onUpdateMeta(issue);
    isProjectsOpen = false;
    await loadData();
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

<div class="flex flex-col gap-6">
  <div class="border-b border-gh-border pb-4">
    <div class="flex items-start justify-between gap-4">
      {#if isEditingTitle}
        <div class="flex-1 flex items-center gap-2">
          <input
            type="text"
            bind:value={editTitleValue}
            onkeydown={(e) => {
              if (e.key === 'Enter') saveTitle();
              if (e.key === 'Escape') isEditingTitle = false;
            }}
            class="flex-1 text-2xl font-normal text-gh-text px-3 py-1 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link"
          />
          <button
            onclick={saveTitle}
            class="bg-gh-green-btn hover:bg-gh-green text-white text-xs font-semibold px-2.5 py-1.5 rounded-md cursor-pointer flex items-center gap-1 shrink-0"
          >
            <Check class="w-3.5 h-3.5" />
            <span>Save</span>
          </button>
          <button
            onclick={() => (isEditingTitle = false)}
            class="border border-gh-border bg-gh-subtle hover:bg-gray-200 text-gh-text text-xs font-semibold px-2.5 py-1.5 rounded-md cursor-pointer flex items-center gap-1 shrink-0"
          >
            <X class="w-3.5 h-3.5" />
            <span>Cancel</span>
          </button>
        </div>
      {:else}
        <div class="flex items-center gap-3 flex-wrap flex-1">
          <h1 class="text-3xl font-normal text-gh-text leading-tight">
            {issue.title}
            <span class="text-gh-muted font-light">#{issue.id}</span>
          </h1>
          <button
            onclick={startEditTitle}
            class="text-xs text-gh-muted hover:text-gh-link border border-gh-border bg-gh-subtle px-2.5 py-1 rounded-md cursor-pointer flex items-center gap-1 shrink-0"
          >
            <Pencil class="w-3 h-3" />
            <span>Edit</span>
          </button>
        </div>
      {/if}

      <div class="shrink-0">
        <button
          onclick={handleToggleStatus}
          class="px-3 py-1.5 text-xs font-medium rounded-md border cursor-pointer transition whitespace-nowrap flex items-center gap-1.5 {issue.status === 'open' ? 'text-gh-purple border-gh-border bg-gh-subtle hover:bg-gh-purple hover:text-white hover:border-gh-purple' : 'text-gh-green border-gh-border bg-gh-subtle hover:bg-gh-green-btn hover:text-white hover:border-gh-green-btn'}"
        >
          {#if issue.status === 'open'}
            <CheckCircle2 class="w-3.5 h-3.5" />
            <span>Close issue</span>
          {:else}
            <CircleDot class="w-3.5 h-3.5" />
            <span>Reopen issue</span>
          {/if}
        </button>
      </div>
    </div>

    <div class="flex items-center gap-2 mt-3 text-xs text-gh-muted">
      {#if issue.status === 'open'}
        <span class="bg-gh-green text-white px-3 py-1 rounded-full font-medium flex items-center gap-1.5">
          <CircleDot class="w-3.5 h-3.5 text-white" />
          Open
        </span>
      {:else}
        <span class="bg-gh-purple text-white px-3 py-1 rounded-full font-medium flex items-center gap-1.5">
          <CheckCircle2 class="w-3.5 h-3.5 text-white" />
          Closed
        </span>
      {/if}
      <span>Opened {formatRelativeTime(issue.created_at)}</span>
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-4 gap-6 items-start">
    <div class="md:col-span-3 flex flex-col gap-6">
      <div class="border border-gh-border rounded-md overflow-hidden bg-white shadow-xs">
        <div class="bg-gh-subtle border-b border-gh-border px-4 py-2.5 text-xs text-gh-muted flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="font-semibold text-gh-text">Description</span>
            {#if revisions.length > 1}
              <button
                type="button"
                onclick={() => (isDiffModalOpen = true)}
                class="text-gh-muted hover:text-gh-link flex items-center gap-1 cursor-pointer transition-colors"
                title="View edit history"
              >
                <span>•</span>
                <History class="w-3 h-3" />
                <span>edited ({revisions.length - 1})</span>
              </button>
            {/if}
          </div>

          {#if !isEditingBody}
            <button
              onclick={startEditBody}
              class="text-gh-muted hover:text-gh-link flex items-center gap-1 cursor-pointer font-medium"
            >
              <Pencil class="w-3 h-3" />
              <span>Edit</span>
            </button>
          {/if}
        </div>

        <div class="p-6 bg-white">
          {#if isEditingBody}
            <div class="flex flex-col gap-3">
              <MarkdownEditor
                bind:value={editBodyValue}
                rows={12}
                onsubmit={saveBody}
              />

              <div class="flex items-center justify-end gap-2">
                <button
                  type="button"
                  onclick={() => (isEditingBody = false)}
                  class="px-3 py-1.5 text-xs font-semibold rounded-md border border-gh-border bg-gh-subtle hover:bg-gray-200 cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onclick={saveBody}
                  class="bg-gh-green-btn hover:bg-gh-green text-white text-xs font-semibold px-3 py-1.5 rounded-md cursor-pointer"
                >
                  Update comment
                </button>
              </div>
            </div>
          {:else}
            <Markdown content={issue.body} />
          {/if}
        </div>
      </div>

      <IssueTimeline {events} />
    </div>

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
          {#each issue.labels as lbl (lbl.id)}
            <LabelBadge label={lbl} size="xs" />
          {/each}
          {#if issue.labels.length === 0}
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
                  {@const isChecked = issue.labels.some((l) => l.id === lbl.id)}
                  <button
                    type="button"
                    onclick={() => toggleLabel(lbl)}
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
                onclick={() => selectProject(null)}
                class="w-full text-left px-3 py-1.5 text-xs flex items-center gap-2 hover:bg-gh-subtle cursor-pointer transition-colors text-gh-muted hover:text-gh-text"
              >
                <span class="w-3.5 flex justify-center shrink-0">
                  {#if issue.project_id === null}
                    <Check class="w-3.5 h-3.5 text-gh-link" />
                  {/if}
                </span>
                <span>No project</span>
              </button>
              {#each filteredProjects as p (p.id)}
                {@const isSelected = issue.project_id === p.id}
                <button
                  type="button"
                  onclick={() => selectProject(p.id)}
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
</div>

<DiffModal
  isOpen={isDiffModalOpen}
  {revisions}
  onClose={() => (isDiffModalOpen = false)}
/>