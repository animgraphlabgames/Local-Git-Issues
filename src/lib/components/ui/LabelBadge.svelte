<script lang="ts">
  import type { Label } from '$lib/types';

  let {
    label,
    size = 'sm'
  }: {
    label: Label;
    size?: 'xs' | 'sm';
  } = $props();
</script>

<span
  class="font-semibold rounded-full border border-black/10 inline-flex items-center leading-none {size === 'xs' ? 'text-[11px] px-2 py-0.5' : 'text-xs px-2.5 py-1'}"
  style="background-color: {label.color}; color: #000;"
>
  {label.name}
</span>