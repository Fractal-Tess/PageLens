<script lang="ts">
  import { onMount } from 'svelte'
  import { push } from 'svelte-spa-router'

  onMount(() => {
    const hash = window.location.hash
    const query = hash.includes('?') ? hash.slice(hash.indexOf('?')) : ''
    push(`/#simple${query}`)
  })
</script>

<div
  class="h-full flex items-center justify-center p-6 text-sm text-muted-foreground"
>
  Redirecting to Simple Analysis...
</div>
