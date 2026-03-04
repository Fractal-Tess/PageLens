<script lang="ts">
  import type { SiteFilesReport } from '$lib/types'
  import { Badge } from '@pagelens/ui/shadcn/badge'
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle
  } from '@pagelens/ui/shadcn/card'
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow
  } from '@pagelens/ui/shadcn/table'
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger
  } from '@pagelens/ui/shadcn/collapsible'
  import { Check, X, ChevronRight, FileText, Map, Bot } from '@lucide/svelte'
  import { cn } from '$lib/utils'

  let { report }: { report: SiteFilesReport } = $props()
</script>

<div class="space-y-6">
  <!-- Robots.txt -->
  <Card>
    <CardHeader class="pb-3">
      <CardTitle class="flex items-center gap-2 text-sm">
        <Bot class="h-4 w-4" />
        robots.txt
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-2">
      <div class="flex items-center gap-2 text-sm">
        {#if report.robots.found}
          <Check class="h-4 w-4 text-green-500" />
          <span>Found</span>
          <Badge variant="outline" class="text-xs">
            {report.robots.status ?? '—'}
          </Badge>
        {:else}
          <X class="h-4 w-4 text-red-500" />
          <span>Not found</span>
        {/if}
      </div>
      {#if report.robots.disallow_all_for_star}
        <p class="text-sm text-amber-500">
          Warning: Disallow all for user-agent *
        </p>
      {/if}
      {#if report.robots.sitemap_directives.length > 0}
        <div class="text-sm">
          <p class="font-medium">Sitemap directives:</p>
          <ul class="list-disc list-inside text-muted-foreground">
            {#each report.robots.sitemap_directives as directive}
              <li class="truncate">{directive}</li>
            {/each}
          </ul>
        </div>
      {/if}
    </CardContent>
  </Card>

  <!-- Sitemaps -->
  <Card>
    <CardHeader class="pb-3">
      <CardTitle class="flex items-center gap-2 text-sm">
        <Map class="h-4 w-4" />
        Sitemaps ({report.sitemaps.length})
      </CardTitle>
    </CardHeader>
    <CardContent>
      {#if report.sitemaps.length === 0}
        <p class="text-sm text-muted-foreground">No sitemaps found</p>
      {:else}
        <div class="space-y-3">
          {#each report.sitemaps as sitemap}
            <Collapsible>
              <CollapsibleTrigger class="flex w-full items-center gap-2 text-sm hover:bg-muted rounded p-2 -mx-2">
                <ChevronRight class="h-4 w-4 transition-transform [[data-state=open]>&]:rotate-90" />
                {#if sitemap.found}
                  <Check class="h-4 w-4 text-green-500" />
                {:else}
                  <X class="h-4 w-4 text-red-500" />
                {/if}
                <span class="truncate font-mono text-xs">{sitemap.url}</span>
                <Badge variant="outline" class="ml-auto text-xs">
                  {sitemap.kind}
                </Badge>
              </CollapsibleTrigger>
              <CollapsibleContent class="pl-8 pt-2 space-y-1">
                {#if sitemap.urls.length > 0}
                  <p class="text-xs text-muted-foreground">
                    {sitemap.urls.length} URLs
                  </p>
                {/if}
                {#if sitemap.child_sitemaps.length > 0}
                  <p class="text-xs text-muted-foreground">
                    {sitemap.child_sitemaps.length} child sitemaps
                  </p>
                {/if}
                {#if sitemap.parse_error}
                  <p class="text-xs text-red-500">
                    Parse error: {sitemap.parse_error}
                  </p>
                {/if}
              </CollapsibleContent>
            </Collapsible>
          {/each}
        </div>
      {/if}
    </CardContent>
  </Card>

  <!-- Misc Files -->
  {#if report.misc_files.length > 0}
    <Card>
      <CardHeader class="pb-3">
        <CardTitle class="flex items-center gap-2 text-sm">
          <FileText class="h-4 w-4" />
          Other Files
        </CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>File</TableHead>
              <TableHead class="w-20">Status</TableHead>
              <TableHead class="w-16">Found</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {#each report.misc_files as file}
              <TableRow>
                <TableCell class="font-mono text-xs">{file.path}</TableCell>
                <TableCell class="text-sm">{file.status ?? '—'}</TableCell>
                <TableCell>
                  {#if file.found}
                    <Check class="h-4 w-4 text-green-500" />
                  {:else}
                    <X class="h-4 w-4 text-muted-foreground" />
                  {/if}
                </TableCell>
              </TableRow>
            {/each}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  {/if}

  <!-- Issues -->
  {#if report.issues.length > 0}
    <Card>
      <CardHeader class="pb-3">
        <CardTitle class="text-sm">
          Issues ({report.issues.length})
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div class="space-y-2">
          {#each report.issues as issue}
            <div class="flex items-start gap-2 text-sm">
              <Badge
                variant={issue.severity === 'Error'
                  ? 'destructive'
                  : 'secondary'}
                class={cn(
                  'text-xs shrink-0',
                  issue.severity === 'Warning' &&
                    'bg-amber-500 hover:bg-amber-500/80 text-white'
                )}
              >
                {issue.severity}
              </Badge>
              <span class="text-muted-foreground">{issue.message}</span>
            </div>
          {/each}
        </div>
      </CardContent>
    </Card>
  {/if}
</div>
