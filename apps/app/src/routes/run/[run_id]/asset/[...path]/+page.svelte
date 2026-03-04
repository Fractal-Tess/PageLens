<script lang="ts">
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { getRunAssets, assetUrl } from '$lib/api';
	import type { AnalysisAsset } from '$lib/api';

	const runId = $derived(String(page.params.run_id ?? ''));
	const filePath = $derived(String(page.params.path ?? ''));

	let asset = $state<AnalysisAsset | null>(null);
	let loading = $state(true);
	let error = $state('');
	let codeHtml = $state('');
	let rawCode = $state('');
	let htmlView = $state<'preview' | 'source'>('source');

	// Font injection
	$effect(() => {
		if (category === 'font' && rawUrl) {
			const style = document.createElement('style');
			style.textContent = `@font-face { font-family: "PagelensPreviewFont"; src: url("${rawUrl}"); }`;
			document.head.appendChild(style);
			return () => style.remove();
		}
	});

	function getCategory(a: AnalysisAsset | null, path: string): string {
		const ct = a?.content_type ?? '';
		const ext = path.split('.').pop()?.toLowerCase() ?? '';
		if (ct.startsWith('image/') || ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'ico', 'avif'].includes(ext))
			return 'image';
		if (ct.startsWith('video/') || ['mp4', 'webm', 'ogv', 'mov'].includes(ext)) return 'video';
		if (ct.startsWith('audio/') || ['mp3', 'wav', 'ogg', 'flac', 'm4a'].includes(ext)) return 'audio';
		if (
			ct.startsWith('font/') ||
			ct.includes('woff') ||
			['woff', 'woff2', 'ttf', 'eot', 'otf'].includes(ext)
		)
			return 'font';
		if (ct === 'text/html' || ext === 'html' || ext === 'htm') return 'html';
		if (
			ct.includes('javascript') ||
			ct === 'text/css' ||
			ct.includes('json') ||
			ct === 'application/xml' ||
			ct === 'image/svg+xml' ||
			['js', 'mjs', 'cjs', 'ts', 'tsx', 'jsx', 'css', 'json', 'jsonld', 'xml', 'svg'].includes(ext)
		)
			return 'code';
		return 'unknown';
	}

	function getLang(a: AnalysisAsset | null, path: string): string {
		const ct = a?.content_type ?? '';
		const ext = path.split('.').pop()?.toLowerCase() ?? '';
		if (ct.includes('javascript') || ['js', 'mjs', 'cjs'].includes(ext)) return 'javascript';
		if (ct === 'text/css' || ext === 'css') return 'css';
		if (ct.includes('json') || ext === 'json' || ext === 'jsonld') return 'json';
		if (ct === 'text/html' || ['html', 'htm'].includes(ext)) return 'html';
		if (['ts', 'tsx'].includes(ext)) return 'typescript';
		if (['jsx'].includes(ext)) return 'jsx';
		if (ext === 'xml' || ct === 'application/xml') return 'xml';
		if (ext === 'svg' || ct === 'image/svg+xml') return 'xml';
		return 'text';
	}

	function fmtSize(bytes: number): string {
		if (bytes === 0) return '—';
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
	}

	function typeBadgeClass(type: string): string {
		const map: Record<string, string> = {
			html: 'text-primary border-primary/60 bg-primary/5',
			favicon: 'text-yellow-400 border-yellow-400/50 bg-yellow-400/5',
			og_image: 'text-pink-400 border-pink-400/50 bg-pink-400/5',
			javascript: 'text-blue-400 border-blue-400/50 bg-blue-400/5',
			stylesheet: 'text-purple-400 border-purple-400/50 bg-purple-400/5',
			media: 'text-teal-400 border-teal-400/50 bg-teal-400/5',
			font: 'text-muted-foreground border-border bg-secondary/40'
		};
		return map[type] ?? 'text-muted-foreground border-border bg-secondary/40';
	}

	const rawUrl = $derived(assetUrl(runId, filePath));
	const category = $derived(getCategory(asset, filePath));

	onMount(async () => {
		try {
			const all = await getRunAssets(runId);
			asset = all.find((a) => a.local_path === filePath) ?? null;

			if (!asset) {
				error = 'Asset not found in run metadata';
				return;
			}
			if (asset.download_error) {
				error = `Download failed: ${asset.download_error}`;
				return;
			}

			const cat = getCategory(asset, filePath);
			if (cat === 'code' || cat === 'html') {
				const res = await fetch(rawUrl);
				if (!res.ok) throw new Error(`HTTP ${res.status}`);
				rawCode = await res.text();

				const lang = getLang(asset, filePath);
				if (lang !== 'text') {
					try {
						const { createHighlighter } = await import('shiki');
						const hl = await createHighlighter({
							themes: ['github-dark'],
							langs: [lang]
						});
						codeHtml = hl.codeToHtml(rawCode, {
							lang,
							theme: 'github-dark',
							colorReplacements: { '#0d1117': 'transparent' }
						});
						hl.dispose();
					} catch {
						// shiki failed — plain text fallback
					}
				}
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load asset';
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>{filePath} — PAGELENS</title>
</svelte:head>

<div class="mx-auto max-w-6xl px-6 py-10">
	<!-- Breadcrumb -->
	<div class="mb-6 flex items-center gap-2 font-mono text-xs text-muted-foreground">
		<a href="/" class="transition-colors hover:text-foreground">HOME</a>
		<span>/</span>
		<a href="/run/{runId}" class="transition-colors hover:text-foreground">
			RUN {runId.slice(0, 8).toUpperCase()}
		</a>
		<span>/</span>
		<span class="max-w-64 truncate text-foreground">{filePath}</span>
	</div>

	{#if loading}
		<div
			class="border border-border px-6 py-20 text-center text-xs tracking-widest text-muted-foreground"
		>
			LOADING...
		</div>
	{:else if error}
		<div class="border border-destructive/40 bg-destructive/10 px-4 py-3 text-xs text-destructive">
			ERROR: {error}
		</div>
	{:else if asset}
		<!-- Asset header -->
		<div class="mb-6 border border-border">
			<div class="flex items-start justify-between gap-4 border-b border-border p-6">
				<div class="min-w-0">
					<div class="mb-3 flex flex-wrap items-center gap-3">
						<span
							class="border px-2 py-0.5 text-xs font-bold uppercase tracking-widest {typeBadgeClass(asset.asset_type)}"
						>
							{asset.asset_type}
						</span>
						<span class="font-mono text-base font-bold">{filePath}</span>
					</div>
					<div class="font-mono text-xs text-muted-foreground break-all">
						{asset.original_url}
					</div>
				</div>
				<a
					href={rawUrl}
					target="_blank"
					rel="noopener noreferrer"
					class="shrink-0 border border-border px-3 py-1 text-xs font-bold tracking-widest uppercase text-muted-foreground transition-colors hover:border-primary hover:text-primary"
				>
					RAW ↗
				</a>
			</div>
			<div class="grid grid-cols-3 gap-px bg-border">
				<div class="bg-background px-4 py-3">
					<div class="mb-1 text-xs uppercase tracking-widest text-muted-foreground">Size</div>
					<div class="font-mono text-xl font-bold">{fmtSize(asset.file_size)}</div>
				</div>
				<div class="bg-background px-4 py-3">
					<div class="mb-1 text-xs uppercase tracking-widest text-muted-foreground">
						Content Type
					</div>
					<div class="font-mono text-sm">{asset.content_type ?? '—'}</div>
				</div>
				<div class="bg-background px-4 py-3">
					<div class="mb-1 text-xs uppercase tracking-widest text-muted-foreground">Cached At</div>
					<div class="font-mono text-sm">{new Date(asset.created_at).toLocaleString()}</div>
				</div>
			</div>
		</div>

		<!-- Content viewer -->
		{#if category === 'image'}
			<div class="border border-border">
				<div class="border-b border-border px-4 py-2">
					<span class="text-xs font-bold uppercase tracking-widest text-primary">Image Preview</span>
				</div>
				<!-- Checkerboard background for transparency -->
				<div
					class="flex min-h-64 items-center justify-center p-8"
					style="background-image: repeating-conic-gradient(oklch(0.14 0 0) 0% 25%, oklch(0.1 0 0) 0% 50%); background-size: 16px 16px;"
				>
					<img
						src={rawUrl}
						alt={asset.original_url}
						class="max-h-[60vh] max-w-full object-contain shadow-2xl"
					/>
				</div>
			</div>
		{:else if category === 'video'}
			<div class="border border-border">
				<div class="border-b border-border px-4 py-2">
					<span class="text-xs font-bold uppercase tracking-widest text-primary">Video</span>
				</div>
				<div class="flex items-center justify-center bg-black p-8">
					<video src={rawUrl} controls class="max-h-[60vh] max-w-full"></video>
				</div>
			</div>
		{:else if category === 'audio'}
			<div class="border border-border p-6">
				<div class="mb-3 text-xs uppercase tracking-widest text-muted-foreground">Audio</div>
				<!-- svelte-ignore a11y_media_has_caption -->
				<audio src={rawUrl} controls class="w-full"></audio>
			</div>
		{:else if category === 'font'}
			<div class="border border-border">
				<div class="border-b border-border px-4 py-2">
					<span class="text-xs font-bold uppercase tracking-widest text-primary">Font Preview</span>
				</div>
				<div class="space-y-6 p-8">
					<div style="font-family: 'PagelensPreviewFont', serif;" class="space-y-5">
						<div class="border-b border-border pb-5">
							<div class="mb-1 text-xs uppercase tracking-widest text-muted-foreground">72px</div>
							<div style="font-size: 72px; line-height: 1.1;">Aa</div>
						</div>
						<div class="border-b border-border pb-5">
							<div class="mb-1 text-xs uppercase tracking-widest text-muted-foreground">36px</div>
							<div style="font-size: 36px; line-height: 1.2;">
								The quick brown fox jumps over the lazy dog
							</div>
						</div>
						<div class="border-b border-border pb-5">
							<div class="mb-1 text-xs uppercase tracking-widest text-muted-foreground">24px</div>
							<div style="font-size: 24px; line-height: 1.3;">
								ABCDEFGHIJKLMNOPQRSTUVWXYZ<br />abcdefghijklmnopqrstuvwxyz
							</div>
						</div>
						<div>
							<div class="mb-1 text-xs uppercase tracking-widest text-muted-foreground">16px</div>
							<div style="font-size: 16px; line-height: 1.5;">
								0123456789 !@#$%^&amp;*()_+-=[]&lbrace;&rbrace;|;':",./{@html '&lt;'}&gt;?
							</div>
						</div>
					</div>
				</div>
			</div>
		{:else if category === 'html'}
			<div class="border border-border">
				<div class="flex items-center justify-between border-b border-border px-4 py-2">
					<span class="text-xs font-bold uppercase tracking-widest text-primary">HTML</span>
					<div class="flex gap-px bg-border">
						{#each [['preview', 'Preview'], ['source', 'Source']] as [id, label]}
							<button
								type="button"
								onclick={() => {
									htmlView = id as 'preview' | 'source';
								}}
								class="px-3 py-1 text-xs font-bold uppercase tracking-widest transition-colors {htmlView === id
									? 'bg-background text-primary'
									: 'bg-background/60 text-muted-foreground hover:text-foreground'}"
							>
								{label}
							</button>
						{/each}
					</div>
				</div>
				{#if htmlView === 'preview'}
					<iframe
						src={rawUrl}
						sandbox="allow-scripts allow-same-origin"
						title="HTML Preview"
						class="h-[70vh] w-full border-0"
					></iframe>
				{:else if codeHtml}
					<div class="code-viewer overflow-auto">
						{@html codeHtml}
					</div>
				{:else}
					<pre
						class="overflow-auto p-6 font-mono text-xs leading-relaxed text-muted-foreground">{rawCode}</pre>
				{/if}
			</div>
		{:else if category === 'code'}
			<div class="border border-border">
				<div class="border-b border-border px-4 py-2">
					<span class="text-xs font-bold uppercase tracking-widest text-primary">Source Code</span>
				</div>
				{#if codeHtml}
					<div class="code-viewer overflow-auto">
						{@html codeHtml}
					</div>
				{:else}
					<pre
						class="overflow-auto p-6 font-mono text-xs leading-relaxed text-muted-foreground">{rawCode}</pre>
				{/if}
			</div>
		{:else}
			<div class="border border-border px-6 py-16 text-center">
				<div class="mb-6 text-xs uppercase tracking-widest text-muted-foreground">
					Binary or unknown file type
				</div>
				<a
					href={rawUrl}
					download={filePath}
					class="border border-border px-5 py-2 text-xs font-bold uppercase tracking-widest text-muted-foreground transition-colors hover:border-primary hover:text-primary"
				>
					DOWNLOAD FILE ↓
				</a>
			</div>
		{/if}
	{/if}
</div>

<style>
	.code-viewer :global(pre.shiki) {
		margin: 0;
		padding: 1.5rem 0;
		background: transparent !important;
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 0.75rem;
		line-height: 1.7;
		overflow-x: auto;
		counter-reset: line;
	}

	.code-viewer :global(.line) {
		display: block;
		padding: 0 1.5rem;
		counter-increment: line;
	}

	.code-viewer :global(.line::before) {
		content: counter(line);
		display: inline-block;
		width: 3ch;
		margin-right: 2rem;
		text-align: right;
		color: oklch(0.32 0 0);
		user-select: none;
	}

	.code-viewer :global(.line:hover) {
		background: oklch(1 0 0 / 3%);
	}
</style>
