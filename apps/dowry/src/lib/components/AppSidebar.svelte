<script lang="ts">
  import { location, link } from 'svelte-spa-router'
  import { toggleMode, mode } from 'mode-watcher'
  import { cn } from '$lib/utils'
  import {
    SidebarHeader,
    SidebarContent,
    SidebarFooter,
    SidebarGroup,
    SidebarGroupLabel,
    SidebarGroupContent,
    SidebarMenu,
    SidebarMenuItem,
    SidebarMenuButton,
    SidebarRail,
    Sidebar
  } from '@pagelens/ui/shadcn/sidebar'
  import { Button } from '@pagelens/ui/shadcn/button'
  import {
    LayoutDashboard,
    Scan,
    SlidersHorizontal,
    Settings2,
    History,
    Activity,
    FileText,
    Sun,
    Moon,
    Search
  } from '@lucide/svelte'

  type NavItem = {
    label: string
    href: string
    icon: typeof LayoutDashboard
  }

  type NavGroup = {
    label: string
    items: NavItem[]
  }

  const primaryNavGroups: NavGroup[] = [
    {
      label: 'Overview',
      items: [{ label: 'Dashboard', href: '/', icon: LayoutDashboard }]
    },
    {
      label: 'Analyze',
      items: [
        { label: 'Simple', href: '/#simple', icon: Scan },
        { label: 'Advanced', href: '/#advanced', icon: SlidersHorizontal }
      ]
    },
    {
      label: 'Manage',
      items: [
        { label: 'Profiles', href: '/#profiles', icon: Settings2 },
        { label: 'History', href: '/#history', icon: History }
      ]
    }
  ]

  const contextualNavItems = $derived(() => {
    const current = $location || '/'
    const items: Array<{ label: string; href: string; icon: typeof Activity }> =
      []

    if (current.startsWith('/run/')) {
      items.push({ label: 'Current Run', href: current, icon: Activity })
    }

    if (current.includes('/page/')) {
      items.push({ label: 'Current Page', href: current, icon: FileText })
    }

    return items
  })

  function isActive(href: string, loc: string): boolean {
    if (href === '/') return loc === '/' || loc === ''
    return loc === href.replace('/#', '/')
  }
</script>

<Sidebar collapsible="icon">
  <SidebarHeader>
    <div class="flex items-center gap-2 px-2 py-1" data-tauri-drag-region>
      <div
        class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-indigo-500 to-violet-600"
      >
        <Search class="h-4 w-4 text-white" />
      </div>
      <span
        class="font-semibold tracking-tight group-data-[collapsible=icon]:hidden"
        >PageLens</span
      >
    </div>
  </SidebarHeader>

  <SidebarContent>
    {#each primaryNavGroups as group}
      <SidebarGroup>
        <SidebarGroupLabel>{group.label}</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            {#each group.items as item}
              <SidebarMenuItem>
                <SidebarMenuButton
                  isActive={isActive(item.href, $location)}
                  tooltipContent={item.label}
                >
                  {#snippet child({ props })}
                    <a use:link href={item.href} {...props}>
                      <item.icon />
                      <span>{item.label}</span>
                    </a>
                  {/snippet}
                </SidebarMenuButton>
              </SidebarMenuItem>
            {/each}
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    {/each}

    {#if contextualNavItems().length > 0}
      <SidebarGroup>
        <SidebarGroupLabel>Context</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            {#each contextualNavItems() as item}
              <SidebarMenuItem>
                <SidebarMenuButton
                  isActive={isActive(item.href, $location)}
                  tooltipContent={item.label}
                >
                  {#snippet child({ props })}
                    <a use:link href={item.href} {...props}>
                      <item.icon />
                      <span>{item.label}</span>
                    </a>
                  {/snippet}
                </SidebarMenuButton>
              </SidebarMenuItem>
            {/each}
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    {/if}
  </SidebarContent>

  <SidebarFooter>
    <SidebarMenu>
      <SidebarMenuItem>
        <SidebarMenuButton onclick={toggleMode} tooltipContent="Toggle theme">
          {#if mode.current === 'dark'}
            <Sun class="h-4 w-4" />
            <span>Light mode</span>
          {:else}
            <Moon class="h-4 w-4" />
            <span>Dark mode</span>
          {/if}
        </SidebarMenuButton>
      </SidebarMenuItem>
    </SidebarMenu>
  </SidebarFooter>

  <SidebarRail />
</Sidebar>
