<script lang="ts">
	import { goto } from '$app/navigation';
	import { startAnalysis } from '$lib/api';
	import { Button } from '@pagelens/ui/shadcn/button';
	import { Input } from '@pagelens/ui/shadcn/input';
	import { z } from 'zod';

	let url = $state('');
	let loading = $state(false);
	let error = $state('');

	const urlSchema = z.string().trim().url().startsWith('http');

	function isValidUrl(str: string): boolean {
		return urlSchema.safeParse(str).success;
	}

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!url.trim()) return;
		loading = true;
		error = '';
		try {
			const { run_id } = await startAnalysis({
				url: url.trim(),
				analysis_type: 'single'
			});
			goto(`/run/${run_id}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
			loading = false;
		}
	}

	const outputMetrics = [
		{ label: 'SEO Score', desc: '0–100 composite score' },
		{ label: 'TTFB', desc: 'Time to first byte (ms)' },
		{ label: 'FCP', desc: 'First contentful paint' },
		{ label: 'LCP', desc: 'Largest contentful paint' },
		{ label: 'Load Time', desc: 'Full page load (ms)' },
		{ label: 'DOM Depth', desc: 'Element nesting depth' },
		{ label: 'Issues', desc: 'Total detected problems' },
		{ label: 'Assets', desc: 'JS, CSS, images, fonts' }
	];
</script>

<svelte:head>
	<title>PAGELENS — Single Page Analysis</title>
</svelte:head>

<div class="mx-auto max-w-6xl px-6 py-16">

	<!-- Breadcrumb -->
	<div class="mb-12 flex items-center gap-2 text-[10px] tracking-widest text-muted-foreground uppercase">
		<a href="/" class="transition-colors hover:text-foreground">PAGELENS</a>
		<span class="text-border">/</span>
		<span class="text-primary">SINGLE PAGE</span>
	</div>

	<!-- Header block -->
	<div class="mb-10 grid grid-cols-1 gap-px bg-border md:grid-cols-[2fr_1fr]">
		<div class="bg-background p-8">
			<div class="mb-4 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
				01 · Analysis mode
			</div>
			<h1 class="text-5xl font-black leading-none tracking-tighter md:text-6xl">
				SINGLE<br />PAGE<br /><span class="text-primary">ANALYSIS</span>
			</h1>
			<p class="mt-5 max-w-sm text-xs leading-relaxed text-muted-foreground">
				Loads the target URL in a headless browser and captures all available data:
				performance vitals, full SEO report, DOM structure, network timings, assets, and detected issues.
			</p>
		</div>
		<div class="bg-background p-8">
			<div class="mb-5 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
				What's captured
			</div>
			<div class="space-y-2.5">
				{#each [
					'Performance vitals (TTFB, FCP, LCP)',
					'SEO score & metadata',
					'Open Graph / Twitter Card',
					'Heading structure (h1–h6)',
					'DOM depth & element counts',
					'Images & alt text coverage',
					'Network requests & redirect chains',
					'JavaScript, CSS, font assets',
					'Structured data (schema.org)',
					'Detected issues & warnings'
				] as item}
					<div class="flex items-baseline gap-2 text-[11px] text-muted-foreground">
						<span class="shrink-0 text-primary/60">—</span>
						<span>{item}</span>
					</div>
				{/each}
			</div>
		</div>
	</div>

	<!-- Form -->
	<form onsubmit={handleSubmit} class="mb-10">
		<div class="border border-border bg-background p-8">
			<div class="mb-1 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
				Target URL
			</div>
			<p class="mb-6 text-xs text-muted-foreground">
				Enter the full URL of the page to analyse. Must start with <span class="text-foreground/80">http://</span> or <span class="text-foreground/80">https://</span>
			</p>

			<div class="flex gap-0">
				<Input
					bind:value={url}
					type="url"
					placeholder="https://example.com/page"
					required
					disabled={loading}
					class="h-14 flex-1 border border-r-0 border-border bg-transparent px-4 font-mono text-sm focus-visible:ring-0 focus-visible:border-primary rounded-none placeholder:text-muted-foreground/30"
				/>
				<Button
					type="submit"
					disabled={loading || !isValidUrl(url)}
					class="h-14 rounded-none border border-primary bg-primary px-10 text-xs font-bold tracking-widest uppercase text-primary-foreground hover:bg-primary/90 disabled:opacity-40"
				>
					{loading ? 'STARTING...' : 'ANALYSE PAGE →'}
				</Button>
			</div>

			{#if error}
				<div class="mt-4 border border-destructive/40 bg-destructive/10 px-4 py-3 text-xs text-destructive">
					ERROR: {error}
				</div>
			{/if}
		</div>
	</form>

	<!-- Output metrics grid -->
	<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
		Output metrics
	</div>
	<div class="grid grid-cols-2 gap-px bg-border md:grid-cols-4">
		{#each outputMetrics as m}
			<div class="bg-background p-5">
				<div class="mb-1.5 text-[10px] font-bold tracking-widest text-primary uppercase">{m.label}</div>
				<div class="text-[10px] leading-relaxed text-muted-foreground">{m.desc}</div>
			</div>
		{/each}
	</div>

	<!-- Bottom nav -->
	<div class="mt-px flex items-center justify-between border border-border bg-background px-6 py-3.5">
		<a href="/" class="text-[10px] tracking-widest text-muted-foreground uppercase transition-colors hover:text-foreground">
			← ALL MODES
		</a>
		<a href="/analyse/crawl" class="text-[10px] tracking-widest text-primary uppercase transition-colors hover:text-primary/80">
			FULL CRAWL →
		</a>
	</div>
</div>
