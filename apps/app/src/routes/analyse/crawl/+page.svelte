<script lang="ts">
	import { goto } from '$app/navigation';
	import { startAnalysis } from '$lib/api';
	import { Button } from '@pagelens/ui/shadcn/button';
	import { Input } from '@pagelens/ui/shadcn/input';
	import { z } from 'zod';

	let url = $state('');
	let loading = $state(false);
	let error = $state('');
	let maxPages = $state(50);
	let maxDepth = $state(3);
	let maxConcurrency = $state(4);

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
				analysis_type: 'crawl',
				crawl_options: {
					max_pages: Math.max(1, Math.floor(maxPages)),
					max_depth: Math.max(0, Math.floor(maxDepth)),
					max_concurrency: Math.max(1, Math.floor(maxConcurrency))
				}
			});
			goto(`/run/${run_id}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
			loading = false;
		}
	}

	const crawlStats = [
		{ label: 'Avg SEO Score', desc: 'Aggregate across all pages' },
		{ label: 'Total Pages', desc: 'Crawled page count' },
		{ label: 'Failed Pages', desc: 'Unreachable or errored' },
		{ label: 'External Links', desc: 'Outbound link count' },
		{ label: 'Total Issues', desc: 'Site-wide problem count' },
		{ label: 'Total Errors', desc: 'Critical failures' },
		{ label: 'Total Warnings', desc: 'Non-critical notices' },
		{ label: 'Crawl Time', desc: 'Total duration (ms)' }
	];
</script>

<svelte:head>
	<title>PAGELENS — Full Site Crawl</title>
</svelte:head>

<div class="mx-auto max-w-6xl px-6 py-8">

	<!-- Breadcrumb -->
	<div class="animate-fade-in mb-6 flex items-center gap-2 text-[10px] tracking-widest text-muted-foreground uppercase">
		<a href="/" class="transition-colors hover:text-foreground">PAGELENS</a>
		<span class="text-border">/</span>
		<span class="text-primary">FULL CRAWL</span>
	</div>

	<!-- Header block -->
	<div class="animate-fade-in-up stagger-1 mb-8 grid grid-cols-1 gap-px bg-border md:grid-cols-[2fr_1fr]">
		<div class="bg-background p-5 md:p-6">
			<div class="mb-2 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
				02 · Analysis mode
			</div>
			<h1 class="text-3xl font-black leading-none tracking-tighter md:text-4xl">
				FULL SITE <span class="text-primary">CRAWL</span>
			</h1>
			<p class="mt-3 max-w-sm text-xs leading-relaxed text-muted-foreground">
				Starts from the seed URL and follows all internal links, building a complete map of your site.
				Each discovered page is fully analysed — same depth as single-page mode, at scale.
			</p>
		</div>
		<div class="bg-background p-5 md:p-6">
			<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
				How it works
			</div>
			<div class="space-y-3">
				<div class="border-l border-primary/40 pl-4">
					<div class="mb-1 text-[10px] font-bold tracking-widest text-primary uppercase">1. Seed</div>
					<div class="text-[11px] leading-relaxed text-muted-foreground">
						Begin from the provided URL. Analyse and extract all internal links.
					</div>
				</div>
				<div class="border-l border-primary/40 pl-4">
					<div class="mb-1 text-[10px] font-bold tracking-widest text-primary uppercase">2. Queue</div>
					<div class="text-[11px] leading-relaxed text-muted-foreground">
						Enqueue discovered internal links. Skip external domains, duplicates, and already-visited pages.
					</div>
				</div>
				<div class="border-l border-primary/40 pl-4">
					<div class="mb-1 text-[10px] font-bold tracking-widest text-primary uppercase">3. Analyse</div>
					<div class="text-[11px] leading-relaxed text-muted-foreground">
						Full headless browser analysis on each page: performance, SEO, DOM, assets, issues.
					</div>
				</div>
				<div class="border-l border-border pl-4">
					<div class="mb-1 text-[10px] font-bold tracking-widest text-muted-foreground uppercase">4. Aggregate</div>
					<div class="text-[11px] leading-relaxed text-muted-foreground">
						Roll up site-wide stats: avg SEO score, total issues, coverage, and inventory.
					</div>
				</div>
			</div>
		</div>
	</div>

	<!-- Form -->
	<form onsubmit={handleSubmit} class="animate-fade-in-up stagger-2 mb-8">
		<div class="border border-border bg-background p-8">
			<div class="mb-1 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
				Seed URL
			</div>
			<p class="mb-6 text-xs text-muted-foreground">
				The crawler starts here and follows internal links recursively. For best results, use your site's root URL.
			</p>

			<div class="flex gap-0">
				<Input
					bind:value={url}
					type="url"
					placeholder="https://example.com"
					required
					disabled={loading}
					class="h-14 flex-1 border border-r-0 border-border bg-transparent px-4 font-mono text-sm focus-visible:ring-0 focus-visible:border-primary rounded-none placeholder:text-muted-foreground/30"
				/>
				<Button
					type="submit"
					disabled={loading || !isValidUrl(url)}
					class="h-14 rounded-none border border-primary bg-primary px-10 text-xs font-bold tracking-widest uppercase text-primary-foreground hover:bg-primary/90 disabled:opacity-40"
				>
					{loading ? 'STARTING...' : 'CRAWL SITE →'}
				</Button>
			</div>

			<div class="mt-6 grid grid-cols-1 gap-3 md:grid-cols-3">
				<div>
					<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Max pages</div>
					<Input
						type="number"
						min="1"
						step="1"
						bind:value={maxPages}
						disabled={loading}
						class="h-10 border border-border bg-transparent px-3 font-mono text-xs rounded-none"
					/>
				</div>
				<div>
					<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Max depth</div>
					<Input
						type="number"
						min="0"
						step="1"
						bind:value={maxDepth}
						disabled={loading}
						class="h-10 border border-border bg-transparent px-3 font-mono text-xs rounded-none"
					/>
				</div>
				<div>
					<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Workers</div>
					<Input
						type="number"
						min="1"
						step="1"
						bind:value={maxConcurrency}
						disabled={loading}
						class="h-10 border border-border bg-transparent px-3 font-mono text-xs rounded-none"
					/>
				</div>
			</div>

			{#if error}
				<div class="mt-4 border border-destructive/40 bg-destructive/10 px-4 py-3 text-xs text-destructive">
					ERROR: {error}
				</div>
			{/if}

			<!-- Crawl note -->
			<div class="mt-6 flex gap-3 border border-border bg-secondary/30 px-4 py-3">
				<span class="shrink-0 text-[10px] text-muted-foreground/60">NOTE</span>
				<p class="text-[11px] leading-relaxed text-muted-foreground">
					Crawl time scales with site size. Large sites may take several minutes.
					Progress is streamed in real time on the run page. Only pages within the same domain as the seed URL will be crawled.
				</p>
			</div>
		</div>
	</form>

	<!-- Aggregate stats grid -->
	<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
		Aggregate output
	</div>
	<div class="grid grid-cols-2 gap-px bg-border md:grid-cols-4">
		{#each crawlStats as m}
			<div class="bg-background p-5">
				<div class="mb-1.5 text-[10px] font-bold tracking-widest text-primary uppercase">{m.label}</div>
				<div class="text-[10px] leading-relaxed text-muted-foreground">{m.desc}</div>
			</div>
		{/each}
	</div>

	<!-- Bottom nav -->
	<div class="mt-px flex items-center justify-between border border-border bg-background px-6 py-3.5">
		<a href="/analyse/page" class="text-[10px] tracking-widest text-muted-foreground uppercase transition-colors hover:text-foreground">
			← SINGLE PAGE
		</a>
		<a href="/analyse/benchmark" class="text-[10px] tracking-widest text-primary uppercase transition-colors hover:text-primary/80">
			HTTP BENCHMARK →
		</a>
	</div>
</div>
