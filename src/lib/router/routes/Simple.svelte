<script lang="ts">
  import { onMount } from 'svelte'
  import { Button } from '$components/ui/button'
  import { Input } from '$components/ui/input'
  import { Label } from '$components/ui/label'
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$components/ui/card'
  import { Checkbox } from '$components/ui/checkbox'
  import { Separator } from '$components/ui/separator'
  import { Scan, Loader2 } from '@lucide/svelte'
  import { push } from 'svelte-spa-router'
  import { toast } from 'svelte-sonner'
  import { commands } from '$lib/ipc'

  let url = $state('')
  let name = $state('')
  let isRunning = $state(false)

  let includeAccessibilityTree = $state(true)
  let includePerformanceTiming = $state(true)

  onMount(() => {
    const hash = window.location.hash
    const queryIndex = hash.indexOf('?')
    if (queryIndex === -1) return
    const query = hash.slice(queryIndex + 1)
    const params = new URLSearchParams(query)
    const prefilledUrl = params.get('url')
    if (prefilledUrl) {
      url = prefilledUrl
    }
  })

  async function runSimpleAnalysis() {
    if (!url.trim()) {
      toast.error('Please enter a URL')
      return
    }

    isRunning = true
    const runId = crypto.randomUUID()
    push(`/run/${runId}`)

    try {
      const result = await commands.analyzeUrl({
        run_id: runId,
        url: url.trim(),
        name: name.trim() || null,
        options: {
          include_html: true,
          include_accessibility_tree: includeAccessibilityTree,
          include_performance_timing: includePerformanceTiming,
          include_computed_styles: false
        }
      })

      if (result.status === 'error') {
        throw new Error(result.error)
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      toast.error(`Analysis failed: ${message}`)
    } finally {
      isRunning = false
    }
  }
</script>

<div class="h-full overflow-auto p-6 space-y-6">
  <div>
    <h1 class="text-3xl font-bold tracking-tight">Simple Analysis</h1>
    <p class="text-sm text-muted-foreground">
      Quick single-page scan with minimal setup.
    </p>
  </div>

  <Card class="max-w-2xl">
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Scan class="h-5 w-5 text-indigo-500" /> Analyze URL
      </CardTitle>
      <CardDescription>Enter a URL and start immediately.</CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="space-y-2">
        <Label for="simple-url">URL</Label>
        <Input
          id="simple-url"
          type="url"
          bind:value={url}
          disabled={isRunning}
          placeholder="https://example.com"
        />
      </div>

      <div class="space-y-2">
        <Label for="simple-name">Run name (optional)</Label>
        <Input
          id="simple-name"
          bind:value={name}
          disabled={isRunning}
          placeholder="Homepage quick check"
        />
      </div>

      <Separator />

      <div class="space-y-3">
        <Label class="text-sm font-medium">Limited options</Label>
        <div class="flex items-center space-x-2">
          <Checkbox
            bind:checked={includeAccessibilityTree}
            id="simple-a11y"
            disabled={isRunning}
          />
          <Label for="simple-a11y" class="text-sm font-normal"
            >Include accessibility tree</Label
          >
        </div>
        <div class="flex items-center space-x-2">
          <Checkbox
            bind:checked={includePerformanceTiming}
            id="simple-performance"
            disabled={isRunning}
          />
          <Label for="simple-performance" class="text-sm font-normal"
            >Include performance timing</Label
          >
        </div>
      </div>

      <Button
        class="w-full"
        onclick={runSimpleAnalysis}
        disabled={isRunning || !url.trim()}
      >
        {#if isRunning}
          <Loader2 class="mr-2 h-4 w-4 animate-spin" /> Running...
        {:else}
          <Scan class="mr-2 h-4 w-4" /> Start Simple Analysis
        {/if}
      </Button>
    </CardContent>
  </Card>
</div>
