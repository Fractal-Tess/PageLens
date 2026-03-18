<script lang="ts">
	const tagCap = 3;

	const modes = [
		{
			id: '01',
			href: '/analyse/page',
			title: 'SINGLE PAGE',
			subtitle: 'ANALYSIS',
			desc: 'Deep inspection of any URL. Performance vitals, SEO metadata, DOM structure, assets, redirect chains, and detected issues — in real time.',
			tags: ['Performance', 'SEO', 'DOM structure', 'Assets', 'Network', 'Issues'],
			meta: 'Single URL · Headless browser',
			accent: '#00D4FF'
		},
		{
			id: '02',
			href: '/analyse/crawl',
			title: 'FULL SITE',
			subtitle: 'CRAWL',
			desc: 'Follow all internal links and map an entire site. Aggregate SEO metrics, site-wide issue detection, and complete page inventory.',
			tags: ['Multi-page', 'Link graph', 'Aggregate SEO', 'Site inventory', 'Cross-page issues'],
			meta: 'Multiple pages · Spider',
			accent: '#B19EEF'
		},
		{
			id: '03',
			href: '/analyse/benchmark',
			title: 'HTTP',
			subtitle: 'BENCHMARK',
			desc: 'Stress test any HTTP endpoint with concurrent requests. Measure RPS, p50/p95/p99 latency, throughput, and failure rates under load.',
			tags: ['Load testing', 'Requests/sec', 'Latency p95/p99', 'Concurrency', 'Error rates'],
			meta: 'Single endpoint · Load test',
			accent: '#A3FF12'
		},
		{
			id: '04',
			href: '/analyse/favicon',
			title: 'FAVICON',
			subtitle: 'ANALYZER',
			desc: 'Inspect favicon coverage for any page. Resolve HTML icon declarations, compare fallback paths, and preview candidate icon files.',
			tags: ['Icon discovery', 'Rel/sizes', 'Default fallback', 'Candidate preview', 'Brand audit'],
			meta: 'Single URL · Brand icon audit',
			accent: '#FFB800'
		},
		{
			id: '05',
			href: '/analyse/pwa',
			title: 'PWA',
			subtitle: 'ANALYZER',
			desc: 'Validate progressive web app readiness: manifest quality, service worker registration, theme metadata, and touch icon support.',
			tags: ['Manifest', 'Service worker', 'Installability', 'Theme color', 'Touch icons'],
			meta: 'Single URL · PWA readiness',
			accent: '#FF6B6B'
		}
	] as const;
</script>

<svelte:head>
	<title>PAGELENS — Web Analysis</title>
</svelte:head>

<div class="relative z-10 h-[calc(100svh-5.5rem)] flex flex-col px-6 overflow-y-auto">
	<div class="w-full max-w-[1100px] m-auto">
		<!-- Slim Header -->
		<header class="animate-fade-in flex items-end justify-between mb-6 pb-4 border-b border-border/30">
			<div class="flex items-baseline gap-4">
				<h1 class="text-3xl md:text-4xl font-black tracking-tighter leading-none">
					PAGE<span class="text-primary">LENS</span>
				</h1>
				<span
					class="hidden sm:inline text-[10px] tracking-[0.4em] text-muted-foreground/50 uppercase"
				>
					web analysis tool
				</span>
			</div>
			<a
				href="/history"
				class="text-[10px] tracking-widest text-muted-foreground/40 uppercase transition-colors hover:text-primary flex items-center gap-1.5"
			>
				HISTORY <span class="text-sm">→</span>
			</a>
		</header>

		<!-- Bento Grid -->
		<div class="bento-grid grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
			{#each modes as mode, i}
				{@const hero = i === 0}
				{@const wide = i === 4}
				<a
					href={mode.href}
					class="card-glow animate-fade-in-up stagger-{i + 1} group relative overflow-hidden bg-background/60 border border-border/50 backdrop-blur-sm transition-all duration-200 hover:-translate-y-0.5
						{hero ? 'md:col-span-2 md:row-span-2' : ''}
						{wide ? 'xl:col-span-2' : ''}"
					style="--accent: {mode.accent};"
				>
					<!-- Accent wash -->
					<div
						class="absolute inset-0 opacity-[0.03] group-hover:opacity-[0.07] transition-opacity duration-300 pointer-events-none"
						style="background: linear-gradient(135deg, var(--accent), transparent 60%);"
					></div>

					<!-- Left rail -->
					<div
						class="absolute left-0 top-0 bottom-0 w-[3px] transition-all duration-200 group-hover:w-[5px]"
						style="background: var(--accent); box-shadow: 0 0 8px var(--accent);"
					></div>

					<div
						class="relative h-full flex flex-col {hero
							? 'p-4 xl:p-6 pl-5 xl:pl-7'
							: 'p-3.5 pl-5'}"
					>
						<!-- ID + arrow -->
						<div class="flex items-center justify-between mb-2">
							<span
								class="inline-flex items-center justify-center w-6 h-6 text-[9px] font-bold border tracking-wider"
								style="border-color: var(--accent); color: var(--accent);"
							>
								{mode.id}
							</span>
							<span
								class="text-sm text-muted-foreground/20 transition-all duration-200 group-hover:text-[var(--accent)] group-hover:translate-x-0.5"
							>
								→
							</span>
						</div>

						<!-- Title -->
						<h2
							class="{hero
								? 'text-base xl:text-2xl xl:mb-2'
								: 'text-base'} font-black tracking-tight mb-1.5 transition-colors duration-200 group-hover:text-[var(--accent)]"
						>
							{mode.title}<span class="font-light opacity-50 ml-2">{mode.subtitle}</span>
						</h2>

						<!-- Desc -->
						<p
							class="{hero
								? 'xl:text-xs'
								: 'text-[11px] line-clamp-2'} text-[11px] leading-relaxed text-muted-foreground/50 mb-3"
						>
							{mode.desc}
						</p>

						<div class="flex-1"></div>

						<!-- Tags -->
						<div class="flex flex-wrap gap-1 mb-2">
							{#each mode.tags.slice(0, tagCap) as tag}
								<span
									class="border border-border/40 px-2 py-0.5 text-[11px] tracking-widest text-muted-foreground/35 uppercase transition-colors duration-200 group-hover:border-border group-hover:text-muted-foreground/60"
								>
									{tag}
								</span>
							{/each}
							{#if mode.tags.length > tagCap}
								<span class="text-[11px] text-muted-foreground/25 self-center">
									+{mode.tags.length - tagCap}
								</span>
							{/if}
						</div>

						<!-- Meta -->
						<div class="text-[11px] tracking-wider text-muted-foreground/30 uppercase">
							{mode.meta}
						</div>
					</div>
				</a>
			{/each}
		</div>
	</div>
</div>

<style>
	.card-glow {
		transition:
			border-color 0.2s,
			box-shadow 0.2s,
			transform 0.2s;
	}
	.card-glow:hover {
		border-color: color-mix(in srgb, var(--accent) 35%, transparent);
		box-shadow:
			0 0 25px -5px var(--accent),
			0 4px 12px rgba(0, 0, 0, 0.3);
	}

	.bento-grid {
		grid-auto-rows: minmax(110px, auto);
	}
</style>
