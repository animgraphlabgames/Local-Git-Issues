<script lang="ts">
  import { ChevronDown, Check } from '@lucide/svelte';

  export interface DropdownItem {
    value: string | number;
    label: string;
    color?: string;
  }

  let {
    items,
    value = null,
    values = [],
    multiple = false,
    placeholder = 'Select',
    title,
    clearable = false,
    align = 'left',
    onselect,
    onchange
  }: {
    items: DropdownItem[];
    value?: string | number | null;
    values?: (string | number)[];
    multiple?: boolean;
    placeholder?: string;
    title?: string;
    clearable?: boolean;
    align?: 'left' | 'right';
    onselect?: (value: string | number | null) => void;
    onchange?: (values: (string | number)[]) => void;
  } = $props();

  let isOpen = $state(false);
  let containerRef = $state<HTMLDivElement | null>(null);

  let selectedItem = $derived(items.find((item) => item.value === value));

  let displayLabel = $derived.by(() => {
    if (multiple) {
      if (values.length === 0) return placeholder;
      if (values.length === 1) {
        const match = items.find((i) => i.value === values[0]);
        return match ? match.label : placeholder;
      }
      return `${placeholder} (${values.length})`;
    }
    return selectedItem ? selectedItem.label : placeholder;
  });

  let displayColor = $derived.by(() => {
    if (multiple) {
      if (values.length === 1) {
        const match = items.find((i) => i.value === values[0]);
        return match?.color;
      }
      return undefined;
    }
    return selectedItem?.color;
  });

  let hasSelection = $derived(multiple ? values.length > 0 : value !== null);

  let headerTitle = $derived(
    title ?? (multiple ? `Filter by ${placeholder.toLowerCase()}` : undefined)
  );

  function handleItemClick(itemValue: string | number) {
    if (multiple) {
      const next = values.includes(itemValue)
        ? values.filter((v) => v !== itemValue)
        : [...values, itemValue];
      onchange?.(next);
    } else {
      onselect?.(itemValue);
      isOpen = false;
    }
  }

  function handleClear() {
    if (multiple) {
      onchange?.([]);
    } else {
      onselect?.(null);
      isOpen = false;
    }
  }

  function handleWindowClick(e: MouseEvent) {
    if (isOpen && containerRef && !containerRef.contains(e.target as Node)) {
      isOpen = false;
    }
  }

  function handleWindowKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      isOpen = false;
    }
  }
</script>

<svelte:window onclick={handleWindowClick} onkeydown={handleWindowKeydown} />

<div class="relative inline-block text-left" bind:this={containerRef}>
  <button
    type="button"
    onclick={() => (isOpen = !isOpen)}
    class="text-xs pl-2.5 pr-2 py-1.5 bg-white hover:bg-gh-subtle border border-gh-border rounded-md focus:outline-none focus:border-gh-link cursor-pointer inline-flex items-center gap-2 transition-colors whitespace-nowrap {hasSelection ? 'font-semibold text-gh-text' : 'text-gh-muted hover:text-gh-text'}"
  >
    {#if displayColor}
      <span class="w-2 h-2 rounded-full shrink-0" style="background-color: {displayColor};"></span>
    {/if}
    <span class="truncate max-w-[260px]">{displayLabel}</span>
    <ChevronDown class="w-3.5 h-3.5 text-gh-muted transition-transform duration-150 {isOpen ? 'rotate-180' : ''}" />
  </button>

  {#if isOpen}
    <div class="absolute {align === 'right' ? 'right-0' : 'left-0'} mt-1 min-w-[200px] max-w-[320px] bg-white border border-gh-border rounded-md shadow-lg py-1 z-50 animate-in fade-in zoom-in-95 duration-100">
      {#if headerTitle}
        <div class="px-3 py-1.5 text-[11px] font-semibold text-gh-muted border-b border-gh-border flex items-center justify-between">
          <span>{headerTitle}</span>
          {#if clearable && hasSelection}
            <button
              type="button"
              onclick={handleClear}
              class="text-[11px] font-normal text-gh-muted hover:text-gh-red cursor-pointer"
            >
              Clear
            </button>
          {/if}
        </div>
      {/if}

      <div class="max-h-56 overflow-y-auto py-0.5">
        {#each items as item (item.value)}
          {@const isChecked = multiple ? values.includes(item.value) : item.value === value}
          <button
            type="button"
            onclick={() => handleItemClick(item.value)}
            class="w-full text-left px-3 py-1.5 text-xs flex items-center gap-2 hover:bg-gh-subtle cursor-pointer transition-colors {isChecked ? 'font-semibold text-gh-text' : 'text-gh-text'}"
          >
            <span class="w-3.5 flex justify-center shrink-0">
              {#if isChecked}
                <Check class="w-3.5 h-3.5 text-gh-link" />
              {/if}
            </span>
            {#if item.color}
              <span class="w-2.5 h-2.5 rounded-full shrink-0" style="background-color: {item.color};"></span>
            {/if}
            <span class="truncate">{item.label}</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>