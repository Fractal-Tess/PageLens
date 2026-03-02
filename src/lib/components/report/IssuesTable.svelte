<script lang="ts">
  import type { Issue } from '$lib/types'
  import { Badge } from '$components/ui/badge'
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow
  } from '$components/ui/table'

  let { issues }: { issues: Issue[] } = $props()

  function severityVariant(
    severity: string
  ): 'destructive' | 'default' | 'secondary' | 'outline' {
    switch (severity) {
      case 'Error':
        return 'destructive'
      case 'Warning':
        return 'default'
      default:
        return 'secondary'
    }
  }

  function severityClass(severity: string): string {
    switch (severity) {
      case 'Warning':
        return 'bg-amber-500 hover:bg-amber-500/80'
      default:
        return ''
    }
  }
</script>

{#if issues.length === 0}
  <div class="flex flex-col items-center py-8 text-center text-muted-foreground">
    <p class="text-sm">No issues found</p>
  </div>
{:else}
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead class="w-24">Severity</TableHead>
        <TableHead class="w-32">Category</TableHead>
        <TableHead>Message</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      {#each issues as issue}
        <TableRow>
          <TableCell>
            <Badge
              variant={severityVariant(issue.severity)}
              class={severityClass(issue.severity)}
            >
              {issue.severity}
            </Badge>
          </TableCell>
          <TableCell class="font-medium text-xs">{issue.category}</TableCell>
          <TableCell class="text-sm">{issue.message}</TableCell>
        </TableRow>
      {/each}
    </TableBody>
  </Table>
{/if}
