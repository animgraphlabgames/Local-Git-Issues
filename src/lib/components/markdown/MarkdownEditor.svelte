<script lang="ts">
  import {
    Bold,
    Italic,
    Strikethrough,
    Code,
    Link,
    List,
    ListOrdered,
    Quote,
    Image as ImageIcon,
    Loader2,
    CircleDot,
    CheckCircle2,
    FolderKanban
  } from '@lucide/svelte';
  import Markdown from './Markdown.svelte';
  import LabelBadge from '$lib/components/ui/LabelBadge.svelte';
  import { convertToWebp } from '$lib/utils/image';
  import { storeAttachment } from '$lib/utils/attachments';
  import { formatRelativeTime, getPlainTextExcerpt } from '$lib/utils/format';
  import type { Issue } from '$lib/types';

  let {
    value = $bindable(''),
    placeholder = '',
    rows = 12,
    issues = [],
    onsubmit,
    onSelectIssue
  }: {
    value: string;
    placeholder?: string;
    rows?: number;
    issues?: Issue[];
    onsubmit?: () => void;
    onSelectIssue?: (id: number) => void;
  } = $props();

  let activeTab = $state<'write' | 'preview'>('write');
  let textareaRef = $state<HTMLTextAreaElement | null>(null);
  let fileInputRef = $state<HTMLInputElement | null>(null);
  let isProcessingImage = $state(false);

  let mentionSearch = $state<string | null>(null);
  let mentionIndex = $state(0);
  let mentionPosition = $state<{ top: number; left: number; placeAbove: boolean }>({ top: 0, left: 0, placeAbove: false });
  let triggerRange = $state<{ start: number; end: number }>({ start: 0, end: 0 });

  let matchingIssues = $derived(mentionSearch === null ? [] : issues.filter((i) => i.id.toString().includes(mentionSearch!) || i.title.toLowerCase().includes(mentionSearch!)).slice(0, 5));
  let highlightedMentionIssue = $derived(matchingIssues[mentionIndex] ?? null);

  function getCaretCoordinates(element: HTMLTextAreaElement, position: number) {
    const div = document.createElement('div');
    const styles = window.getComputedStyle(element);
    const properties = [
      'boxSizing', 'width', 'height', 'overflowX', 'overflowY',
      'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth',
      'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
      'fontStyle', 'fontVariant', 'fontWeight', 'fontSize',
      'lineHeight', 'fontFamily', 'textAlign', 'letterSpacing', 'wordSpacing'
    ];
    for (const prop of properties) {
      div.style.setProperty(prop, styles.getPropertyValue(prop));
    }
    div.style.position = 'absolute';
    div.style.top = '0px';
    div.style.left = '-9999px';
    div.style.visibility = 'hidden';
    div.style.whiteSpace = 'pre-wrap';
    div.style.wordWrap = 'break-word';

    div.textContent = element.value.substring(0, position);
    const span = document.createElement('span');
    span.textContent = element.value.substring(position) || '.';
    div.appendChild(span);

    document.body.appendChild(div);
    const top = span.offsetTop - element.scrollTop;
    const left = span.offsetLeft - element.scrollLeft;
    document.body.removeChild(div);
    return { top, left };
  }

  function updateMentionState() {
    if (!textareaRef || activeTab !== 'write') {
      mentionSearch = null;
      return;
    }
    const cursor = textareaRef.selectionStart;
    const match = /(?:^|\s)#([a-zA-Z0-9_-]*)$/.exec(value.substring(0, cursor));

    if (match) {
      const query = match[1];
      const start = cursor - query.length - 1;
      triggerRange = { start, end: cursor };

      const newSearch = query.toLowerCase();
      if (mentionSearch !== newSearch) {
        mentionSearch = newSearch;
        mentionIndex = 0;
      }

      const rect = textareaRef.getBoundingClientRect();
      const coords = getCaretCoordinates(textareaRef, start);
      const absY = rect.top + coords.top;
      const placeAbove = absY + 340 > window.innerHeight;

      mentionPosition = {
        top: placeAbove ? absY - 8 : absY + 24,
        left: Math.max(12, Math.min(window.innerWidth - 400, rect.left + coords.left)),
        placeAbove
      };
    } else {
      mentionSearch = null;
    }
  }

  function selectMention(item: Issue) {
    if (!textareaRef) return;
    const before = value.substring(0, triggerRange.start);
    const after = value.substring(triggerRange.end);
    const replacement = `#${item.id} `;
    value = `${before}${replacement}${after}`;
    const nextCursor = triggerRange.start + replacement.length;
    mentionSearch = null;
    setTimeout(() => {
      textareaRef?.focus();
      textareaRef?.setSelectionRange(nextCursor, nextCursor);
    }, 10);
  }

  function applyWrap(before: string, after: string, defaultText = 'text') {
    if (!textareaRef) return;
    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const selected = value.substring(start, end);
    const replacement = `${before}${selected || defaultText}${after}`;

    textareaRef.focus();
    textareaRef.setRangeText(replacement, start, end, 'select');
    textareaRef.setSelectionRange(
      start + before.length,
      start + before.length + (selected ? selected.length : defaultText.length)
    );
    value = textareaRef.value;
  }

  function applyPrefix(prefix: string) {
    if (!textareaRef) return;
    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const lineStart = value.lastIndexOf('\n', start - 1) + 1;
    const lineEnd = value.indexOf('\n', end) === -1 ? value.length : value.indexOf('\n', end);
    const transformed = value
      .substring(lineStart, lineEnd)
      .split('\n')
      .map((l) => `${prefix}${l}`)
      .join('\n');

    textareaRef.focus();
    textareaRef.setRangeText(transformed, lineStart, lineEnd, 'select');
    textareaRef.setSelectionRange(lineStart, lineStart + transformed.length);
    value = textareaRef.value;
  }

  function applyLink() {
    if (!textareaRef) return;
    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const selected = value.substring(start, end);
    const isUrl = selected.startsWith('http://') || selected.startsWith('https://');
    const replacement = isUrl ? `[url](${selected})` : selected ? `[${selected}](url)` : `[link](url)`;

    textareaRef.focus();
    textareaRef.setRangeText(replacement, start, end, 'select');
    textareaRef.setSelectionRange(
      start + (isUrl ? 1 : selected ? selected.length + 3 : 1),
      start + (isUrl ? 4 : selected ? selected.length + 6 : 5)
    );
    value = textareaRef.value;
  }

  function applyCodeBlock() {
    if (!textareaRef) return;
    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const selected = value.substring(start, end) || 'code';
    const beforeNewline = start > 0 && value[start - 1] !== '\n' ? '\n' : '';
    const afterNewline = end < value.length && value[end] !== '\n' ? '\n' : '';
    const replacement = `${beforeNewline}\`\`\`\n${selected}\n\`\`\`${afterNewline}`;

    textareaRef.focus();
    textareaRef.setRangeText(replacement, start, end, 'select');
    if (!value.substring(start, end)) {
      const cursor = start + beforeNewline.length + 4;
      textareaRef.setSelectionRange(cursor, cursor + 4);
    }
    value = textareaRef.value;
  }

  export async function insertImageFile(file: File | Blob) {
    isProcessingImage = true;
    try {
      const webpBlob = await convertToWebp(file, 0.8);
      const name = await storeAttachment(webpBlob);
      const markdown = `![image](attachment:${name})\n`;

      if (textareaRef) {
        const start = textareaRef.selectionStart;
        const end = textareaRef.selectionEnd;
        textareaRef.focus();
        textareaRef.setRangeText(markdown, start, end, 'end');
        value = textareaRef.value;
      } else {
        value = value ? `${value}\n${markdown}` : markdown;
      }
    } catch (err) {
      console.error(err);
    } finally {
      isProcessingImage = false;
    }
  }

  async function handlePaste(e: ClipboardEvent) {
    const item = Array.from(e.clipboardData?.items || []).find(
      (i) => i.kind === 'file' && i.type.startsWith('image/')
    );
    if (!item) return;
    const file = item.getAsFile();
    if (file) {
      e.preventDefault();
      e.stopPropagation();
      await insertImageFile(file);
    }
  }

  async function handleFileInputChange(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (file) {
      await insertImageFile(file);
      input.value = '';
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (mentionSearch !== null && matchingIssues.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        mentionIndex = (mentionIndex + 1) % matchingIssues.length;
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        mentionIndex = (mentionIndex - 1 + matchingIssues.length) % matchingIssues.length;
        return;
      }
      if (e.key === 'Enter' || e.key === 'Tab') {
        e.preventDefault();
        selectMention(matchingIssues[mentionIndex]);
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        mentionSearch = null;
        return;
      }
    }

    const isMod = e.ctrlKey || e.metaKey;

    if (isMod && e.shiftKey && (e.key === 'p' || e.key === 'P')) {
      e.preventDefault();
      activeTab = activeTab === 'write' ? 'preview' : 'write';
      return;
    }

    if (isMod && e.key === 'Enter') {
      e.preventDefault();
      onsubmit?.();
      return;
    }

    if (isMod && (e.key === 'b' || e.key === 'B')) {
      e.preventDefault();
      applyWrap('**', '**', 'bold text');
      return;
    }

    if (isMod && (e.key === 'i' || e.key === 'I')) {
      e.preventDefault();
      applyWrap('*', '*', 'italic text');
      return;
    }

    if (isMod && (e.key === 'k' || e.key === 'K')) {
      e.preventDefault();
      applyLink();
      return;
    }

    if (isMod && (e.key === 'e' || e.key === 'E')) {
      e.preventDefault();
      applyWrap('`', '`', 'code');
      return;
    }

    if (isMod && e.shiftKey && (e.key === 'x' || e.key === 'X')) {
      e.preventDefault();
      applyWrap('~~', '~~', 'strikethrough text');
      return;
    }

    if (isMod && e.shiftKey && (e.key === 'c' || e.key === 'C')) {
      e.preventDefault();
      applyCodeBlock();
      return;
    }

    if (isMod && e.shiftKey && e.key === '8') {
      e.preventDefault();
      applyPrefix('- ');
      return;
    }

    if (isMod && e.shiftKey && e.key === '7') {
      e.preventDefault();
      applyPrefix('1. ');
      return;
    }

    if (isMod && e.shiftKey && (e.key === '.' || e.key === '>')) {
      e.preventDefault();
      applyPrefix('> ');
      return;
    }

    if (e.key === 'Tab') {
      e.preventDefault();
      if (!textareaRef) return;
      const start = textareaRef.selectionStart;
      const end = textareaRef.selectionEnd;

      if (e.shiftKey) {
        const lineStart = textareaRef.value.lastIndexOf('\n', start - 1) + 1;
        if (textareaRef.value.substring(lineStart, lineStart + 2) === '  ') {
          textareaRef.setRangeText('', lineStart, lineStart + 2, 'end');
          value = textareaRef.value;
        }
      } else {
        textareaRef.setRangeText('  ', start, end, 'end');
        value = textareaRef.value;
      }
    }
  }
