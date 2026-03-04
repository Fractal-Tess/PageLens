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
			meta: 'Single URL · Headless browser'
		},
		{
			id: '02',
			href: '/analyse/crawl',
			title: 'FULL SITE',
			subtitle: 'CRAWL',
			desc: 'Follow all internal links and map an entire site. Aggregate SEO metrics, site-wide issue detection, and complete page inventory.',
			tags: ['Multi-page spider', 'Link graph', 'Aggregate SEO', 'Site inventory', 'Cross-page issues', 'Coverage stats'],
			meta: 'Multiple pages · Spider'
		},
		{
			id: '03',
			href: '/analyse/benchmark',
			title: 'HTTP',
			subtitle: 'BENCHMARK',
			desc: 'Stress test any HTTP endpoint with concurrent requests. Measure RPS, p50/p95/p99 latency, throughput, and failure rates under load.',
			tags: ['Load testing', 'Requests/sec', 'Latency p95/p99', 'Concurrency', 'Status codes', 'Error rates'],
			meta: 'Single endpoint · Load test'
		},
		{
			id: '04',
			href: '/analyse/favicon',
			title: 'FAVICON',
			subtitle: 'ANALYZER',
			desc: 'Inspect favicon coverage for any page. Resolve HTML icon declarations, compare fallback paths, and preview candidate icon files in one pass.',
			tags: ['Icon discovery', 'Rel/sizes metadata', 'Default fallback', 'Candidate preview', 'Brand consistency', 'Asset quality'],
			meta: 'Single URL · Brand icon audit'
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

<div class="relative mx-auto max-w-6xl px-6 py-16">

	<!-- Header -->
	<div class="mb-16 border-b border-border pb-10 bg-background/60 backdrop-blur-sm px-0">
		<div class="mb-4 text-[10px] tracking-[0.5em] text-muted-foreground uppercase">
			web analysis tool
		</div>
		<h1 class="text-[80px] font-black leading-none tracking-tighter md:text-[112px]">
			PAGE<span class="text-primary">LENS</span>
		</h1>
		<p class="mt-5 max-w-xl text-xs leading-relaxed text-muted-foreground">
			Select an analysis mode below. Single-page inspection, full-site crawl, or HTTP load testing.
		</p>
	</div>

	<!-- Mode list -->
	<div class="mb-2 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
		Select analysis mode —
	</div>

	<div class="border-t border-border">
		{#each modes as mode}
			<a
				href={mode.href}
				class="group block border-b border-border bg-background/80 backdrop-blur-sm transition-colors hover:bg-secondary/80"
			>
				<div class="grid grid-cols-[40px_1fr] gap-6 px-6 py-8 md:grid-cols-[60px_1fr_200px]">
					<!-- Number -->
					<div class="text-[10px] tracking-[0.3em] text-muted-foreground/50 pt-1">{mode.id}</div>

					<!-- Content -->
					<div>
						<div class="mb-3 flex items-baseline gap-3">
							<h2 class="text-2xl font-black tracking-tight md:text-3xl">
								{mode.title}
								<span class="text-primary">{mode.subtitle}</span>
							</h2>
							<span
								class="translate-x-0 text-lg text-muted-foreground/40 transition-all duration-200 group-hover:translate-x-2 group-hover:text-primary"
							>→</span>
						</div>
						<p class="mb-5 max-w-2xl text-xs leading-relaxed text-muted-foreground">
							{mode.desc}
						</p>
						<div class="flex flex-wrap gap-1.5">
							{#each mode.tags as tag}
								<span
									class="border border-border px-2 py-0.5 text-[9px] tracking-widest text-muted-foreground/50 uppercase"
								>{tag}</span>
							{/each}
						</div>
					</div>

					<!-- Meta (desktop) -->
					<div class="hidden pt-1 text-right md:block">
						<div class="text-[10px] leading-relaxed tracking-wider text-muted-foreground/40 uppercase">
							{mode.meta}
						</div>
					</div>
				</div>
			</a>
		{/each}
	</div>

	<!-- History footer bar -->
	<div
		class="mt-px flex items-center justify-between border border-border bg-background/80 px-6 py-3.5 backdrop-blur-sm"
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
