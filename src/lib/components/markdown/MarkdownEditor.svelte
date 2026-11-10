<script lang="ts">
  import { onMount } from 'svelte';
  import { marked } from 'marked';
  import {
    Bold,
    Italic,
    Strikethrough,
    Code,
    FileCode,
    Link,
    List,
    ListOrdered,
    Quote,
    Heading1,
    Heading2,
    Heading3,
    Table,
    Minus,
    Image as ImageIcon,
    Loader2,
    CircleDot,
    CheckCircle2,
    FolderKanban
  } from '@lucide/svelte';
  import LabelBadge from '$lib/components/ui/LabelBadge.svelte';
  import { convertToWebp } from '$lib/utils/image';
  import { storeAttachment, getAttachmentUrl } from '$lib/utils/attachments';
  import { formatRelativeTime, getPlainTextExcerpt } from '$lib/utils/format';
  import type { Issue } from '$lib/types';

  marked.use({
    gfm: true,
    breaks: true,
    extensions: [
      {
        name: 'issueRef',
        level: 'inline',
        start(src: string) {
          const match = /(?:^|[\s(\[])(#\d+\b|\[#\d+\](?!\())/.exec(src);
          return match ? match.index + (match[0].length - match[1].length) : -1;
        },
        tokenizer(src: string) {
          const match = /^(?:\[#(\d+)\](?!\()|#(\d+))\b/.exec(src);
          if (match) {
            return {
              type: 'issueRef',
              raw: match[0],
              issueId: match[1] || match[2]
            };
          }
        },
        renderer(token: any) {
          return `<a href="#issue-${token.issueId}" class="issue-mention font-semibold text-gh-link hover:underline" data-issue-id="${token.issueId}">#${token.issueId}</a>`;
        }
      }
    ]
  });

  let {
    value = $bindable(''),
    placeholder = 'Write your description...',
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

  let editorRef = $state<HTMLDivElement | null>(null);
  let fileInputRef = $state<HTMLInputElement | null>(null);
  let isProcessingImage = $state(false);
  let lastSyncedMd = $state('');

  let mentionSearch = $state<string | null>(null);
  let mentionIndex = $state(0);
  let mentionPosition = $state<{ top: number; left: number; placeAbove: boolean }>({ top: 0, left: 0, placeAbove: false });

  let matchingIssues = $derived(
    mentionSearch === null
      ? []
      : issues.filter((i) => i.id.toString().includes(mentionSearch!) || i.title.toLowerCase().includes(mentionSearch!)).slice(0, 5)
  );

  let highlightedMentionIssue = $derived(matchingIssues[mentionIndex] ?? null);

  function nodeToMarkdown(node: Node): string {
    if (node.nodeType === Node.TEXT_NODE) {
      return node.textContent || '';
    }
    if (node.nodeType !== Node.ELEMENT_NODE) {
      return '';
    }
    const el = node as HTMLElement;
    const tag = el.tagName.toLowerCase();

    const childrenMd = () => Array.from(el.childNodes).map(nodeToMarkdown).join('');

    switch (tag) {
      case 'h1': return `\n# ${childrenMd().trim()}\n\n`;
      case 'h2': return `\n## ${childrenMd().trim()}\n\n`;
      case 'h3': return `\n### ${childrenMd().trim()}\n\n`;
      case 'h4': return `\n#### ${childrenMd().trim()}\n\n`;
      case 'h5': return `\n##### ${childrenMd().trim()}\n\n`;
      case 'h6': return `\n###### ${childrenMd().trim()}\n\n`;
      case 'p': {
        const content = childrenMd().trim();
        return content ? `\n${content}\n\n` : '\n';
      }
      case 'strong':
      case 'b': {
        const content = childrenMd();
        return content ? `**${content}**` : '';
      }
      case 'em':
      case 'i': {
        const content = childrenMd();
        return content ? `*${content}*` : '';
      }
      case 'del':
      case 's':
      case 'strike': {
        const content = childrenMd();
        return content ? `~~${content}~~` : '';
      }
      case 'code': {
        if (el.parentElement?.tagName.toLowerCase() === 'pre') {
          return el.textContent || '';
        }
        return `\`${el.textContent || ''}\``;
      }
      case 'pre': {
        const codeEl = el.querySelector('code');
        const code = (codeEl ? codeEl.textContent : el.textContent) || '';
        return `\n\`\`\`\n${code.replace(/\n+$/, '')}\n\`\`\`\n\n`;
      }
      case 'blockquote': {
        const text = childrenMd().trim();
        const lines = text.split('\n').map((l) => `> ${l}`).join('\n');
        return `\n${lines}\n\n`;
      }
      case 'ul': {
        const items = Array.from(el.children).filter((c) => c.tagName.toLowerCase() === 'li');
        const listMd = items.map((li) => `- ${nodeToMarkdown(li).trim()}`).join('\n');
        return `\n${listMd}\n\n`;
      }
      case 'ol': {
        const items = Array.from(el.children).filter((c) => c.tagName.toLowerCase() === 'li');
        const listMd = items.map((li, i) => `${i + 1}. ${nodeToMarkdown(li).trim()}`).join('\n');
        return `\n${listMd}\n\n`;
      }
      case 'li': {
        return childrenMd();
      }
      case 'hr': {
        return '\n---\n\n';
      }
      case 'br': {
        return '\n';
      }
      case 'a': {
        const issueId = el.getAttribute('data-issue-id');
        if (issueId) {
          return `#${issueId}`;
        }
        const href = el.getAttribute('href') || '';
        if (href.startsWith('#issue-')) {
          return `#${href.replace('#issue-', '')}`;
        }
        const text = childrenMd();
        return `[${text || href}](${href})`;
      }
      case 'img': {
        const alt = el.getAttribute('alt') || 'image';
        const attachName = el.getAttribute('data-attachment');
        if (attachName) {
          return `![${alt}](attachment:${attachName})`;
        }
        const src = el.getAttribute('src') || '';
        const match = src.match(/attachment:([^"'\s]+)/);
        if (match) {
          return `![${alt}](attachment:${match[1]})`;
        }
        return `![${alt}](${src})`;
      }
      case 'table': {
        const rows = Array.from(el.querySelectorAll('tr'));
        if (rows.length === 0) return '';
        const firstRowCells = Array.from(rows[0].querySelectorAll('th, td'));
        if (firstRowCells.length === 0) return '';
        let tableMd = '\n';
        tableMd += '| ' + firstRowCells.map((c) => nodeToMarkdown(c).trim().replace(/\n/g, ' ') || ' ').join(' | ') + ' |\n';
        tableMd += '| ' + firstRowCells.map(() => '---').join(' | ') + ' |\n';
        for (let r = 1; r < rows.length; r++) {
          const cells = Array.from(rows[r].querySelectorAll('td, th'));
          tableMd += '| ' + cells.map((c) => nodeToMarkdown(c).trim().replace(/\n/g, ' ') || ' ').join(' | ') + ' |\n';
        }
        return tableMd + '\n';
      }
      case 'div': {
        const content = childrenMd().trim();
        return content ? `\n${content}\n` : '\n';
      }
      default:
        return childrenMd();
    }
  }

  function htmlToMarkdown(htmlContainer: HTMLElement): string {
    const md = Array.from(htmlContainer.childNodes).map(nodeToMarkdown).join('');
    return md.replace(/\n{3,}/g, '\n\n').trim();
  }

  async function markdownToEditorHtml(md: string): Promise<string> {
    if (!md.trim()) return '';
    let processed = md;
    const attachRegex = /!\[(.*?)\]\(attachment:([a-zA-Z0-9_\-\.]+)\)/g;
    const matches = Array.from(md.matchAll(attachRegex));
    for (const match of matches) {
      const alt = match[1];
      const name = match[2];
      try {
        const url = await getAttachmentUrl(name);
        processed = processed.replace(
          match[0],
          `<img src="${url}" alt="${alt}" data-attachment="${name}" />`
        );
      } catch (e) {
        console.error(e);
      }
    }
    return marked.parse(processed) as string;
  }

  function syncValue() {
    if (!editorRef) return;
    const md = htmlToMarkdown(editorRef);
    lastSyncedMd = md;
    value = md;
  }

  $effect(() => {
    if (value !== lastSyncedMd && editorRef) {
      lastSyncedMd = value;
      markdownToEditorHtml(value).then((html) => {
        if (editorRef) {
          editorRef.innerHTML = html;
        }
      });
    }
  });

  onMount(() => {
    if (editorRef && value) {
      lastSyncedMd = value;
      markdownToEditorHtml(value).then((html) => {
        if (editorRef) {
          editorRef.innerHTML = html;
        }
      });
    }
  });

  function insertHtmlAtCaret(html: string) {
    editorRef?.focus();
    const sel = window.getSelection();
    if (sel && sel.rangeCount > 0) {
      const range = sel.getRangeAt(0);
      range.deleteContents();
      const el = document.createElement('div');
      el.innerHTML = html;
      const frag = document.createDocumentFragment();
      let node: Node | null;
      let lastNode: Node | null = null;
      while ((node = el.firstChild)) {
        lastNode = frag.appendChild(node);
      }
      range.insertNode(frag);
      if (lastNode) {
        range.setStartAfter(lastNode);
        range.collapse(true);
        sel.removeAllRanges();
        sel.addRange(range);
      }
    }
  }

  function applyFormatBlock(tag: string) {
    editorRef?.focus();
    try {
      document.execCommand('formatBlock', false, `<${tag}>`);
    } catch {
      document.execCommand('formatBlock', false, tag);
    }
    syncValue();
  }

  function toggleHeading(level: 1 | 2 | 3 | 4 | 5) {
    editorRef?.focus();
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) return;
    let parent = sel.getRangeAt(0).commonAncestorContainer as HTMLElement;
    if (parent.nodeType === Node.TEXT_NODE) parent = parent.parentElement as HTMLElement;
    const target = `h${level}`;
    const isAlready = parent.closest(target);
    applyFormatBlock(isAlready ? 'p' : target);
  }

  function toggleInlineCode() {
    editorRef?.focus();
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) return;
    const range = sel.getRangeAt(0);
    let parent = range.commonAncestorContainer as HTMLElement;
    if (parent.nodeType === Node.TEXT_NODE) parent = parent.parentElement as HTMLElement;
    const codeEl = parent.closest('code');
    if (codeEl && codeEl.parentElement?.tagName.toLowerCase() !== 'pre') {
      const text = codeEl.textContent || '';
      codeEl.replaceWith(document.createTextNode(text));
      syncValue();
      return;
    }
    if (range.collapsed) return;
    const code = document.createElement('code');
    code.textContent = range.toString();
    range.deleteContents();
    range.insertNode(code);
    range.setStartAfter(code);
    range.collapse(true);
    sel.removeAllRanges();
    sel.addRange(range);
    syncValue();
  }

  function insertCodeBlock() {
    editorRef?.focus();
    const sel = window.getSelection();
    const text = sel?.toString() || 'code';
    insertHtmlAtCaret(`<pre><code>${text}</code></pre><p><br></p>`);
    syncValue();
  }

  function insertTable() {
    editorRef?.focus();
    insertHtmlAtCaret(`<table><thead><tr><th>Header 1</th><th>Header 2</th><th>Header 3</th></tr></thead><tbody><tr><td>Cell 1</td><td>Cell 2</td><td>Cell 3</td></tr><tr><td>Cell 4</td><td>Cell 5</td><td>Cell 6</td></tr></tbody></table><p><br></p>`);
    syncValue();
  }

  function insertLink() {
    editorRef?.focus();
    const sel = window.getSelection();
    const selectedText = sel?.toString() || '';
    const url = prompt('Enter URL:', selectedText.startsWith('http') ? selectedText : 'https://');
    if (url) {
      if (selectedText && !selectedText.startsWith('http')) {
        document.execCommand('createLink', false, url);
      } else {
        insertHtmlAtCaret(`<a href="${url}">${url}</a>`);
      }
      syncValue();
    }
  }

  export async function insertImageFile(file: File | Blob) {
    isProcessingImage = true;
    try {
      const webpBlob = await convertToWebp(file, 0.8);
      const name = await storeAttachment(webpBlob);
      const url = await getAttachmentUrl(name);
      editorRef?.focus();
      insertHtmlAtCaret(`<img src="${url}" alt="image" data-attachment="${name}" /><p><br></p>`);
      syncValue();
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

  function updateMentionState() {
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) {
      mentionSearch = null;
      return;
    }
    const range = sel.getRangeAt(0);
    const container = range.startContainer;
    if (container.nodeType === Node.TEXT_NODE) {
      const text = container.textContent || '';
      const textBefore = text.substring(0, range.startOffset);
      const match = /(?:^|\s)#([a-zA-Z0-9_-]*)$/.exec(textBefore);
      if (match) {
        mentionSearch = match[1].toLowerCase();
        mentionIndex = 0;
        const rect = range.getBoundingClientRect();
        const placeAbove = rect.top + 340 > window.innerHeight;
        mentionPosition = {
          top: placeAbove ? rect.top - 8 : rect.bottom + 8,
          left: Math.max(12, Math.min(window.innerWidth - 400, rect.left)),
          placeAbove
        };
        return;
      }
    }
    mentionSearch = null;
  }

  function selectMention(item: Issue) {
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) return;
    const range = sel.getRangeAt(0);
    const container = range.startContainer;
    if (container.nodeType === Node.TEXT_NODE) {
      const text = container.textContent || '';
      const textBefore = text.substring(0, range.startOffset);
      const hashIdx = textBefore.lastIndexOf('#');
      if (hashIdx !== -1) {
        range.setStart(container, hashIdx);
        range.setEnd(container, range.startOffset);
        range.deleteContents();

        const link = document.createElement('a');
        link.href = `#issue-${item.id}`;
        link.className = 'issue-mention font-semibold text-gh-link hover:underline';
        link.setAttribute('data-issue-id', item.id.toString());
        link.textContent = `#${item.id}`;

        const space = document.createTextNode(' ');
        range.insertNode(space);
        range.insertNode(link);

        range.setStartAfter(space);
        range.collapse(true);
        sel.removeAllRanges();
        sel.addRange(range);
      }
    }
    mentionSearch = null;
    syncValue();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (mentionSearch !== null && matchingIssues.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        e.stopPropagation();
        mentionIndex = (mentionIndex + 1) % matchingIssues.length;
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        e.stopPropagation();
        mentionIndex = (mentionIndex - 1 + matchingIssues.length) % matchingIssues.length;
        return;
      }
      if (e.key === 'Enter' || e.key === 'Tab') {
        e.preventDefault();
        e.stopPropagation();
        selectMention(matchingIssues[mentionIndex]);
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        e.stopPropagation();
        mentionSearch = null;
        return;
      }
    }

    const isMod = e.ctrlKey || e.metaKey;

    if (isMod && e.key === 'Enter') {
      e.preventDefault();
      e.stopPropagation();
      onsubmit?.();
      return;
    }

    if (isMod && (e.key === 'b' || e.key === 'B')) {
      e.preventDefault();
      e.stopPropagation();
      document.execCommand('bold', false);
      syncValue();
      return;
    }

    if (isMod && (e.key === 'i' || e.key === 'I')) {
      e.preventDefault();
      e.stopPropagation();
      document.execCommand('italic', false);
      syncValue();
      return;
    }

    if (isMod && (e.key === 'k' || e.key === 'K')) {
      e.preventDefault();
      e.stopPropagation();
      insertLink();
      return;
    }

    if (isMod && (e.key === 'e' || e.key === 'E')) {
      e.preventDefault();
      e.stopPropagation();
      toggleInlineCode();
      return;
    }

    if (isMod && e.shiftKey) {
      let headingLevel: (1 | 2 | 3 | 4 | 5) | null = null;
      if (e.code === 'Digit1' || e.code === 'Numpad1' || e.key === '1' || e.key === '!') headingLevel = 1;
      else if (e.code === 'Digit2' || e.code === 'Numpad2' || e.key === '2' || e.key === '@') headingLevel = 2;
      else if (e.code === 'Digit3' || e.code === 'Numpad3' || e.key === '3' || e.key === '#') headingLevel = 3;
      else if (e.code === 'Digit4' || e.code === 'Numpad4' || e.key === '4' || e.key === '$') headingLevel = 4;
      else if (e.code === 'Digit5' || e.code === 'Numpad5' || e.key === '5' || e.key === '%') headingLevel = 5;

      if (headingLevel !== null) {
        e.preventDefault();
        e.stopPropagation();
        toggleHeading(headingLevel);
        return;
      }
    }

    if (isMod && e.shiftKey && (e.key === 'x' || e.key === 'X')) {
      e.preventDefault();
      e.stopPropagation();
      document.execCommand('strikeThrough', false);
      syncValue();
      return;
    }

    if (isMod && e.shiftKey && (e.key === 'c' || e.key === 'C')) {
      e.preventDefault();
      e.stopPropagation();
      insertCodeBlock();
      return;
    }

    if (isMod && e.shiftKey && e.key === '8') {
      e.preventDefault();
      e.stopPropagation();
      document.execCommand('insertUnorderedList', false);
      syncValue();
      return;
    }

    if (isMod && e.shiftKey && e.key === '7') {
      e.preventDefault();
      e.stopPropagation();
      document.execCommand('insertOrderedList', false);
      syncValue();
      return;
    }

    if (isMod && e.shiftKey && (e.key === '.' || e.key === '>')) {
      e.preventDefault();
      e.stopPropagation();
      applyFormatBlock('blockquote');
      syncValue();
      return;
    }

    if (e.key === 'Tab') {
      e.preventDefault();
      e.stopPropagation();
      if (e.shiftKey) {
        document.execCommand('outdent', false);
      } else {
        document.execCommand('indent', false);
      }
      syncValue();
      return;
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

  <div class="bg-gh-subtle border-b border-gh-border px-3 py-2 flex items-center justify-between flex-wrap gap-2 text-gh-muted select-none">
    <div class="flex items-center gap-1 flex-wrap">
      <button
        type="button"
        onclick={() => toggleHeading(1)}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Heading 1 (Ctrl+Shift+1)"
      >
        <Heading1 class="w-4 h-4" />
      </button>
      <button
        type="button"
        onclick={() => toggleHeading(2)}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Heading 2 (Ctrl+Shift+2)"
      >
        <Heading2 class="w-4 h-4" />
      </button>
      <button
        type="button"
        onclick={() => toggleHeading(3)}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Heading 3 (Ctrl+Shift+3)"
      >
        <Heading3 class="w-4 h-4" />
      </button>

      <span class="w-px h-4 bg-gh-border mx-1"></span>

      <button
        type="button"
        onclick={() => { document.execCommand('bold', false); syncValue(); }}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Bold (Ctrl+B)"
      >
        <Bold class="w-3.5 h-3.5" />
      </button>
      <button
        type="button"
        onclick={() => { document.execCommand('italic', false); syncValue(); }}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Italic (Ctrl+I)"
      >
        <Italic class="w-3.5 h-3.5" />
      </button>
      <button
        type="button"
        onclick={() => { document.execCommand('strikeThrough', false); syncValue(); }}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Strikethrough (Ctrl+Shift+X)"
      >
        <Strikethrough class="w-3.5 h-3.5" />
      </button>

      <span class="w-px h-4 bg-gh-border mx-1"></span>

      <button
        type="button"
        onclick={toggleInlineCode}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Inline code (Ctrl+E)"
      >
        <Code class="w-3.5 h-3.5" />
      </button>
      <button
        type="button"
        onclick={insertCodeBlock}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Code block (Ctrl+Shift+C)"
      >
        <FileCode class="w-3.5 h-3.5" />
      </button>
      <button
        type="button"
        onclick={() => { applyFormatBlock('blockquote'); syncValue(); }}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Quote (Ctrl+Shift+>)"
      >
        <Quote class="w-3.5 h-3.5" />
      </button>

      <span class="w-px h-4 bg-gh-border mx-1"></span>

      <button
        type="button"
        onclick={() => { document.execCommand('insertUnorderedList', false); syncValue(); }}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Bullet list (Ctrl+Shift+8)"
      >
        <List class="w-3.5 h-3.5" />
      </button>
      <button
        type="button"
        onclick={() => { document.execCommand('insertOrderedList', false); syncValue(); }}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Numbered list (Ctrl+Shift+7)"
      >
        <ListOrdered class="w-3.5 h-3.5" />
      </button>
      <button
        type="button"
        onclick={insertTable}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Insert table"
      >
        <Table class="w-3.5 h-3.5" />
      </button>

      <span class="w-px h-4 bg-gh-border mx-1"></span>

      <button
        type="button"
        onclick={insertLink}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Insert link (Ctrl+K)"
      >
        <Link class="w-3.5 h-3.5" />
      </button>
      <button
        type="button"
        onclick={() => fileInputRef?.click()}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Attach image"
      >
        <ImageIcon class="w-3.5 h-3.5" />
      </button>
      <button
        type="button"
        onclick={() => { document.execCommand('insertHorizontalRule', false); syncValue(); }}
        class="p-1 rounded hover:bg-gray-200 hover:text-gh-text cursor-pointer"
        title="Divider"
      >
        <Minus class="w-3.5 h-3.5" />
      </button>
    </div>

    {#if isProcessingImage}
      <span class="flex items-center gap-1 text-[11px] text-gh-muted pr-2">
        <Loader2 class="w-3 h-3 animate-spin" />
        <span>Processing WebP...</span>
      </span>
    {/if}
  </div>

  <div
    bind:this={editorRef}
    contenteditable="true"
    role="textbox"
    tabindex="0"
    aria-multiline="true"
    data-placeholder={placeholder}
    oninput={() => { syncValue(); updateMentionState(); }}
    onclick={updateMentionState}
    onkeyup={(e) => {
      if (!['ArrowUp', 'ArrowDown', 'Enter', 'Tab', 'Escape'].includes(e.key)) {
        updateMentionState();
      }
    }}
    onkeydown={handleKeydown}
    onpaste={handlePaste}
    class="gh-markdown p-5 focus:outline-none bg-white text-gh-text overflow-y-auto selection:bg-blue-100"
    style="min-height: {Math.max(180, (rows || 12) * 22)}px;"
  ></div>
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

<style>
  [contenteditable][data-placeholder]:empty::before {
    content: attr(data-placeholder);
    color: var(--color-gh-muted);
    pointer-events: none;
  }
</style>