</script>

<svelte:window onresize={updateMentionState} onscroll={updateMentionState} />

<div class="border border-gh-border rounded-md overflow-hidden bg-white">
  <input
    type="file"
    accept="image/*"
    bind:this={fileInputRef}
    onchange={handleFileInputChange}
    class="hidden"
  />

  <div class="bg-gh-subtle border-b border-gh-border px-3 pt-2 flex items-center justify-between">
    <div class="flex gap-1">
      <button
        type="button"
        class="px-3 py-1.5 text-xs font-semibold rounded-t-md cursor-pointer {activeTab === 'write' ? 'bg-white border-t border-x border-gh-border -mb-px text-gh-text' : 'text-gh-muted hover:text-gh-text'}"
        onclick={() => (activeTab = 'write')}
      >
        Write
      </button>
      <button
        type="button"
        class="px-3 py-1.5 text-xs font-semibold rounded-t-md cursor-pointer {activeTab === 'preview' ? 'bg-white border-t border-x border-gh-border -mb-px text-gh-text' : 'text-gh-muted hover:text-gh-text'}"
        onclick={() => (activeTab = 'preview')}
      >
        Preview
      </button>
    </div>

    {#if activeTab === 'write'}
      <div class="flex items-center gap-1 pb-1.5 text-gh-muted">
        {#if isProcessingImage}
          <span class="flex items-center gap-1 text-[11px] text-gh-muted pr-2">
            <Loader2 class="w-3 h-3 animate-spin" />
            <span>Processing WebP...</span>
          </span>
        {/if}
        <button
          type="button"
          onclick={() => fileInputRef?.click()}
          class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
          title="Upload or attach image"
        >
          <ImageIcon class="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onclick={() => applyWrap('**', '**', 'bold text')}
          class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
          title="Bold (Ctrl+B)"
        >
          <Bold class="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onclick={() => applyWrap('*', '*', 'italic text')}
          class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
          title="Italic (Ctrl+I)"
        >
          <Italic class="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onclick={() => applyWrap('~~', '~~', 'strikethrough text')}
          class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
          title="Strikethrough (Ctrl+Shift+X)"
        >
          <Strikethrough class="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onclick={applyLink}
          class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
          title="Insert link (Ctrl+K)"
        >
          <Link class="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onclick={() => applyWrap('`', '`', 'code')}
          class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
          title="Inline code (Ctrl+E)"
        >
          <Code class="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onclick={() => applyPrefix('- ')}
          class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
          title="Unordered list (Ctrl+Shift+8)"
        >
          <List class="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onclick={() => applyPrefix('1. ')}
          class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
          title="Ordered list (Ctrl+Shift+7)"
        >
          <ListOrdered class="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onclick={() => applyPrefix('> ')}
          class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
          title="Blockquote (Ctrl+Shift+>)"
        >
          <Quote class="w-3.5 h-3.5" />
        </button>
      </div>
    {/if}
  </div>

  {#if activeTab === 'write'}
    <div>
      <textarea
        bind:this={textareaRef}
        {rows}
        {placeholder}
        bind:value
        oninput={updateMentionState}
        onclick={updateMentionState}
        onkeyup={(e) => {
          if (!['ArrowUp', 'ArrowDown', 'Enter', 'Tab', 'Escape'].includes(e.key)) {
            updateMentionState();
          }
        }}
        onscroll={updateMentionState}
        onkeydown={handleKeydown}
        onpaste={handlePaste}
        class="w-full font-mono text-sm p-3 focus:outline-none border-0 block resize-y"
      ></textarea>
    </div>
  {:else}
    <div class="p-4 min-h-[250px]">
      {#if value.trim()}
        <Markdown content={value} {issues} {onSelectIssue} />
      {:else}
        <p class="text-gh-muted text-xs">Nothing to preview</p>
      {/if}
    </div>
  {/if}
</div>

{#if mentionSearch !== null && matchingIssues.length > 0}
  <div
    class="fixed z-50 w-96 bg-white border border-gh-border rounded-lg shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-100 flex flex-col select-none {mentionPosition.placeAbove ? '-translate-y-full' : ''}"
    style="top: {mentionPosition.top}px; left: {mentionPosition.left}px;"
  >
    <div class="bg-gh-subtle border-b border-gh-border px-3 py-1.5 text-[11px] font-semibold text-gh-muted flex items-center justify-between">
      <span>Reference an issue</span>
      <kbd class="text-[10px] bg-white border border-gh-border px-1.5 py-0.5 rounded font-mono shadow-2xs">↑↓ navigate</kbd>
    </div>

    <div class="max-h-48 overflow-y-auto divide-y divide-gh-border/50">
      {#each matchingIssues as issue, idx (issue.id)}
        <div
          role="button"
          tabindex="-1"
          onmousedown={(e) => {
            e.preventDefault();
            selectMention(issue);
          }}
          onmouseenter={() => (mentionIndex = idx)}
          class="w-full text-left px-3 py-2 flex items-center gap-2.5 text-xs hover:bg-gh-subtle cursor-pointer transition-colors {idx === mentionIndex ? 'bg-blue-50/80 border-l-2 border-gh-link' : 'border-l-2 border-transparent'}"
        >
          <div class="shrink-0">
            {#if issue.status === 'open'}
              <CircleDot class="w-3.5 h-3.5 text-gh-green" />
            {:else}
              <CheckCircle2 class="w-3.5 h-3.5 text-gh-purple" />
            {/if}
          </div>
          <span class="font-mono font-semibold text-gh-muted shrink-0">#{issue.id}</span>
          <span class="truncate flex-1 font-medium text-gh-text">{issue.title}</span>
        </div>
      {/each}
    </div>

    {#if highlightedMentionIssue}
      <div class="bg-gh-subtle/70 border-t border-gh-border p-3 flex flex-col gap-2 text-xs">
        <div class="flex items-center gap-2">
          {#if highlightedMentionIssue.status === 'open'}
            <span class="flex items-center gap-1 text-[10px] font-semibold text-gh-green bg-green-50 border border-green-200 px-1.5 py-0.2 rounded-full">
              <CircleDot class="w-3 h-3" />
              Open
            </span>
          {:else}
            <span class="flex items-center gap-1 text-[10px] font-semibold text-gh-purple bg-purple-50 border border-purple-200 px-1.5 py-0.2 rounded-full">
              <CheckCircle2 class="w-3 h-3" />
              Closed
            </span>
          {/if}
          <span class="font-semibold text-gh-text truncate flex-1">{highlightedMentionIssue.title}</span>
          <span class="text-gh-muted text-[11px] shrink-0">{formatRelativeTime(highlightedMentionIssue.created_at)}</span>
        </div>

        <p class="text-gh-muted text-[11px] line-clamp-2 leading-relaxed">
          {getPlainTextExcerpt(highlightedMentionIssue.body, 110) || 'No description provided.'}
        </p>

        {#if highlightedMentionIssue.labels.length > 0 || highlightedMentionIssue.project_title}
          <div class="flex flex-wrap items-center gap-1 pt-1">
            {#each highlightedMentionIssue.labels.slice(0, 3) as lbl (lbl.id)}
              <LabelBadge label={lbl} size="xs" />
            {/each}
            {#if highlightedMentionIssue.project_title}
              <span class="text-gh-muted flex items-center gap-1 text-[10px] ml-auto">
                <FolderKanban class="w-3 h-3" />
                <span class="truncate max-w-[100px]">{highlightedMentionIssue.project_title}</span>
              </span>
            {/if}
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}