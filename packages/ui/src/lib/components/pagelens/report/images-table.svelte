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
  import type { ImageInfo } from './types';
  
  interface Props {
    images: ImageInfo[];
    emptyMessage?: string;
    class?: string;
  }
  
  let { images, emptyMessage = 'No images found', class: className = '' }: Props = $props();
  
  const missingAlt = $derived(images.filter(i => !i.has_alt).length);
</script>

{#if images.length === 0}
  <div class="flex flex-col items-center py-8 text-center text-muted-foreground">
    <p class="text-sm">{emptyMessage}</p>
  </div>
{:else}
  <div class="space-y-3 {className}">
    <div class="flex items-center gap-4 text-sm">
      <span>{images.length} images total</span>
      {#if missingAlt > 0}
        <Badge variant="destructive">{missingAlt} missing alt text</Badge>
      {:else}
        <Badge variant="default" class="bg-green-500">All have alt text</Badge>
      {/if}
    </div>
    
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Source</TableHead>
          <TableHead>Alt Text</TableHead>
          <TableHead class="w-20">Has Alt</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {#each images as image}
          <TableRow>
            <TableCell class="max-w-xs truncate text-xs font-mono">
              {image.src}
            </TableCell>
            <TableCell class="max-w-xs text-sm text-muted-foreground">
              {image.alt ?? '—'}
            </TableCell>
            <TableCell>
              {#if image.has_alt}
                <Badge variant="default" class="bg-green-500 text-xs">Yes</Badge>
              {:else}
                <Badge variant="destructive" class="text-xs">No</Badge>
              {/if}
            </TableCell>
          </TableRow>
        {/each}
      </TableBody>
    </Table>
  </div>
{/if}
