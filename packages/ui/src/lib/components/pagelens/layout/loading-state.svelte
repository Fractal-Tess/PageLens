<script lang="ts">
  import { Loader2 } from '@lucide/svelte';
  
  interface Props {
    message?: string;
    class?: string;
  }
  
  let { message = 'Loading...', class: className = '' }: Props = $props();
</script>

<div class="flex flex-1 items-center justify-center {className}">
  <div class="flex items-center gap-2 text-muted-foreground">
    <Loader2 class="h-5 w-5 animate-spin" />
    <span>{message}</span>
  </div>
</div>
