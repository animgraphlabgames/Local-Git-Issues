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
    Loader2
  } from '@lucide/svelte';
  import Markdown from './Markdown.svelte';
  import { convertToWebp } from '$lib/utils/image';
  import { storeAttachment } from '$lib/utils/attachments';

  let {
    value = $bindable(''),
    placeholder = '',
    rows = 12,
    onsubmit
  }: {
    value: string;
    placeholder?: string;
    rows?: number;
    onsubmit?: () => void;
  } = $props();

  let activeTab = $state<'write' | 'preview'>('write');
  let textareaRef = $state<HTMLTextAreaElement | null>(null);
  let fileInputRef = $state<HTMLInputElement | null>(null);
  let isProcessingImage = $state(false);

  function applyWrap(before: string, after: string, defaultText = 'text') {
    if (!textareaRef) return;
    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const selected = value.substring(start, end);
    const content = selected || defaultText;
    const replacement = `${before}${content}${after}`;

    textareaRef.focus();
    textareaRef.setRangeText(replacement, start, end, 'select');
    if (!selected) {
      textareaRef.setSelectionRange(start + before.length, start + before.length + defaultText.length);
    } else {
      textareaRef.setSelectionRange(start + before.length, end + before.length);
    }
    value = textareaRef.value;
  }

  function applyPrefix(prefix: string) {
    if (!textareaRef) return;
    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const lineStart = value.lastIndexOf('\n', start - 1) + 1;
    let lineEnd = value.indexOf('\n', end);
    if (lineEnd === -1) lineEnd = value.length;

    const lines = value.substring(lineStart, lineEnd).split('\n');
    const transformed = lines.map((l) => `${prefix}${l}`).join('\n');

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
    let replacement: string;
    let selectStart: number;
    let selectEnd: number;

    if (selected.startsWith('http://') || selected.startsWith('https://')) {
      replacement = `[url](${selected})`;
      selectStart = start + 1;
      selectEnd = start + 4;
    } else if (selected) {
      replacement = `[${selected}](url)`;
      selectStart = start + selected.length + 3;
      selectEnd = start + selected.length + 6;
    } else {
      replacement = `[link](url)`;
      selectStart = start + 1;
      selectEnd = start + 5;
    }

    textareaRef.focus();
    textareaRef.setRangeText(replacement, start, end, 'select');
    textareaRef.setSelectionRange(selectStart, selectEnd);
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
    const items = e.clipboardData?.items;
    if (!items) return;

    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      if (item.kind === 'file' && item.type.startsWith('image/')) {
        e.preventDefault();
        e.stopPropagation();
        const file = item.getAsFile();
        if (file) {
          await insertImageFile(file);
        }
        return;
      }
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
    <textarea
      bind:this={textareaRef}
      {rows}
      {placeholder}
      bind:value
      onkeydown={handleKeydown}
      onpaste={handlePaste}
      class="w-full font-mono text-sm p-3 focus:outline-none border-0 block resize-y"
    ></textarea>
  {:else}
    <div class="p-4 min-h-[250px]">
      {#if value.trim()}
        <Markdown content={value} />
      {:else}
        <p class="text-gh-muted text-xs">Nothing to preview</p>
      {/if}
    </div>
  {/if}
</div>