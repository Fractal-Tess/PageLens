<script lang="ts">
  import type { AccessibilityNode } from './types';
  
  interface Props {
    nodes: AccessibilityNode[];
    depth?: number;
    class?: string;
  }
  
  let { nodes, depth = 0, class: className = '' }: Props = $props();
</script>

<ul class="space-y-1 {className}">
  {#each nodes as node}
    <li>
      <div
        class="rounded border bg-muted/50 px-2 py-1 text-xs"
        style="margin-left: {depth * 14}px"
      >
        <span class="font-semibold">{node.role || 'unknown'}</span>
        {#if node.name}
          <span class="text-muted-foreground"> - {node.name}</span>
        {/if}
        {#if node.level != null}
          <span class="text-muted-foreground"> (lvl {node.level})</span>
        {/if}
      </div>
      {#if node.children.length > 0}
        <svelte:self nodes={node.children} depth={depth + 1} />
      {/if}
    </li>
  {/each}
</ul>
