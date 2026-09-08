<script lang="ts">
  import { CircleDot, CheckCircle2, Search, FolderKanban, X } from '@lucide/svelte';
  import Dropdown from '$lib/components/ui/Dropdown.svelte';
  import LabelBadge from '$lib/components/ui/LabelBadge.svelte';
  import { formatRelativeTime } from '$lib/utils/format';
  import type { Issue, Label, Project, IssueStatus } from '$lib/types';

  let {
    issues,
    labels,
    projects,
    onSelectIssue,
    filterTab = $bindable<IssueStatus>('open'),
    searchRef = $bindable<HTMLInputElement | null>(null),
    highlightedIndex = $bindable<number>(0)
  }: {
    issues: Issue[];
    labels: Label[];
    projects: Project[];
    onSelectIssue: (id: number) => void;
    filterTab?: IssueStatus;
    searchRef?: HTMLInputElement | null;
    highlightedIndex?: number;
  } = $props();

  let search = $state('');
  let selectedLabelFilters = $state<number[]>([]);
  let selectedProjectFilters = $state<number[]>([]);

  let openCount = $derived(issues.filter((i) => i.status === 'open').length);
  let closedCount = $derived(issues.filter((i) => i.status === 'closed').length);

  let labelDropdownItems = $derived(
    labels.map((lbl) => ({ value: lbl.id, label: lbl.name, color: lbl.color }))
  );

  let projectDropdownItems = $derived(
    projects.map((prj) => ({ value: prj.id, label: prj.title }))
  );

  let filteredIssues = $derived(
    issues.filter((issue) => {
      if (issue.status !== filterTab) return false;
      if (search && !issue.title.toLowerCase().includes(search.toLowerCase())) return false;
      if (
        selectedLabelFilters.length > 0 &&
        !selectedLabelFilters.every((id) => issue.labels.some((l) => l.id === id))
      ) {
        return false;
      }
      if (
        selectedProjectFilters.length > 0 &&
        (issue.project_id === null || !selectedProjectFilters.includes(issue.project_id))
      ) {
        return false;
      }
      return true;
    })
  );

  let hasActiveFilters = $derived(
    selectedLabelFilters.length > 0 || selectedProjectFilters.length > 0 || search.length > 0
  );

  $effect(() => {
    if (highlightedIndex >= filteredIssues.length) {
      highlightedIndex = Math.max(0, filteredIssues.length - 1);
    }
  });
</script>

<div class="border border-gh-border rounded-md bg-white">
  <div class="bg-gh-subtle p-3 border-b border-gh-border rounded-t-md flex flex-wrap gap-3 items-center justify-between relative z-20">
    <div class="flex items-center gap-4 text-sm">
      <button
        class="flex items-center gap-1.5 py-1 cursor-pointer {filterTab === 'open' ? 'font-semibold text-gh-text' : 'text-gh-muted'}"
        onclick={() => (filterTab = 'open')}
      >
        <CircleDot class="w-4 h-4 text-gh-green" />
        <span>{openCount} Open</span>
      </button>
      <button
        class="flex items-center gap-1.5 py-1 cursor-pointer {filterTab === 'closed' ? 'font-semibold text-gh-text' : 'text-gh-muted'}"
        onclick={() => (filterTab = 'closed')}
      >
        <CheckCircle2 class="w-4 h-4 text-gh-purple" />
        <span>{closedCount} Closed</span>
      </button>
    </div>

    <div class="flex items-center gap-2 flex-wrap">
      <Dropdown
        placeholder="Label"
        multiple={true}
        items={labelDropdownItems}
        values={selectedLabelFilters}
        onchange={(vals) => {
          selectedLabelFilters = vals.map(Number);
        }}
      />

      <Dropdown
        placeholder="Project"
        multiple={true}
        items={projectDropdownItems}
        values={selectedProjectFilters}
        onchange={(vals) => {
          selectedProjectFilters = vals.map(Number);
        }}
      />

      <div class="relative w-48">
        <Search class="w-3.5 h-3.5 absolute left-2.5 top-2 text-gh-muted" />
        <input
          bind:this={searchRef}
          type="text"
          bind:value={search}
          placeholder="Filter issues (Ctrl+L)..."
          class="w-full text-xs pl-7 pr-3 py-1.5 bg-white border border-gh-border rounded-md focus:outline-none focus:border-gh-link"
        />
      </div>

      {#if hasActiveFilters}
        <button
          onclick={() => { selectedLabelFilters = []; selectedProjectFilters = []; search = ''; }}
          class="text-xs text-gh-muted hover:text-gh-red flex items-center gap-1 px-2 py-1.5 cursor-pointer"
        >
          <X class="w-3.5 h-3.5" />
          <span>Clear</span>
        </button>
      {/if}
    </div>
  </div>

  <div class="divide-y divide-gh-border relative z-0">
    {#if filteredIssues.length === 0}
      <div class="text-center py-12 text-gh-muted text-sm rounded-b-md">
        No issues matched your criteria.
      </div>
    {:else}
      {#each filteredIssues as issue, index (issue.id)}
        <div
          class="p-3 pl-3.5 flex items-start gap-3 transition cursor-pointer border-l-2 last:rounded-b-md {index === highlightedIndex ? 'border-l-gh-link bg-gh-subtle' : 'border-l-transparent hover:bg-gh-subtle'}"
          onclick={() => onSelectIssue(issue.id)}
          onmouseenter={() => (highlightedIndex = index)}
          role="button"
          tabindex="0"
          onkeydown={(e) => { if (e.key === 'Enter') onSelectIssue(issue.id); }}
        >
          <div class="mt-0.5">
            {#if issue.status === 'open'}
              <CircleDot class="w-4 h-4 text-gh-green" />
            {:else}
              <CheckCircle2 class="w-4 h-4 text-gh-purple" />
            {/if}
          </div>

          <div class="flex-1">
            <div class="flex flex-wrap items-center gap-2">
              <span class="text-left font-semibold text-gh-text hover:text-gh-link text-base leading-snug">
                {issue.title}
              </span>
              {#each issue.labels as lbl (lbl.id)}
                <LabelBadge label={lbl} size="xs" />
              {/each}
            </div>
            <div class="text-xs text-gh-muted mt-1 flex items-center gap-3">
              <span>#{issue.id} opened {formatRelativeTime(issue.created_at)}</span>
              {#if issue.project_title}
                <span class="flex items-center gap-1">
                  <FolderKanban class="w-3.5 h-3.5" />
                  {issue.project_title}
                </span>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>