<script lang="ts">
  import { onMount } from 'svelte';
  import { push } from 'svelte-spa-router';

  onMount(() => {
    push('/#simple');
  });
</script>

<div class="flex h-full items-center justify-center">
  <p class="text-muted-foreground">Redirecting...</p>
</div>
