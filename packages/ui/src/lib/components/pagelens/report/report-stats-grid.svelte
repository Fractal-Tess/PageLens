<script lang="ts">
  import { Card, CardContent } from '../../ui/card';
  import type { Snippet } from 'svelte';
  
  interface StatItem {
    label: string;
    value: string | number;
    icon?: typeof import('@lucide/svelte').LayoutDashboard;
    iconClass?: string;
  }
  
  interface Props {
    stats: StatItem[];
    columns?: 2 | 3 | 4 | 6;
    class?: string;
  }
  
  let { stats, columns = 4, class: className = '' }: Props = $props();
  
  const gridCols = $derived({
    2: 'grid-cols-2',
    3: 'grid-cols-3',
    4: 'grid-cols-2 sm:grid-cols-4',
    6: 'grid-cols-3 sm:grid-cols-6',
  }[columns]);
</script>

<div class="grid gap-4 {gridCols} {className}">
  {#each stats as stat}
    <Card>
      <CardContent class="pt-4 text-center">
        {#if stat.icon}
          <svelte:component
            this={stat.icon}
            class="mx-auto mb-1 h-5 w-5 {stat.iconClass ?? 'text-muted-foreground'}"
          />
        {/if}
        <p class="text-2xl font-bold">{stat.value}</p>
        <p class="text-xs text-muted-foreground">{stat.label}</p>
      </CardContent>
    </Card>
  {/each}
</div>
