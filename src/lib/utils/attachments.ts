import { attachmentsApi } from '$lib/api';

const blobCache = new Map<string, string>();

export async function storeAttachment(blob: Blob): Promise<string> {
  const name = `attachment-${Date.now()}-${Math.random().toString(36).slice(2, 8)}.webp`;
  const buffer = await blob.arrayBuffer();
  const data = Array.from(new Uint8Array(buffer));
  await attachmentsApi.save(name, data);
  const url = URL.createObjectURL(blob);
  blobCache.set(name, url);
  return name;
}

export async function getAttachmentUrl(name: string): Promise<string> {
  const cached = blobCache.get(name);
  if (cached) return cached;
  const data = await attachmentsApi.get(name);
  const blob = new Blob([new Uint8Array(data)], { type: 'image/webp' });
  const url = URL.createObjectURL(blob);
  blobCache.set(name, url);
  return url;
}

export async function resolveAttachmentsInMarkdown(content: string): Promise<string> {
  if (!content) return '';
  const regex = /attachment:([a-zA-Z0-9_\-\.]+)/g;
  const matches = Array.from(content.matchAll(regex));
  if (matches.length === 0) return content;

  const names = Array.from(new Set(matches.map((m) => m[1])));
  await Promise.all(
    names.map(async (name) => {
      try {
        await getAttachmentUrl(name);
      } catch (err) {
        console.error(err);
      }
    })
  );

  return content.replace(regex, (_, name) => blobCache.get(name) || `attachment:${name}`);
}