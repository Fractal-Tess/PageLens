<script lang="ts">
  import { Input } from '../../ui/input';
  import { Label } from '../../ui/label';
  import { Button } from '../../ui/button';
  import { Scan } from '@lucide/svelte';
  import type { Snippet } from 'svelte';
  
  interface Props {
    value: string;
    disabled?: boolean;
    placeholder?: string;
    onSubmit?: () => void;
    children?: Snippet;
    class?: string;
  }
  
  let {
    value = $bindable(),
    disabled = false,
    placeholder = 'https://example.com',
    onSubmit,
    children,
    class: className = ''
  }: Props = $props();
  
  function handleSubmit(e: Event) {
    e.preventDefault();
    onSubmit?.();
  }
</script>

<form class="flex gap-3 {className}" onsubmit={handleSubmit}>
  <Input
    type="url"
    {placeholder}
    bind:value
    {disabled}
    class="flex-1"
  />
  {@render children?.()}
</form>
