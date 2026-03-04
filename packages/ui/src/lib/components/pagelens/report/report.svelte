<script lang="ts">
  import { setContext, type Snippet } from 'svelte';
  import type { SeoReport, PerformanceTiming } from './types';
  
  interface Props {
    children: Snippet;
    class?: string;
  }
  
  let { children, class: className = '' }: Props = $props();
  
  // Context key for report data
  const REPORT_CONTEXT = Symbol('report');
  
  export function setReportData(data: {
    seoReport?: SeoReport;
    performanceTiming?: PerformanceTiming | null;
  }) {
    setContext(REPORT_CONTEXT, data);
  }
</script>

<div class="space-y-6 {className}">
  {@render children()}
</div>
