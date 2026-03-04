<script lang="ts">
  import { Badge } from '../../ui/badge';
  import { Button } from '../../ui/button';
  import { ArrowLeft, ExternalLink, Clock, FileText, Globe } from '@lucide/svelte';
  import type { Snippet } from 'svelte';
  
  interface Props {
    title: string;
    url: string;
    createdAt: string;
    analysisType: 'single' | 'crawl';
    onBack?: () => void;
    children?: Snippet;
    class?: string;
  }
  
  let {
    title,
    url,
    createdAt,
    analysisType,
    onBack,
    children,
    class: className = ''
  }: Props = $props();
  
  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString();
  }
</script>

<div class="flex items-start justify-between gap-4 {className}">
  <div class="flex items-center gap-4">
    {#if onBack}
      <Button variant="ghost" size="icon" onclick={onBack}>
        <ArrowLeft class="h-4 w-4" />
      </Button>
    {/if}
    <div>
      <h1 class="text-2xl font-bold tracking-tight">{title}</h1>
      <div class="flex items-center gap-3 mt-1 text-sm text-muted-foreground">
        <a
          href={url}
          target="_blank"
          rel="noreferrer noopener"
          class="flex items-center gap-1 hover:text-foreground truncate max-w-md"
        >
          {url}
          <ExternalLink class="h-3 w-3 shrink-0" />
        </a>
        <span class="flex items-center gap-1">
          <Clock class="h-3 w-3" />
          {formatDate(createdAt)}
        </span>
      </div>
    </div>
  </div>
  
  <div class="flex items-center gap-2">
    {@render children?.()}
    
    <Badge variant={analysisType === 'single' ? 'default' : 'secondary'}>
      {#if analysisType === 'single'}
        <FileText class="mr-1 h-3 w-3" />
        Single Page
      {:else}
        <Globe class="mr-1 h-3 w-3" />
        Crawl
      {/if}
    </Badge>
  </div>
</div>
