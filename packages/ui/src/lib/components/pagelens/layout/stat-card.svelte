<script lang="ts">
  import { Card, CardContent } from '../../ui/card';
  import type { Component } from 'svelte';
  
  interface Props {
    value: string | number;
    label: string;
    icon?: typeof import('@lucide/svelte').LayoutDashboard;
    iconClass?: string;
    valueClass?: string;
    class?: string;
  }
  
  let {
    value,
    label,
    icon: Icon,
    iconClass = 'text-indigo-500',
    valueClass = 'text-2xl font-bold',
    class: className = ''
  }: Props = $props();
</script>

<Card class={className}>
  <CardContent class="flex items-center gap-4 pt-6">
    {#if Icon}
      <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-indigo-500/10">
        <Icon class="h-5 w-5 {iconClass}" />
      </div>
    {/if}
    <div>
      <p class={valueClass}>{value}</p>
      <p class="text-xs text-muted-foreground">{label}</p>
    </div>
  </CardContent>
</Card>
