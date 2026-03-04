<script lang="ts">
  import { AlertCircle } from '@lucide/svelte';
  import { Button } from '../../ui/button';
  
  interface Props {
    title?: string;
    message: string;
    onRetry?: () => void;
    class?: string;
  }
  
  let {
    title = 'Error',
    message,
    onRetry,
    class: className = ''
  }: Props = $props();
</script>

<div class="flex flex-1 flex-col items-center justify-center gap-4 p-6 {className}">
  <AlertCircle class="h-12 w-12 text-destructive" />
  <p class="text-lg font-medium">{title}</p>
  <p class="text-sm text-muted-foreground text-center max-w-md">{message}</p>
  {#if onRetry}
    <Button variant="outline" onclick={onRetry}>Try Again</Button>
  {/if}
</div>
