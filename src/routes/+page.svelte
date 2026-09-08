<script lang="ts">
  import { onMount } from 'svelte';
  import { CircleHelp } from '@lucide/svelte';
  import { issuesApi, labelsApi, projectsApi, systemApi } from '$lib/api';
  import Nav from '$lib/components/navigation/Nav.svelte';
  import IssueList from '$lib/components/issues/IssueList.svelte';
  import IssueDetail from '$lib/components/issues/IssueDetail.svelte';
  import IssueNew from '$lib/components/issues/IssueNew.svelte';
  import LabelsView from '$lib/components/labels/LabelsView.svelte';
  import ProjectsView from '$lib/components/projects/ProjectsView.svelte';
  import KeybindsModal from '$lib/components/modals/KeybindsModal.svelte';
  import SettingsModal from '$lib/components/modals/SettingsModal.svelte';
  import type { Issue, Label, Project, NavTab, IssueView, IssueStatus, CreateIssuePayload } from '$lib/types';

  let issues = $state<Issue[]>([]);
  let labels = $state<Label[]>([]);
  let projects = $state<Project[]>([]);

  let currentNav = $state<NavTab>('issues');
  let currentView = $state<IssueView>('list');
  let selectedId = $state<number | null>(null);

  let filterTab = $state<IssueStatus>('open');
  let searchInputRef = $state<HTMLInputElement | null>(null);
  let highlightedIndex = $state<number>(0);
  let isKeybindsOpen = $state(false);
  let isSettingsOpen = $state(false);
  let autostart = $state(false);

  async function loadData() {
    try {
      const [fetchedIssues, fetchedLabels, fetchedProjects, fetchedAutostart] = await Promise.all([
        issuesApi.getAll(),
        labelsApi.getAll(),
        projectsApi.getAll(),
        systemApi.getAutostart()
      ]);
      issues = fetchedIssues;
      labels = fetchedLabels;
      projects = fetchedProjects;
      autostart = fetchedAutostart;
    } catch (e) {
      console.error(e);
    }
  }

  async function handleToggleAutostart(enabled: boolean) {
    try {
      await systemApi.setAutostart(enabled);
      autostart = enabled;
    } catch (e) {
      console.error(e);
    }
  }

  async function handleToggleStatus(id: number) {
    try {
      const updated = await issuesApi.toggleStatus(id);
      issues = issues.map((i) => (i.id === updated.id ? updated : i));
    } catch (e) {
      console.error(e);
    }
  }

  async function handleCreateIssue(data: CreateIssuePayload) {
    try {
      const created = await issuesApi.create(data);
      issues = [created, ...issues];
      selectedId = created.id;
      currentView = 'detail';
    } catch (e) {
      console.error(e);
    }
  }

  async function handleUpdateContent(id: number, title: string, body: string) {
    try {
      const updated = await issuesApi.updateContent(id, title, body);
      issues = issues.map((i) => (i.id === updated.id ? updated : i));
    } catch (e) {
      console.error(e);
    }
  }

  async function handleUpdateMeta(issue: Issue) {
    try {
      const pid = issue.project_id !== null && !isNaN(Number(issue.project_id)) ? Number(issue.project_id) : null;
      const updated = await issuesApi.updateMeta({
        id: issue.id,
        projectId: pid,
        labelIds: issue.labels.map((l) => Number(l.id))
      });
      issues = issues.map((i) => (i.id === updated.id ? updated : i));
    } catch (e) {
      console.error(e);
    }
  }

  async function handleCreateLabel(name: string, color: string, description: string) {
    try {
      const created = await labelsApi.create(name, color, description);
      labels = [...labels, created];
    } catch (e) {
      console.error(e);
    }
  }

  async function handleUpdateLabel(id: number, name: string, color: string, description: string) {
    try {
      const updated = await labelsApi.update(id, name, color, description);
      labels = labels.map((l) => (l.id === updated.id ? updated : l));
      await loadData();
    } catch (e) {
      console.error(e);
    }
  }

  async function handleDeleteLabel(id: number) {
    try {
      await labelsApi.delete(id);
      labels = labels.filter((l) => l.id !== id);
      await loadData();
    } catch (e) {
      console.error(e);
    }
  }

  async function handleCreateProject(title: string, description: string) {
    try {
      const created = await projectsApi.create(title, description);
      projects = [created, ...projects];
    } catch (e) {
      console.error(e);
    }
  }

  async function handleUpdateProject(id: number, title: string, description: string) {
    try {
      const updated = await projectsApi.update(id, title, description);
      projects = projects.map((p) => (p.id === updated.id ? updated : p));
      await loadData();
    } catch (e) {
      console.error(e);
    }
  }

  async function handleDeleteProject(id: number) {
    try {
      await projectsApi.delete(id);
      projects = projects.filter((p) => p.id !== id);
      await loadData();
    } catch (e) {
      console.error(e);
    }
  }

  let selectedIssue = $derived(issues.find((i) => i.id === selectedId));
  let visibleIssues = $derived(issues.filter((i) => i.status === filterTab));

  function handleKeydown(e: KeyboardEvent) {
    const isModifier = e.ctrlKey || e.metaKey;
    const isInputActive = ['INPUT', 'TEXTAREA', 'SELECT'].includes(document.activeElement?.tagName || '');

    if (isModifier && e.key === '/') {
      e.preventDefault();
      isKeybindsOpen = !isKeybindsOpen;
      return;
    }

    if (isModifier && e.key === ',') {
      e.preventDefault();
      isSettingsOpen = !isSettingsOpen;
      return;
    }

    if (e.key === 'Escape') {
      if (isKeybindsOpen) {
        isKeybindsOpen = false;
        return;
      }
      if (isSettingsOpen) {
        isSettingsOpen = false;
        return;
      }
      if (currentView !== 'list') {
        currentView = 'list';
        selectedId = null;
        return;
      }
    }

    if (isInputActive) {
      return;
    }

    if (isModifier && (e.key === 'n' || e.key === 'N')) {
      e.preventDefault();
      currentNav = 'issues';
      currentView = 'new';
      return;
    }

    if (isModifier && (e.key === 'k' || e.key === 'K')) {
      e.preventDefault();
      if (currentView === 'detail' && selectedId !== null) {
        handleToggleStatus(selectedId);
      } else if (currentView === 'list' && visibleIssues[highlightedIndex]) {
        handleToggleStatus(visibleIssues[highlightedIndex].id);
      }
      return;
    }

    if (isModifier && (e.key === 'l' || e.key === 'L')) {
      e.preventDefault();
      currentNav = 'issues';
      currentView = 'list';
      setTimeout(() => searchInputRef?.focus(), 10);
      return;
    }

    if (currentNav === 'issues' && currentView === 'list') {
      if (e.key === 'Tab') {
        e.preventDefault();
        filterTab = filterTab === 'open' ? 'closed' : 'open';
        highlightedIndex = 0;
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        if (highlightedIndex < visibleIssues.length - 1) {
          highlightedIndex += 1;
        }
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        if (highlightedIndex > 0) {
          highlightedIndex -= 1;
        }
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (visibleIssues[highlightedIndex]) {
          selectedId = visibleIssues[highlightedIndex].id;
          currentView = 'detail';
        }
      }
    }
  }

  onMount(loadData);
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="min-h-screen bg-white text-gh-text flex flex-col font-sans p-6 max-w-[1280px] mx-auto w-full relative">
  <Nav
    {currentNav}
    {currentView}
    issuesCount={issues.length}
    labelsCount={labels.length}
    projectsCount={projects.length}
    onNavChange={(nav) => {
      currentNav = nav;
      currentView = 'list';
      selectedId = null;
    }}
    onNewIssue={() => (currentView = 'new')}
    onOpenSettings={() => (isSettingsOpen = true)}
  />

  <main class="flex-1">
    {#if currentNav === 'issues'}
      {#if currentView === 'list'}
        <IssueList
          {issues}
          {labels}
          {projects}
          bind:filterTab
          bind:searchRef={searchInputRef}
          bind:highlightedIndex
          onSelectIssue={(id) => {
            selectedId = id;
            currentView = 'detail';
          }}
        />
      {:else if currentView === 'detail' && selectedIssue}
        <IssueDetail
          issue={selectedIssue}
          {labels}
          {projects}
          onToggleStatus={handleToggleStatus}
          onUpdateMeta={handleUpdateMeta}
          onUpdateContent={handleUpdateContent}
        />
      {:else if currentView === 'new'}
        <IssueNew
          {labels}
          {projects}
          onSubmit={handleCreateIssue}
          onCancel={() => (currentView = 'list')}
        />
      {/if}
    {:else if currentNav === 'labels'}
      <LabelsView
        {labels}
        onCreateLabel={handleCreateLabel}
        onUpdateLabel={handleUpdateLabel}
        onDeleteLabel={handleDeleteLabel}
      />
    {:else if currentNav === 'projects'}
      <ProjectsView
        {projects}
        onCreateProject={handleCreateProject}
        onUpdateProject={handleUpdateProject}
        onDeleteProject={handleDeleteProject}
      />
    {/if}
  </main>

  <button
    onclick={() => (isKeybindsOpen = true)}
    title="Keyboard shortcuts (Ctrl+/)"
    class="fixed bottom-5 right-5 z-40 p-2.5 rounded-full border border-gh-border bg-white hover:bg-gh-subtle text-gh-muted hover:text-gh-text shadow-sm transition cursor-pointer flex items-center justify-center"
  >
    <CircleHelp class="w-4 h-4" />
  </button>
</div>

<KeybindsModal isOpen={isKeybindsOpen} onClose={() => (isKeybindsOpen = false)} />

<SettingsModal
  isOpen={isSettingsOpen}
  {autostart}
  onClose={() => (isSettingsOpen = false)}
  onToggleAutostart={handleToggleAutostart}
/>