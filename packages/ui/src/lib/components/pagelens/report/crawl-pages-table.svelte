<script lang="ts">
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '../../ui/table';
  import { Badge } from '../../ui/badge';
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from '../../ui/collapsible';
  import { ChevronRight } from '@lucide/svelte';
  import { cn } from '../../../utils';
  import type { CrawledPage } from './types';
  import { getScoreColor, getSeverityVariant, getSeverityClass } from '../../../stores/report.svelte';
  
  interface Props {
    pages: CrawledPage[];
    class?: string;
  }
  
  let { pages, class: className = '' }: Props = $props();
  
  let expandedUrl = $state<string | null>(null);
  
  function toggleExpand(url: string) {
    expandedUrl = expandedUrl === url ? null : url;
  }
  
  function getScoreBgClass(score: number): string {
    if (score >= 80) return 'bg-green-500';
    if (score >= 50) return 'bg-amber-500';
    return 'bg-red-500';
  }
</script>

<Table class={className}>
  <TableHeader>
    <TableRow>
      <TableHead class="w-8"></TableHead>
      <TableHead>URL</TableHead>
      <TableHead class="w-20">Score</TableHead>
      <TableHead class="w-20">Issues</TableHead>
      <TableHead class="w-16">Depth</TableHead>
      <TableHead class="w-20">Status</TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    {#each pages as page}
      <TableRow class="cursor-pointer" onclick={() => toggleExpand(page.url)}>
        <TableCell>
          <ChevronRight
            class={cn('h-4 w-4 transition-transform', expandedUrl === page.url && 'rotate-90')}
          />
        </TableCell>
        <TableCell class="max-w-sm truncate text-xs font-mono">{page.url}</TableCell>
        <TableCell>
          <Badge class={cn('text-white text-xs', getScoreBgClass(page.seo_report.score))}>
            {Math.round(page.seo_report.score)}
          </Badge>
        </TableCell>
        <TableCell class="text-sm">{page.seo_report.issues.length}</TableCell>
        <TableCell class="text-sm text-muted-foreground">{page.depth}</TableCell>
        <TableCell>
          {#if page.success}
            <Badge variant="default" class="bg-green-500 text-xs">OK</Badge>
          {:else}
            <Badge variant="destructive" class="text-xs">Failed</Badge>
          {/if}
        </TableCell>
      </TableRow>
      
      {#if expandedUrl === page.url}
        <TableRow>
          <TableCell colspan={6} class="bg-muted/50 p-4">
            <div class="space-y-2">
              <div class="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span class="font-medium">Title: </span>
                  <span class="text-muted-foreground">
                    {page.seo_report.meta.title ?? 'None'}
                  </span>
                </div>
                <div>
                  <span class="font-medium">Links found: </span>
                  <span class="text-muted-foreground">{page.links_found.length}</span>
                </div>
              </div>
              {#if page.error}
                <p class="text-sm text-red-500">Error: {page.error}</p>
              {/if}
              {#if page.seo_report.issues.length > 0}
                <div class="space-y-1">
                  <p class="text-xs font-medium">Issues:</p>
                  {#each page.seo_report.issues.slice(0, 5) as issue}
                    <div class="flex items-center gap-2 text-xs">
                      <Badge
                        variant={getSeverityVariant(issue.severity)}
                        class={cn('text-[10px] px-1 py-0', getSeverityClass(issue.severity))}
                      >
                        {issue.severity}
                      </Badge>
                      <span class="text-muted-foreground">{issue.message}</span>
                    </div>
                  {/each}
                  {#if page.seo_report.issues.length > 5}
                    <p class="text-xs text-muted-foreground">
                      ...and {page.seo_report.issues.length - 5} more
                    </p>
                  {/if}
                </div>
              {/if}
            </div>
          </TableCell>
        </TableRow>
      {/if}
    {/each}
  </TableBody>
</Table>
