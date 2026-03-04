<script lang="ts">
	import { analyzeFavicon, type FaviconAnalysisResult } from '$lib/api';
	import { Button } from '@pagelens/ui/shadcn/button';
	import { Input } from '@pagelens/ui/shadcn/input';
	import { z } from 'zod';

	let url = $state('');
	let loading = $state(false);
	let error = $state('');
	let result = $state<FaviconAnalysisResult | null>(null);

	const urlSchema = z.string().trim().url().startsWith('http');

	function isValidUrl(str: string): boolean {
		return urlSchema.safeParse(str).success;
	}

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!url.trim()) return;
		loading = true;
		error = '';
		result = null;

		try {
			result = await analyzeFavicon(url.trim());
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>PAGELENS — Favicon Analyzer</title>
</svelte:head>

<div class="mx-auto max-w-6xl px-6 py-16">
	<div class="mb-12 flex items-center gap-2 text-[10px] tracking-widest text-muted-foreground uppercase">
		<a href="/" class="transition-colors hover:text-foreground">PAGELENS</a>
		<span class="text-border">/</span>
		<span class="text-primary">FAVICON ANALYZER</span>
	</div>

	<div class="mb-10 grid grid-cols-1 gap-px bg-border md:grid-cols-[2fr_1fr]">
		<div class="bg-background p-8">
			<div class="mb-4 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">04 · Analysis mode</div>
			<h1 class="text-5xl font-black leading-none tracking-tighter md:text-6xl">
				FAVICON<br />
				<span class="text-primary">ANALYZER</span>
			</h1>
			<p class="mt-5 max-w-sm text-xs leading-relaxed text-muted-foreground">
				Extract and review all favicon candidates for a URL: HTML-declared icons, sizes,
				MIME hints, and the default <span class="font-mono text-foreground/80">/favicon.ico</span> fallback.
			</p>
		</div>
		<div class="bg-background p-8">
			<div class="mb-5 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Checks</div>
			<div class="space-y-2.5">
				{#each ['HTML icon declarations', 'Resolved absolute icon URLs', 'Declared sizes and MIME types', 'Default /favicon.ico fallback', 'Duplicate candidate suppression'] as item}
					<div class="flex items-baseline gap-2 text-[11px] text-muted-foreground">
						<span class="shrink-0 text-primary/60">—</span>
						<span>{item}</span>
					</div>
				{/each}
			</div>
		</div>
	</div>

	<form onsubmit={handleSubmit} class="mb-10">
		<div class="border border-border bg-background p-8">
			<div class="mb-1 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Target URL</div>
			<p class="mb-6 text-xs text-muted-foreground">
				Provide the page URL to inspect for favicon sources.
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
					{loading ? 'ANALYZING...' : 'ANALYZE FAVICON →'}
				</Button>
			</div>

			{#if error}
				<div class="mt-4 border border-destructive/40 bg-destructive/10 px-4 py-3 text-xs text-destructive">
					ERROR: {error}
				</div>
			{/if}
		</div>
	</form>

	{#if result}
		<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Analysis result</div>
		<div class="mb-6 grid grid-cols-1 gap-px bg-border md:grid-cols-3">
			<div class="bg-background p-5">
				<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Input URL</div>
				<div class="break-all font-mono text-xs">{result.input_url}</div>
			</div>
			<div class="bg-background p-5">
				<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Resolved page</div>
				<div class="break-all font-mono text-xs">{result.resolved_page_url}</div>
			</div>
			<div class="bg-background p-5">
				<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Candidates</div>
				<div class="font-mono text-xs">{result.candidates.length}</div>
			</div>
		</div>

		<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Favicon candidates</div>
		<div class="border border-border bg-background">
			<div class="grid gap-px bg-border md:grid-cols-2">
				{#each result.candidates as icon}
					<div class="bg-background p-5">
						<div class="mb-4 flex items-start gap-3">
							<div class="h-10 w-10 shrink-0 border border-border bg-secondary/40 p-1.5">
								<img src={icon.url} alt="favicon candidate" class="h-full w-full object-contain" loading="lazy" />
							</div>
							<div class="min-w-0 flex-1">
								<div class="mb-1 text-[10px] font-bold tracking-widest text-primary uppercase">{icon.source}</div>
								<div class="break-all font-mono text-[11px] leading-relaxed text-muted-foreground">{icon.url}</div>
							</div>
						</div>
						<div class="grid grid-cols-3 gap-px bg-border text-[10px]">
							<div class="bg-background px-3 py-2">
								<div class="mb-1 tracking-widest text-muted-foreground uppercase">Rel</div>
								<div class="font-mono text-foreground/90">{icon.rel}</div>
							</div>
							<div class="bg-background px-3 py-2">
								<div class="mb-1 tracking-widest text-muted-foreground uppercase">Sizes</div>
								<div class="font-mono text-foreground/90">{icon.sizes ?? '—'}</div>
							</div>
							<div class="bg-background px-3 py-2">
								<div class="mb-1 tracking-widest text-muted-foreground uppercase">Type</div>
								<div class="font-mono text-foreground/90">{icon.mime_type ?? '—'}</div>
							</div>
						</div>
					</div>
				{/each}
			</div>
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

	<div class="mt-px flex items-center justify-between border border-border bg-background px-6 py-3.5">
		<a href="/analyse/benchmark" class="text-[10px] tracking-widest text-muted-foreground uppercase transition-colors hover:text-foreground">
			← HTTP BENCHMARK
		</a>
		<a href="/" class="text-[10px] tracking-widest text-primary uppercase transition-colors hover:text-primary/80">
			ALL MODES →
		</a>
	</div>
</div>
