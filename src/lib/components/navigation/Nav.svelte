<script lang="ts">
  import { CircleDot, Tag, FolderKanban, Plus, Settings } from '@lucide/svelte';
  import type { NavTab, IssueView } from '$lib/types';

  let {
    currentNav,
    currentView,
    issuesCount,
    labelsCount,
    projectsCount,
    onNavChange,
    onNewIssue,
    onOpenSettings
  }: {
    currentNav: NavTab;
    currentView: IssueView;
    issuesCount: number;
    labelsCount: number;
    projectsCount: number;
    onNavChange: (nav: NavTab) => void;
    onNewIssue: () => void;
    onOpenSettings: () => void;
  } = $props();
</script>

<div class="flex items-center justify-between border-b border-gh-border pb-4 mb-6">
  <nav class="flex items-center gap-2">
    <button
      onclick={() => onNavChange('issues')}
      class="px-3 py-1.5 text-sm font-semibold rounded-md flex items-center gap-2 cursor-pointer transition-colors {currentNav === 'issues' ? 'bg-gh-subtle text-gh-text border border-gh-border' : 'text-gh-muted hover:text-gh-text'}"
    >
      <CircleDot class="w-4 h-4 text-gh-green" />
      <span>Issues</span>
      <span class="bg-gray-200 text-gh-text text-xs px-2 py-0.5 rounded-full font-normal">{issuesCount}</span>
    </button>

    <button
      onclick={() => onNavChange('labels')}
      class="px-3 py-1.5 text-sm font-semibold rounded-md flex items-center gap-2 cursor-pointer transition-colors {currentNav === 'labels' ? 'bg-gh-subtle text-gh-text border border-gh-border' : 'text-gh-muted hover:text-gh-text'}"
    >
      <Tag class="w-4 h-4 text-gh-muted" />
      <span>Labels</span>
      <span class="bg-gray-200 text-gh-text text-xs px-2 py-0.5 rounded-full font-normal">{labelsCount}</span>
    </button>

    <button
      onclick={() => onNavChange('projects')}
      class="px-3 py-1.5 text-sm font-semibold rounded-md flex items-center gap-2 cursor-pointer transition-colors {currentNav === 'projects' ? 'bg-gh-subtle text-gh-text border border-gh-border' : 'text-gh-muted hover:text-gh-text'}"
    >
      <FolderKanban class="w-4 h-4 text-gh-muted" />
      <span>Projects</span>
      <span class="bg-gray-200 text-gh-text text-xs px-2 py-0.5 rounded-full font-normal">{projectsCount}</span>
    </button>
  </nav>

  <div class="flex items-center gap-2">
    <button
      onclick={onOpenSettings}
      title="Settings (Ctrl+,)"
      class="border border-gh-border bg-gh-subtle hover:bg-gray-200 text-gh-text text-xs font-semibold px-2.5 py-1.5 rounded-md cursor-pointer flex items-center gap-1.5 transition-colors"
    >
      <Settings class="w-3.5 h-3.5 text-gh-muted" />
      <span>Settings</span>
    </button>

    {#if currentNav === 'issues' && currentView === 'list'}
      <button
        onclick={onNewIssue}
        class="bg-gh-green-btn hover:bg-gh-green text-white text-xs font-semibold px-3 py-1.5 rounded-md cursor-pointer flex items-center gap-1.5 transition-colors"
      >
        <Plus class="w-3.5 h-3.5" />
        <span>New issue</span>
      </button>
    {/if}
  </div>
</div>