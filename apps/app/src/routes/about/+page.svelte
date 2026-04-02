<script lang="ts">
  const capabilities = [
    {
      id: '01',
      title: 'Single Page Analysis',
      description: 'Deep-dive audits of any URL — performance, accessibility, SEO, security headers, and raw HTTP data in one shot.',
      accent: '#00D4FF',
      icon: '◎',
      tags: ['PERFORMANCE', 'SEO', 'SECURITY'],
    },
    {
      id: '02',
      title: 'Full Site Crawl',
      description: 'Spider an entire domain, map every route, surface broken links, and aggregate metrics across hundreds of pages.',
      accent: '#B19EEF',
      icon: '⬡',
      tags: ['CRAWL', 'MAPPING', 'LINKS'],
    },
    {
      id: '03',
      title: 'HTTP Benchmark',
      description: 'Hammer endpoints with configurable concurrency and request counts. Surface latency distributions and throughput limits.',
      accent: '#A3FF12',
      icon: '⚡',
      tags: ['LATENCY', 'THROUGHPUT', 'LOAD'],
    },
    {
      id: '04',
      title: 'Favicon Analyzer',
      description: 'Validate every icon size, format, and manifest declaration. Ensure perfect rendering across browsers and devices.',
      accent: '#FFB800',
      icon: '◈',
      tags: ['ICONS', 'MANIFEST', 'BRANDING'],
    },
    {
      id: '05',
      title: 'PWA Compliance',
      description: 'Score Progressive Web App readiness — service workers, offline capability, install criteria, and manifest completeness.',
      accent: '#FF6B6B',
      icon: '◉',
      tags: ['OFFLINE', 'INSTALL', 'WORKERS'],
    },
    {
      id: '06',
      title: 'History & Reports',
      description: 'Every run is persisted. Track regressions over time, compare snapshots, and export findings for your team.',
      accent: '#FF9F1C',
      icon: '▦',
      tags: ['HISTORY', 'COMPARE', 'EXPORT'],
    },
  ];
</script>

<svelte:head>
  <title>About — PageLens</title>
</svelte:head>

