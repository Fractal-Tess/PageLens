<script lang="ts">
  import { Badge } from '../../ui/badge';
  import type { HeadingsInfo } from './types';
  
  interface Props {
    headings: HeadingsInfo;
    class?: string;
  }
  
  let { headings, class: className = '' }: Props = $props();
  
  const counts = $derived([
    { level: 'H1', count: headings.h1_count },
    { level: 'H2', count: headings.h2_count },
    { level: 'H3', count: headings.h3_count },
    { level: 'H4', count: headings.h4_count },
    { level: 'H5', count: headings.h5_count },
    { level: 'H6', count: headings.h6_count },
  ]);
  
  const h1Warning = $derived(
    headings.h1_count === 0 ? 'No H1 tag found' :
    headings.h1_count > 1 ? `Multiple H1 tags found (${headings.h1_count})` :
    null
  );
</script>

<div class="space-y-4 {className}">
  <!-- Count grid -->
  <div class="grid grid-cols-6 gap-2">
    {#each counts as { level, count }}
      <div class="flex flex-col items-center rounded-md bg-muted p-2 text-center">
        <span class="text-xs font-medium text-muted-foreground">{level}</span>
        <span class="text-lg font-bold">{count}</span>
      </div>
    {/each}
  </div>
  
  {#if h1Warning}
    <p class="text-sm {headings.h1_count === 0 ? 'text-red-500' : 'text-amber-500'}">
      {h1Warning}
    </p>
  {/if}
  
  <!-- Structure -->
  {#if headings.structure.length > 0}
    <div class="space-y-1">
      <h4 class="text-sm font-medium">Heading Structure</h4>
      <div class="max-h-48 overflow-auto rounded-md bg-muted p-3">
        {#each headings.structure as heading}
          {@const level = parseInt(heading.match(/^h(\d)/i)?.[1] ?? '1')}
          <div class="text-sm" style="padding-left: {(level - 1) * 16}px">
            <Badge variant="outline" class="mr-1 text-[10px] px-1 py-0">
              H{level}
            </Badge>
            <span class="text-muted-foreground">
              {heading.replace(/^h\d\s*/i, '')}
            </span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
