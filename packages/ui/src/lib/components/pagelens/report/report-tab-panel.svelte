<script lang="ts">
  import { Card, CardContent } from '../../ui/card';
  import type { Snippet } from 'svelte';
  
  interface Props {
    value: string;
    activeTab: string;
    children: Snippet;
    class?: string;
  }
  
  let { value, activeTab, children, class: className = '' }: Props = $props();
  
  const isActive = $derived(value === activeTab);
</script>

{#if isActive}
  <div class="mt-4 {className}" role="tabpanel" aria-labelledby="tab-{value}">
    <Card>
      <CardContent class="pt-4">
        {@render children()}
      </CardContent>
    </Card>
  </div>
{/if}
