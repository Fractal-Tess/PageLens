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
  import type { Issue } from './types';
  import { getSeverityVariant, getSeverityClass } from '../../../stores/report.svelte';
  
  interface Props {
    issues: Issue[];
    emptyMessage?: string;
    class?: string;
  }
  
  let { issues, emptyMessage = 'No issues found', class: className = '' }: Props = $props();
</script>

{#if issues.length === 0}
  <div class="flex flex-col items-center py-8 text-center text-muted-foreground">
    <p class="text-sm">{emptyMessage}</p>
  </div>
{:else}
  <Table class={className}>
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
              variant={getSeverityVariant(issue.severity)}
              class={getSeverityClass(issue.severity)}
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
