<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { analyzePwa, type PwaAnalysisResult } from '$lib/api';
	import { Button } from '@pagelens/ui/shadcn/button';
	import { Input } from '@pagelens/ui/shadcn/input';
	import { z } from 'zod';

	let url = $state('');
	let loading = $state(false);
	let error = $state('');
	let result = $state<PwaAnalysisResult | null>(null);
	let lastAnalyzedUrl = $state('');

	const urlSchema = z.string().trim().url().startsWith('http');

	function isValidUrl(str: string): boolean {
		return urlSchema.safeParse(str).success;
	}

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!url.trim()) return;
		await goto(`/analyse/pwa?url=${encodeURIComponent(url.trim())}`);
	}

	const requestedUrl = $derived(page.url.searchParams.get('url')?.trim() ?? '');

	$effect(() => {
		if (!requestedUrl || !isValidUrl(requestedUrl)) return;
		if (requestedUrl === lastAnalyzedUrl) return;

		url = requestedUrl;
		lastAnalyzedUrl = requestedUrl;
		loading = true;
		error = '';
		result = null;

		void analyzePwa(requestedUrl)
			.then((value) => {
				result = value;
			})
			.catch((err: unknown) => {
				error = err instanceof Error ? err.message : 'Unknown error';
			})
			.finally(() => {
				loading = false;
			});
	});

	const recommendations = $derived.by(() => {
		if (!result) return [] as string[];
		const recs: string[] = [];
		if (!result.has_manifest) recs.push('Add a web app manifest linked via <link rel="manifest">.');
		if (!result.has_service_worker_registration) recs.push('Register a service worker for offline capability and installability.');
		if (!result.has_theme_color_meta) recs.push('Declare <meta name="theme-color"> for branded browser chrome.');
		if (result.apple_touch_icon_count === 0) recs.push('Provide at least one apple-touch-icon for iOS home screen integration.');
		if (result.manifest && result.manifest.icon_count === 0) recs.push('Add 192x192 and 512x512 icons to the manifest icons array.');
		return recs;
	});
</script>

<svelte:head>
	<title>PAGELENS — PWA Analyzer</title>
</svelte:head>

<div class="mx-auto max-w-6xl px-6 py-8">
	<div class="animate-fade-in mb-6 flex items-center gap-2 text-[10px] tracking-widest text-muted-foreground uppercase">
		<a href="/" class="transition-colors hover:text-foreground">PAGELENS</a>
		<span class="text-border">/</span>
		<span class="text-primary">PWA ANALYZER</span>
	</div>

	<div class="animate-fade-in-up stagger-1 mb-8 grid grid-cols-1 gap-px bg-border md:grid-cols-[2fr_1fr]">
		<div class="bg-background p-5 md:p-6">
			<div class="mb-2 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">05 · Analysis mode</div>
			<h1 class="text-3xl font-black leading-none tracking-tighter md:text-4xl">
				PWA <span class="text-primary">ANALYZER</span>
			</h1>
			<p class="mt-3 max-w-sm text-xs leading-relaxed text-muted-foreground">
				Inspect installability signals and PWA readiness: manifest, service worker hints,
				theme metadata, and touch icon coverage.
			</p>
		</div>
		<div class="bg-background p-5 md:p-6">
			<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Checks</div>
			<div class="space-y-1.5">
				{#each ['Web app manifest link and fetch', 'Manifest metadata and icon count', 'Service worker registration hints', 'Theme-color meta declaration', 'iOS touch icon coverage'] as item}
					<div class="flex items-baseline gap-2 text-[11px] text-muted-foreground">
						<span class="shrink-0 text-primary/60">—</span>
						<span>{item}</span>
					</div>
				{/each}
			</div>
		</div>
	</div>

	<form onsubmit={handleSubmit} class="animate-fade-in-up stagger-2 mb-8">
		<div class="border border-border bg-background p-8">
			<div class="mb-1 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Target URL</div>
			<p class="mb-6 text-xs text-muted-foreground">Analyze a page for PWA readiness and installability indicators.</p>

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
					{loading ? 'ANALYZING...' : 'ANALYZE PWA →'}
				</Button>
			</div>

			{#if error}
				<div class="mt-4 border border-destructive/40 bg-destructive/10 px-4 py-3 text-xs text-destructive">ERROR: {error}</div>
			{/if}
		</div>
	</form>

	{#if result}
		<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">PWA score</div>
		<div class="mb-6 grid grid-cols-2 gap-px bg-border md:grid-cols-5">
			<div class="bg-background p-5">
				<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Installability</div>
				<div class="font-mono text-2xl font-black {result.installability_score >= 70 ? 'text-green-400' : result.installability_score >= 40 ? 'text-yellow-300' : 'text-destructive'}">{result.installability_score}</div>
			</div>
			<div class="bg-background p-5">
				<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Manifest</div>
				<div class="font-mono text-xs uppercase">{result.has_manifest ? 'Yes' : 'No'}</div>
			</div>
			<div class="bg-background p-5">
				<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Service Worker</div>
				<div class="font-mono text-xs uppercase">{result.has_service_worker_registration ? 'Detected' : 'Not detected'}</div>
			</div>
			<div class="bg-background p-5">
				<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Theme Color</div>
				<div class="font-mono text-xs uppercase">{result.has_theme_color_meta ? 'Present' : 'Missing'}</div>
			</div>
			<div class="bg-background p-5">
				<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Touch Icons</div>
				<div class="font-mono text-xs">{result.apple_touch_icon_count}</div>
			</div>
		</div>

		{#if result.manifest}
			<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Manifest details</div>
			<div class="mb-6 grid grid-cols-1 gap-px bg-border md:grid-cols-2 lg:grid-cols-4">
				<div class="bg-background p-4"><div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Name</div><div class="font-mono text-xs">{result.manifest.name ?? '—'}</div></div>
				<div class="bg-background p-4"><div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Short name</div><div class="font-mono text-xs">{result.manifest.short_name ?? '—'}</div></div>
				<div class="bg-background p-4"><div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Display</div><div class="font-mono text-xs uppercase">{result.manifest.display ?? '—'}</div></div>
				<div class="bg-background p-4"><div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Icons</div><div class="font-mono text-xs">{result.manifest.icon_count}</div></div>
			</div>
		{/if}

		<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Recommendations</div>
		<div class="border border-border bg-background px-4 py-3 text-xs">
			{#if recommendations.length > 0}
				<div class="space-y-1.5 text-muted-foreground">
					{#each recommendations as recommendation}
						<div>• {recommendation}</div>
					{/each}
				</div>
			{:else}
				<div class="text-green-400">No immediate recommendations. Baseline PWA signals look good.</div>
			{/if}
		</div>

		{#if result.warnings.length > 0}
			<div class="mt-6 border border-yellow-500/40 bg-yellow-500/10 px-4 py-3">
				<div class="mb-2 text-[10px] tracking-widest text-yellow-300 uppercase">Warnings</div>
				<div class="space-y-1.5 text-xs text-yellow-200/90">
					{#each result.warnings as warning}
						<div>• {warning}</div>
					{/each}
				</div>
			</div>
		{/if}
	{/if}
</div>