<!-- Page wrapper -->
<div class="relative flex min-h-[calc(100vh-3rem-2.5rem)] flex-col items-center justify-center gap-12 overflow-hidden px-6 py-12 pb-16">

  <!-- Scanline overlay -->
  <div class="scan-line pointer-events-none absolute inset-0 z-0" aria-hidden="true"></div>

  <!-- Background grid -->
  <div
    class="pointer-events-none absolute inset-0 z-0"
    style="background-image: linear-gradient(oklch(0.22 0 0 / 0.4) 1px, transparent 1px), linear-gradient(90deg, oklch(0.22 0 0 / 0.4) 1px, transparent 1px); background-size: 48px 48px; mask-image: radial-gradient(ellipse 80% 70% at 50% 40%, black 30%, transparent 100%);"
    aria-hidden="true"
  ></div>

  <!-- Hero -->
  <section class="animate-fade-in relative z-10 flex w-full max-w-[700px] flex-col items-center gap-7 text-center">

    <!-- Eyebrow -->
    <div class="flex items-center gap-4 text-[0.6rem] uppercase tracking-[0.5em] text-muted-foreground">
      <span class="h-px max-w-[80px] flex-1 bg-gradient-to-r from-transparent via-border to-transparent"></span>
      <span>WEB ANALYSIS SUITE</span>
      <span class="h-px max-w-[80px] flex-1 bg-gradient-to-r from-transparent via-border to-transparent"></span>
    </div>

    <!-- Logo -->
    <h1 class="m-0 flex items-center gap-2 text-[clamp(3rem,10vw,6.5rem)] font-black uppercase leading-none tracking-[0.12em]">
      <span class="text-[0.7em] text-primary [animation:pulse-glow_3s_ease-in-out_infinite] [filter:drop-shadow(0_0_20px_oklch(0.72_0.19_47_/_0.6))]">◈</span>
      <span class="text-foreground">PAGE</span><span class="text-primary [text-shadow:0_0_40px_oklch(0.72_0.19_47_/_0.35)]">LENS</span>
    </h1>

    <!-- Quote -->
    <blockquote class="stagger-1 animate-fade-in m-0 flex max-w-[520px] flex-col items-center gap-2 px-6 text-[0.95rem] italic leading-[1.7] text-muted-foreground">
      <p class="m-0 text-center">
        <span class="not-italic text-primary">"</span>Testing leads to failure, and failure leads to understanding.<span class="not-italic text-primary">"</span> <cite class="text-[0.65rem] not-italic uppercase tracking-[0.3em] text-[oklch(0.45_0_0)]">— Burt Rutan</cite>
      </p>
    </blockquote>

    <!-- Divider -->
    <div class="stagger-2 animate-fade-in flex w-full items-center gap-4">
      <span class="h-px flex-1 bg-border"></span>
      <span class="whitespace-nowrap text-[0.6rem] uppercase tracking-[0.45em] text-[oklch(0.45_0_0)]">CAPABILITIES</span>
      <span class="h-px flex-1 bg-border"></span>
    </div>
  </section>

  <!-- Capabilities grid -->
  <section class="stagger-2 animate-fade-in relative z-10 w-full max-w-[960px]">
    <div class="grid grid-cols-3 gap-px border border-border bg-border max-md:grid-cols-2 max-sm:grid-cols-1">
      {#each capabilities as cap, i}
        <div
          class="group relative flex cursor-default flex-col gap-3 overflow-hidden bg-[oklch(0.11_0_0)] p-6 transition-colors duration-200 hover:bg-card stagger-{Math.min(i + 1, 5)}"
          style="--cap-accent: {cap.accent}"
        >
          <!-- Left accent bar -->
          <div
            class="absolute left-0 top-0 h-full w-[3px] opacity-50 transition-opacity duration-200 group-hover:opacity-100"
            style="background: {cap.accent}; box-shadow: 0 0 12px {cap.accent};"
          ></div>

          <!-- Header -->
          <div class="flex items-center justify-between">
            <span class="text-[0.6rem] font-bold uppercase tracking-[0.25em] text-[oklch(0.35_0_0)]">{cap.id}</span>
            <span class="text-[1.1rem] leading-none [filter:drop-shadow(0_0_6px_currentColor)]" style="color: {cap.accent}">{cap.icon}</span>
          </div>

          <h3 class="m-0 text-[0.78rem] font-bold uppercase tracking-[0.06em] text-foreground/90">{cap.title}</h3>
          <p class="m-0 flex-1 text-[0.72rem] leading-[1.6] text-muted-foreground">{cap.description}</p>

          <!-- Tags -->
          <div class="flex flex-wrap gap-1.5">
            {#each cap.tags as tag}
              <span
                class="border px-[0.45rem] py-[0.2rem] text-[0.55rem] font-bold uppercase tracking-[0.25em]"
                style="color: {cap.accent}; border-color: {cap.accent}22;"
              >{tag}</span>
            {/each}
          </div>

          <!-- Hover glow -->
          <div
            class="pointer-events-none absolute -bottom-[30px] -left-[20px] h-20 w-20 rounded-full opacity-100 blur-[20px] transition-opacity duration-300 group-hover:opacity-0"
            style="background: radial-gradient(circle, {cap.accent} 0%, transparent 70%);"
          ></div>
        </div>
      {/each}
    </div>
  </section>

  <!-- Footer badge -->
  <div class="animate-fade-in stagger-5 relative z-10 flex items-center gap-3 text-[0.6rem] uppercase tracking-[0.35em] text-[oklch(0.35_0_0)]">
    <span class="h-1 w-1 rounded-full bg-primary shadow-[0_0_6px_oklch(0.72_0.19_47_/_0.7)]"></span>
    <span>PAGELENS / WEB ANALYSIS SUITE / v1.0</span>
    <span class="h-1 w-1 rounded-full bg-primary shadow-[0_0_6px_oklch(0.72_0.19_47_/_0.7)]"></span>
  </div>
</div>

<style>
  @keyframes pulse-glow {
    0%, 100% { filter: drop-shadow(0 0 20px oklch(0.72 0.19 47 / 0.5)); }
    50%       { filter: drop-shadow(0 0 35px oklch(0.72 0.19 47 / 0.9)); }
  }
</style>
