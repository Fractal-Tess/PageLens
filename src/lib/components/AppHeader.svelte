<script lang="ts">
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
  import { location, link } from 'svelte-spa-router'
  import { commands } from '$lib/ipc'
  import { SidebarTrigger } from '$components/ui/sidebar'
  import { Separator } from '$components/ui/separator'
  import { Button } from '$components/ui/button'
  import { Badge } from '$components/ui/badge'
  import * as Breadcrumb from '$components/ui/breadcrumb'
  import { Github, Minus, X } from '@lucide/svelte'

  const appWindow = getCurrentWebviewWindow()

  type Crumb = {
    label: string
    href?: string
  }

  type RunStatus = 'pending' | 'running' | 'completed' | 'failed'

  function shortId(value: string): string {
    return value.length <= 8 ? value : value.slice(0, 8)
  }

  function breadcrumbsFromPath(rawLocation: string): Crumb[] {
    const locationWithoutQuery = rawLocation.split('?')[0] ?? ''
    const normalized = locationWithoutQuery.startsWith('/#')
      ? `/${locationWithoutQuery.slice(2)}`
      : locationWithoutQuery

    if (!normalized || normalized === '/') {
      return [{ label: 'Dashboard' }]
    }
    if (normalized === '/simple') return [{ label: 'Simple' }]
    if (normalized === '/advanced') return [{ label: 'Advanced' }]
    if (normalized === '/profiles') return [{ label: 'Profiles' }]
    if (normalized === '/history') return [{ label: 'History' }]

    const segments = normalized.split('/').filter(Boolean)
    if (segments[0] === 'run' && segments[1]) {
      if (segments[2] === 'page' && segments[3]) {
        return [
          { label: `Run ${shortId(segments[1])}`, href: `/run/${segments[1]}` },
          { label: `Page ${shortId(segments[3])}` }
        ]
      }
      return [{ label: `Run ${shortId(segments[1])}` }]
    }

    if (segments[0] === 'report' && segments[1]) {
      return [{ label: `Report ${shortId(segments[1])}` }]
    }

    return [{ label: 'PageLens' }]
  }

  function runIdFromPath(rawLocation: string): string | null {
    const locationWithoutQuery = rawLocation.split('?')[0] ?? ''
    const normalized = locationWithoutQuery.startsWith('/#')
      ? `/${locationWithoutQuery.slice(2)}`
      : locationWithoutQuery
    const segments = normalized.split('/').filter(Boolean)
    if (segments[0] === 'run' && segments[1]) return segments[1]
    return null
  }

  let currentRunStatus = $state<RunStatus | null>(null)
  let statusLoadNonce = 0

  $effect(() => {
    const runId = runIdFromPath($location || '/')
    statusLoadNonce += 1
    const nonce = statusLoadNonce

    if (!runId) {
      currentRunStatus = null
      return
    }

    void (async () => {
      const result = await commands.getHistoryItem(runId)
      if (nonce !== statusLoadNonce) return
      if (result.status === 'ok') {
        const status = (result.data as { status?: RunStatus }).status
        currentRunStatus = status ?? null
      }
    })()
  })

  const breadcrumbs = $derived(breadcrumbsFromPath($location || '/'))
</script>

<header
  class="flex h-12 shrink-0 items-center gap-2 border-b px-4"
  data-tauri-drag-region
>
  <SidebarTrigger class="-ml-1" />
  <Separator orientation="vertical" class="mr-2 !h-4" />

  <Breadcrumb.Root>
    <Breadcrumb.List>
      {#each breadcrumbs as crumb, index}
        <Breadcrumb.Item>
          {#if crumb.href && index < breadcrumbs.length - 1}
            <Breadcrumb.Link>
              <a use:link href={crumb.href}>{crumb.label}</a>
            </Breadcrumb.Link>
          {:else}
            <Breadcrumb.Page>{crumb.label}</Breadcrumb.Page>
          {/if}
        </Breadcrumb.Item>
        {#if index < breadcrumbs.length - 1}
          <Breadcrumb.Separator />
        {/if}
      {/each}
    </Breadcrumb.List>
  </Breadcrumb.Root>

  <div class="flex-1" data-tauri-drag-region></div>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="flex items-center gap-1"
    ondragstart={event => event.preventDefault()}
  >
    {#if currentRunStatus}
      <Badge
        variant={currentRunStatus === 'failed'
          ? 'destructive'
          : currentRunStatus === 'completed'
            ? 'default'
            : 'secondary'}
        class="mr-1 uppercase"
      >
        {currentRunStatus}
      </Badge>
    {/if}

    <Button
      variant="ghost"
      size="icon"
      class="h-7 w-7"
      href="https://github.com/Fractal-Tess/pagelens"
      target="_blank"
      rel="noreferrer noopener"
    >
      <Github class="h-4 w-4" />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      class="h-7 w-7"
      onclick={() => appWindow.minimize()}
    >
      <Minus class="h-4 w-4" />
    </Button>
    <Button
      variant="ghost"
      size="icon"
      class="h-7 w-7 hover:bg-destructive hover:text-destructive-foreground"
      onclick={() => appWindow.close()}
    >
      <X class="h-4 w-4" />
    </Button>
  </div>
</header>
