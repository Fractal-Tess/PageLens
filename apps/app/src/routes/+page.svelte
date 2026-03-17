<script lang="ts">
	import PixelBlast from '$lib/components/PixelBlast.svelte';

	const modes = [
		{
			id: '01',
			href: '/analyse/page',
			title: 'SINGLE PAGE',
			subtitle: 'ANALYSIS',
			desc: 'Deep inspection of any URL. Performance vitals, SEO metadata, DOM structure, assets, redirect chains, and detected issues — in real time.',
			tags: ['Performance', 'SEO', 'DOM structure', 'Assets', 'Network', 'Issues'],
			meta: 'Single URL · Headless browser',
			accentColor: '#B19EEF'
		},
		{
			id: '02',
			href: '/analyse/crawl',
			title: 'FULL SITE',
			subtitle: 'CRAWL',
			desc: 'Follow all internal links and map an entire site. Aggregate SEO metrics, site-wide issue detection, and complete page inventory.',
			tags: ['Multi-page spider', 'Link graph', 'Aggregate SEO', 'Site inventory', 'Cross-page issues', 'Coverage stats'],
			meta: 'Multiple pages · Spider',
			accentColor: '#7C5CFF'
		},
		{
			id: '03',
			href: '/analyse/benchmark',
			title: 'HTTP',
			subtitle: 'BENCHMARK',
			desc: 'Stress test any HTTP endpoint with concurrent requests. Measure RPS, p50/p95/p99 latency, throughput, and failure rates under load.',
			tags: ['Load testing', 'Requests/sec', 'Latency p95/p99', 'Concurrency', 'Status codes', 'Error rates'],
			meta: 'Single endpoint · Load test',
			accentColor: '#9D7FFF'
		},
		{
			id: '04',
			href: '/analyse/favicon',
			title: 'FAVICON',
			subtitle: 'ANALYZER',
			desc: 'Inspect favicon coverage for any page. Resolve HTML icon declarations, compare fallback paths, and preview candidate icon files in one pass.',
			tags: ['Icon discovery', 'Rel/sizes metadata', 'Default fallback', 'Candidate preview', 'Brand consistency', 'Asset quality'],
			meta: 'Single URL · Brand icon audit',
			accentColor: '#C4A9FF'
		},
		{
			id: '05',
			href: '/analyse/pwa',
			title: 'PWA',
			subtitle: 'ANALYZER',
			desc: 'Validate progressive web app readiness: manifest quality, service worker registration hints, theme metadata, and touch icon support.',
			tags: ['Manifest', 'Service worker', 'Installability', 'Theme color', 'Touch icons', 'PWA score'],
			meta: 'Single URL · PWA readiness',
			accentColor: '#A088F0'
		}
	] as const;
</script>

<svelte:head>
	<title>PAGELENS — Web Analysis</title>
</svelte:head>

<!-- Background -->
<div class="fixed inset-0 -z-10">
	<PixelBlast
		variant="square"
		pixelSize={4}
		color="#B19EEF"
		patternScale={2}
		patternDensity={1}
		pixelSizeJitter={0}
		enableRipples={true}
		rippleSpeed={0.4}
		rippleThickness={0.12}
		rippleIntensityScale={1.5}
		liquid={false}
		speed={0.5}
		edgeFade={0.25}
		transparent={true}
	/>
</div>

<div class="relative mx-auto max-w-7xl px-6 py-12">

	<!-- Compact Header -->
	<div class="mb-12 bg-background/60 backdrop-blur-sm">
		<div class="mb-2 text-[10px] tracking-[0.5em] text-muted-foreground uppercase">
			web analysis tool
		</div>
		<h1 class="text-5xl md:text-7xl font-black leading-none tracking-tighter mb-3">
			PAGE<span class="text-primary">LENS</span>
		</h1>
		<p class="text-xs leading-relaxed text-muted-foreground max-w-lg">
			Select an analysis mode. Single-page inspection, full-site crawl, or HTTP load testing.
		</p>
	</div>

	<!-- Mode Cards Grid -->
	<div class="mb-8">
		<div class="mb-4 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
			Select analysis mode —
		</div>

		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
			{#each modes as mode, index}
				<a
					href={mode.href}
					class="group relative bg-background/70 border border-border backdrop-blur-sm transition-all duration-200 hover:bg-background/90 hover:border-primary/50 hover:shadow-lg hover:-translate-y-1"
					style="--accent-color: {mode.accentColor}; animation-delay: {index * 40}ms"
				>
					<!-- Thick bottom border accent -->
					<div
						class="absolute bottom-0 left-0 right-0 h-1 transition-all duration-200 group-hover:h-1.5"
						style="background-color: var(--accent-color);"
					></div>

					<div class="p-5 pb-6">
						<!-- ID + Meta -->
						<div class="flex items-start justify-between mb-3">
							<span class="text-[10px] tracking-[0.3em] text-muted-foreground/50 uppercase">
								{mode.id}
							</span>
							<span
								class="text-xs text-muted-foreground/40 transition-all duration-200 group-hover:text-primary translate-x-0 group-hover:translate-x-0.5"
							>→</span>
						</div>

						<!-- Title -->
						<h2 class="text-lg md:text-xl font-black tracking-tight mb-2 group-hover:text-primary transition-colors duration-200">
							{mode.title}
							<span style="color: var(--accent-color);">{mode.subtitle}</span>
						</h2>

						<!-- Description -->
						<p class="text-xs leading-relaxed text-muted-foreground mb-4 line-clamp-3">
							{mode.desc}
						</p>

						<!-- Tags (first 3) -->
						<div class="flex flex-wrap gap-1 mb-4">
							{#each mode.tags.slice(0, 3) as tag}
								<span
									class="border border-border/50 px-1.5 py-0.5 text-[8px] tracking-widest text-muted-foreground/40 uppercase transition-colors duration-200 group-hover:border-primary/30 group-hover:text-muted-foreground/60"
								>{tag}</span>
							{/each}
							{#if mode.tags.length > 3}
								<span class="text-[8px] text-muted-foreground/30 self-center">
									+{mode.tags.length - 3}
								</span>
							{/if}
						</div>

						<!-- Card Meta -->
						<div class="text-[9px] leading-relaxed tracking-wider text-muted-foreground/40 uppercase">
							{mode.meta}
						</div>
					</div>
				</a>
			{/each}
		</div>
	</div>

	<!-- History footer bar -->
	<div
		class="flex items-center justify-between border border-border bg-background/70 px-5 py-3 backdrop-blur-sm"
	>
		<span class="text-[10px] tracking-wider text-muted-foreground/60 uppercase">
			Previous analysis runs
		</span>
		<a
			href="/history"
			class="text-[10px] tracking-widest text-primary uppercase transition-colors hover:text-primary/80"
		>
			VIEW HISTORY →
		</a>
	</div>
</div>
