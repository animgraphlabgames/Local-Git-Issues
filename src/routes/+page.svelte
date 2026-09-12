<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { CircleHelp } from '@lucide/svelte';
  import { issuesApi, labelsApi, projectsApi, dependenciesApi, systemApi } from '$lib/api';
  import Nav from '$lib/components/navigation/Nav.svelte';
  import IssueList from '$lib/components/issues/IssueList.svelte';
  import IssueDetail from '$lib/components/issues/IssueDetail.svelte';
  import IssueNew from '$lib/components/issues/IssueNew.svelte';
  import LabelsView from '$lib/components/labels/LabelsView.svelte';
  import ProjectsView from '$lib/components/projects/ProjectsView.svelte';
  import DependenciesView from '$lib/components/dependencies/DependenciesView.svelte';
  import KeybindsModal from '$lib/components/modals/KeybindsModal.svelte';
  import SettingsModal from '$lib/components/modals/SettingsModal.svelte';
  import type { Issue, Label, Project, Dependency, NavTab, IssueView, IssueStatus, CreateIssuePayload } from '$lib/types';

  let issues = $state<Issue[]>([]);
  let labels = $state<Label[]>([]);
  let projects = $state<Project[]>([]);
  let dependencies = $state<Dependency[]>([]);
  let currentNav = $state<NavTab>('issues');
  let currentView = $state<IssueView>('list');
  let selectedId = $state<number | null>(null);

  let filterTab = $state<IssueStatus>(
    typeof window !== 'undefined' && JSON.parse(localStorage.getItem('local_issues_filters') || '{}').tab === 'closed' ? 'closed' : 'open'
  );
  let searchInputRef = $state<HTMLInputElement | null>(null);
  let isKeybindsOpen = $state(false);
  let isSettingsOpen = $state(false);
  let autostart = $state(false);
  let selectedIssue = $derived(issues.find((i) => i.id === selectedId));
  let dependenciesUnreadCount = $derived(dependencies.filter((d) => d.has_update).length);

  async function loadData() {
    try {
      const [fetchedIssues, fetchedLabels, fetchedProjects, fetchedDeps, fetchedAutostart] = await Promise.all([
        issuesApi.getAll(),
        labelsApi.getAll(),
        projectsApi.getAll(),
        dependenciesApi.getAll(),
        systemApi.getAutostart()
      ]);
      issues = fetchedIssues;
      labels = fetchedLabels;
      projects = fetchedProjects;
      dependencies = fetchedDeps;
      autostart = fetchedAutostart;
    } catch (e) {
      console.error(e);
    }
  }

  async function handleCheckDependencies() {
    try {
      dependencies = await dependenciesApi.checkAll();
    } catch (e) {
      console.error(e);
    }
  }

  async function handleAcknowledgeDependency(id: number) {
    try {
      const updated = await dependenciesApi.acknowledge(id);
      dependencies = dependencies.map((d) => (d.id === updated.id ? updated : d));
    } catch (e) {
      console.error(e);
    }
  }

  async function handleAddDependency(name: string, target: string) {
    try {
      dependencies = [await dependenciesApi.add(name, target), ...dependencies];
    } catch (e) {
      console.error(e);
    }
  }

  async function handleDeleteDependency(id: number) {
    try {
      await dependenciesApi.delete(id);
      dependencies = dependencies.filter((d) => d.id !== id);
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
      const updated = await issuesApi.updateMeta({
        id: issue.id,
        projectId: issue.project_id !== null && !isNaN(Number(issue.project_id)) ? Number(issue.project_id) : null,
        labelIds: issue.labels.map((l) => Number(l.id))
      });
      issues = issues.map((i) => (i.id === updated.id ? updated : i));
    } catch (e) {
      console.error(e);
    }
  }

  async function handleCreateLabel(name: string, color: string, description: string) {
    try {
      labels = [...labels, await labelsApi.create(name, color, description)];
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
      projects = [await projectsApi.create(title, description), ...projects];
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

  function handleSelectIssue(id: number) {
    selectedId = id;
    currentNav = 'issues';
    currentView = 'detail';
  }

  function handleKeydown(e: KeyboardEvent) {
    const isModifier = e.ctrlKey || e.metaKey;
    if (['INPUT', 'TEXTAREA', 'SELECT'].includes(document.activeElement?.tagName || '')) return;

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
      if (isKeybindsOpen) return void (isKeybindsOpen = false);
      if (isSettingsOpen) return void (isSettingsOpen = false);
      if (currentView !== 'list') {
        currentView = 'list';
        selectedId = null;
      }
      return;
    }

    if (isModifier && (e.key === 'n' || e.key === 'N')) {
      e.preventDefault();
      currentNav = 'issues';
      currentView = 'new';
      return;
    }

    if (isModifier && (e.key === 'k' || e.key === 'K')) {
      if (currentView === 'detail' && selectedId !== null) {
        e.preventDefault();
        handleToggleStatus(selectedId);
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
  }

  onMount(() => {
    loadData();
    const unlisten = listen('dependencies-updated', async () => {
      dependencies = await dependenciesApi.getAll();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="min-h-screen bg-white text-gh-text flex flex-col font-sans p-6 max-w-[1280px] mx-auto w-full relative">
  <Nav
    {currentNav}
    {currentView}
    issuesCount={issues.filter((i) => i.status === 'open').length}
    labelsCount={labels.length}
    projectsCount={projects.length}
    dependenciesCount={dependencies.length}
    {dependenciesUnreadCount}
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
          onSelectIssue={handleSelectIssue}
          onToggleStatus={handleToggleStatus}
        />
      {:else if currentView === 'detail' && selectedIssue}
        <IssueDetail
          issue={selectedIssue}
          {issues}
          {labels}
          {projects}
          onToggleStatus={handleToggleStatus}
          onUpdateMeta={handleUpdateMeta}
          onUpdateContent={handleUpdateContent}
          onSelectIssue={handleSelectIssue}
        />
      {:else if currentView === 'new'}
        <IssueNew
          {issues}
          {labels}
          {projects}
          onSubmit={handleCreateIssue}
          onCancel={() => (currentView = 'list')}
          onSelectIssue={handleSelectIssue}
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
        {issues}
        onCreateProject={handleCreateProject}
        onUpdateProject={handleUpdateProject}
        onDeleteProject={handleDeleteProject}
      />
    {:else if currentNav === 'dependencies'}
      <DependenciesView
        {dependencies}
        onCheckAll={handleCheckDependencies}
        onAcknowledge={handleAcknowledgeDependency}
        onAdd={handleAddDependency}
        onDelete={handleDeleteDependency}
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