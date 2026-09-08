<script lang="ts">
  import { Settings, Monitor } from '@lucide/svelte';
  import Modal from '$lib/components/ui/Modal.svelte';

  let {
    isOpen,
    autostart,
    onClose,
    onToggleAutostart
  }: {
    isOpen: boolean;
    autostart: boolean;
    onClose: () => void;
    onToggleAutostart: (enabled: boolean) => void;
  } = $props();
</script>

<Modal {isOpen} {onClose} title="Settings" icon={Settings}>
  <div class="p-4 text-xs">
    <div class="flex items-center justify-between gap-4 p-3 bg-gh-subtle border border-gh-border rounded-md">
      <div class="flex items-start gap-2.5">
        <Monitor class="w-4 h-4 text-gh-muted shrink-0 mt-0.5" />
        <div>
          <div class="font-semibold text-gh-text">Launch at system startup</div>
          <div class="text-gh-muted mt-0.5">Automatically open app when Windows boots</div>
        </div>
      </div>
      <label class="relative inline-flex items-center cursor-pointer shrink-0">
        <input
          type="checkbox"
          checked={autostart}
          onchange={(e) => onToggleAutostart((e.currentTarget as HTMLInputElement).checked)}
          class="sr-only peer"
        />
        <div class="w-9 h-5 bg-gray-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-gh-link"></div>
      </label>
    </div>
  </div>

  {#snippet footer()}
    <button
      onclick={onClose}
      class="bg-gh-green-btn hover:bg-gh-green text-white text-xs font-semibold px-3 py-1.5 rounded-md cursor-pointer"
    >
      Done
    </button>
  {/snippet}
</Modal>