<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import {
		getRun, getRunPages, getRunAssets, parseRunPayload,
		startAnalysis, assetUrl, isCrawlRunPayload, isHttpBenchmarkRunPayload
	} from '$lib/api';
	import type { AnalysisAsset, AnalysisPageResult, RunPayload, SeoIssue } from '$lib/api';

	const runId = $derived.by(() => String(page.params.run_id ?? ''));
	const pageId = $derived.by(() => String(page.params.page_id ?? ''));

	let pg = $state<AnalysisPageResult | null>(null);
	let payload = $state<RunPayload | null>(null);
	let assets = $state<AnalysisAsset[]>([]);
	let error = $state('');
	let loading = $state(true);

	// Re-run state
	let rerunning = $state(false);

	async function rerun() {
		if (!pg || rerunning) return;
		rerunning = true;
		try {
			const result = await startAnalysis({ url: pg.url });
			await goto(`/run/${result.run_id}`);
		} catch (e) {
			rerunning = false;
		}
	}

	onMount(async () => {
		try {
			const [run, pages] = await Promise.all([getRun(runId), getRunPages(runId)]);
			pg = pages.find((p) => p.id === pageId) ?? null;
			if (run.payload_json) {
				payload = parseRunPayload(run.payload_json);
			}
			// Load assets in background — don't fail if not yet available
			getRunAssets(runId).then((a) => { assets = a; }).catch(() => {});
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load page data';
		} finally {
			loading = false;
		}
	});

	const pagePayload = $derived.by(() => {
		if (!payload) return null;
		if (isHttpBenchmarkRunPayload(payload)) return null;
		if (!isCrawlRunPayload(payload)) {
			return {
				snapshot: payload.snapshot,
				seo_report: payload.seo_report
			};
		}

		const currentUrl = pg?.url;
		const selectedPage = currentUrl
			? payload.pages.find((p) => p.url === currentUrl)
			: null;
		const fallbackPage = payload.pages[0] ?? null;
		const pageData = selectedPage ?? fallbackPage;
		if (!pageData) return null;

		return {
			snapshot: pageData.snapshot,
			seo_report: pageData.seo_report
		};
	});

	const pageSnapshot = $derived(pagePayload?.snapshot ?? null);
	const pageSeoReport = $derived(pagePayload?.seo_report ?? null);

	// Derive relative performance times from navigation_start
	const timing = $derived.by(() => {
		const t = pageSnapshot?.performance_timing;
		if (!t) return null;
		const ns = t.navigation_start;
		function rel(v: number | null | undefined) {
			if (v == null || v === 0) return null;
			const r = v - ns;
			return r > 0 ? r : null;
		}
		return {
			ttfb: rel(t.response_start),
			domInteractive: rel(t.dom_interactive),
			domLoaded: rel(t.dom_content_loaded),
			pageLoad: rel(t.load_complete),
			firstPaint: rel(t.first_paint),
			fcp: rel(t.first_contentful_paint),
			lcp: rel(t.largest_contentful_paint),
			cls: t.cumulative_layout_shift,
			inp: t.interaction_to_next_paint
		};
	});

	function fmtMs(n: number | null | undefined) {
		if (n == null) return '—';
		if (n < 1000) return `${Math.round(n)}ms`;
		return `${(n / 1000).toFixed(2)}s`;
	}

	function fmtCls(n: number | null | undefined) {
		if (n == null) return '—';
		return n.toFixed(3);
	}

	function scoreColor(score: number | null) {
		if (score == null) return 'text-muted-foreground';
		if (score >= 80) return 'text-green-400';
		if (score >= 50) return 'text-yellow-400';
		return 'text-destructive';
	}

	function severityColor(s: string) {
		if (s === 'error') return 'text-destructive';
		if (s === 'warning') return 'text-yellow-400';
		return 'text-muted-foreground';
	}

	function vitalsColor(label: string, val: number | null) {
		if (val == null) return 'text-muted-foreground';
		const thresholds: Record<string, [number, number]> = {
			ttfb: [800, 1800],
			fcp: [1800, 3000],
			lcp: [2500, 4000],
			pageLoad: [3000, 6000],
			inp: [200, 500]
		};
		const [good, poor] = thresholds[label] ?? [Infinity, Infinity];
		if (val <= good) return 'text-green-400';
		if (val <= poor) return 'text-yellow-400';
		return 'text-destructive';
	}

	function performanceMetricTooltip(key: string, value: number | null): string {
		const valueText = fmtMs(value);
		if (key === 'ttfb') {
			return `TTFB (Time to First Byte): server + network delay before first byte arrives. Current: ${valueText}.`;
		}
		if (key === 'fcp') {
			return `FCP (First Contentful Paint): when first visible content appears. Current: ${valueText}.`;
		}
		if (key === 'lcp') {
			return `LCP (Largest Contentful Paint): when the main visible content finishes rendering. Current: ${valueText}.`;
		}
		if (key === 'pageLoad') {
			return `Load: page load event completion time. Current: ${valueText}.`;
		}
		return valueText;
	}

	// Derived cached asset helpers
	const ogImageAsset = $derived(assets.find((a) => a.asset_type === 'og_image' && a.local_path && !a.download_error) ?? null);
	const faviconAsset = $derived(assets.find((a) => a.asset_type === 'favicon' && a.local_path && !a.download_error) ?? null);
	const htmlAsset = $derived(assets.find((a) => a.asset_type === 'html' && a.local_path && !a.download_error) ?? null);

	function fmtSize(bytes: number): string {
		if (bytes === 0) return '—';
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
	}

	function typeBadgeClass(type: string): string {
		const map: Record<string, string> = {
			html: 'text-primary border-primary/60',
			favicon: 'text-yellow-400 border-yellow-400/50',
			og_image: 'text-pink-400 border-pink-400/50',
			javascript: 'text-blue-400 border-blue-400/50',
			stylesheet: 'text-purple-400 border-purple-400/50',
			media: 'text-teal-400 border-teal-400/50',
			font: 'text-muted-foreground border-border'
		};
		return map[type] ?? 'text-muted-foreground border-border';
	}

	function assetLookupKeys(url: string): string[] {
		const keys = new Set<string>();
		keys.add(url);

		try {
			const parsed = new URL(url);
			parsed.hash = '';
			keys.add(parsed.toString());
			keys.add(parsed.pathname + parsed.search);
			keys.add(parsed.pathname);

			const withoutQuery = new URL(parsed.toString());
			withoutQuery.search = '';
			keys.add(withoutQuery.toString());
			keys.add(withoutQuery.pathname);
		} catch {
			const noHash = url.split('#')[0] ?? url;
			keys.add(noHash);
			keys.add(noHash.split('?')[0] ?? noHash);
		}

		return [...keys].filter((key) => key.length > 0);
	}

	const assetByOriginalUrl = $derived.by(() => {
		const map = new Map<string, AnalysisAsset>();
		for (const asset of assets) {
			for (const key of assetLookupKeys(asset.original_url)) {
				if (!map.has(key)) map.set(key, asset);
			}
		}
		return map;
	});

	function getReferencedAsset(itemUrl: string): AnalysisAsset | null {
		for (const key of assetLookupKeys(itemUrl)) {
			const match = assetByOriginalUrl.get(key);
			if (match) return match;
		}
		return null;
	}

	function assetViewerHref(localPath: string): string {
		const encodedPath = localPath
			.split('/')
			.filter((part) => part.length > 0)
			.map((part) => encodeURIComponent(part))
			.join('/');
		return `/run/${runId}/asset/${encodedPath}`;
	}

	function extractDiscoveredLinksCount(html: string | null | undefined, baseUrl: string | null | undefined): number {
		if (!html) return 0;
		const links = new Set<string>();
		const hrefRegex = /<a\b[^>]*\bhref\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/gi;

		for (const match of html.matchAll(hrefRegex)) {
			const rawHref = (match[1] ?? match[2] ?? match[3] ?? '').trim();
			if (!rawHref || rawHref.startsWith('#') || rawHref.toLowerCase().startsWith('javascript:')) {
				continue;
			}

			if (rawHref.startsWith('data:')) {
				links.add(rawHref);
				continue;
			}

			try {
				if (baseUrl) {
					links.add(new URL(rawHref, baseUrl).toString());
				} else {
					links.add(new URL(rawHref).toString());
				}
			} catch {
				// Skip invalid URLs
			}
		}

		return links.size;
	}

	const discoveredLinksCount = $derived.by(() => {
		const fromHtml = extractDiscoveredLinksCount(pageSnapshot?.html ?? null, pg?.url ?? null);
		return Math.max(pg?.links_found_count ?? 0, fromHtml);
	});

	// Group issues by category
	const issuesByCategory = $derived.by(() => {
		const issues = pageSeoReport?.issues ?? [];
		const map = new Map<string, SeoIssue[]>();
		for (const issue of issues) {
			const cat = issue.category ?? 'other';
			if (!map.has(cat)) map.set(cat, []);
			map.get(cat)!.push(issue);
		}
		return map;
	});

	type AnalysisTab = 'overview' | 'performance' | 'seo' | 'html' | 'assets';

	const TAB_LABELS: Record<AnalysisTab, string> = {
		overview: 'Overview',
		performance: 'Performance',
		seo: 'SEO',
		html: 'HTML',
		assets: 'Assets'
	};

	const TAB_ISSUE_CATEGORIES: Record<Exclude<AnalysisTab, 'overview'>, Set<string>> = {
		performance: new Set(['performance']),
		seo: new Set(['seo', 'meta', 'canonical', 'hreflang', 'headings', 'links', 'images', 'favicon', 'accessibility']),
		html: new Set(['meta', 'canonical', 'hreflang', 'headings', 'links', 'accessibility', 'seo']),
		assets: new Set(['images', 'favicon', 'performance'])
	};

	function issuesForTab(tab: AnalysisTab): SeoIssue[] {
		const issues = pageSeoReport?.issues ?? [];
		if (tab === 'overview') return issues;
		const allowedCategories = TAB_ISSUE_CATEGORIES[tab];
		return issues.filter((issue: SeoIssue) => allowedCategories.has(issue.category ?? 'other'));
	}

	const tabIssues = $derived.by(() => issuesForTab(activeTab));

	const tabIssuesByCategory = $derived.by(() => {
		const map = new Map<string, SeoIssue[]>();
		for (const issue of tabIssues) {
			const cat = issue.category ?? 'other';
			if (!map.has(cat)) map.set(cat, []);
			map.get(cat)!.push(issue);
		}
		return map;
	});

	let activeTab = $state<AnalysisTab>('overview');

	const availableTabs = $derived.by(() => {
		const tabs: Array<{ id: AnalysisTab; label: string }> = [];
		tabs.push({ id: 'overview', label: TAB_LABELS.overview });

		if (timing || pageSnapshot?.main_resource_network) {
			tabs.push({ id: 'performance', label: TAB_LABELS.performance });
		}

		if (pageSeoReport) {
			tabs.push({ id: 'seo', label: TAB_LABELS.seo });
		}

		if (pageSnapshot || htmlAsset || pageSeoReport) {
			tabs.push({ id: 'html', label: TAB_LABELS.html });
		}

		if (pageSnapshot?.referenced_assets || assets.length > 0) {
			tabs.push({ id: 'assets', label: TAB_LABELS.assets });
		}

		return tabs;
	});

	$effect(() => {
		if (availableTabs.length === 0) return;
		if (!availableTabs.some((tab) => tab.id === activeTab)) {
			activeTab = availableTabs[0].id;
		}
	});
</script>

<svelte:head>
	<title>PAGE DETAIL — PAGELENS</title>
</svelte:head>

<div class="mx-auto max-w-6xl px-6 py-10">
	<!-- Breadcrumb -->
	<div class="mb-6 flex items-center gap-2 font-mono text-xs text-muted-foreground">
		<a href="/" class="hover:text-foreground transition-colors">HOME</a>
		<span>/</span>
		<a href="/run/{runId}" class="hover:text-foreground transition-colors">
			RUN {runId.slice(0, 8).toUpperCase()}
		</a>
		<span>/</span>
		<span class="text-foreground">PAGE</span>
	</div>

	{#if loading}
		<div class="border border-border px-6 py-20 text-center text-xs tracking-widest text-muted-foreground">
			LOADING...
		</div>
	{:else if error}
		<div class="border border-destructive/40 bg-destructive/10 px-4 py-3 text-xs text-destructive">
			ERROR: {error}
		</div>
	{:else if !pg}
		<div class="border border-border px-6 py-20 text-center text-xs text-muted-foreground">
			Page not found.
		</div>
	{:else}
		<!-- Page header -->
		<div class="mb-6 border border-border p-6">
			<div class="mb-4 flex items-start justify-between gap-4">
				<div class="min-w-0">
					<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Analysed URL</div>
					<div class="flex items-center gap-2">
						{#if faviconAsset}
							<img
								src={assetUrl(runId, faviconAsset.local_path)}
								alt="favicon"
								class="h-4 w-4 shrink-0 object-contain"
								onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
							/>
						{:else if pageSeoReport?.favicon_url}
							<img
								src={pageSeoReport.favicon_url}
								alt="favicon"
								class="h-4 w-4 shrink-0 object-contain"
								onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
							/>
						{/if}
						<div class="font-mono text-sm break-all">{pg.url}</div>
					</div>
					{#if pageSnapshot?.title}
						<div class="mt-1 text-xs text-muted-foreground">{pageSnapshot.title}</div>
					{/if}
				</div>
				<div class="flex shrink-0 items-center gap-2">
					<button
						onclick={rerun}
						disabled={rerunning}
						class="flex items-center gap-1.5 border border-border px-3 py-1 text-xs font-bold tracking-widest uppercase text-muted-foreground hover:border-primary hover:text-primary transition-colors disabled:opacity-40"
					>
						{#if rerunning}
							<svg class="h-3 w-3 animate-spin" viewBox="0 0 24 24" fill="none">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
							</svg>
							RUNNING
						{:else}
							↻ RE-RUN
						{/if}
					</button>
					<div
						class="border px-3 py-1 text-xs font-bold tracking-widest uppercase {pg.success
							? 'border-green-700 text-green-400'
							: 'border-destructive/60 text-destructive'}"
					>
						{pg.success ? 'SUCCESS' : 'FAILED'}
					</div>
				</div>
			</div>

			<!-- Key metrics -->
			<div class="grid grid-cols-3 gap-px bg-border md:grid-cols-6">
				{#each [
					{ label: 'SEO Score', value: pg.seo_score != null ? Math.round(pg.seo_score).toString() : '—', color: scoreColor(pg.seo_score) },
					{ label: 'Issues', value: pg.total_issues.toString(), color: pg.total_issues > 0 ? 'text-destructive' : 'text-green-400' },
					{ label: 'Errors', value: pg.error_count.toString(), color: pg.error_count > 0 ? 'text-destructive' : 'text-muted-foreground' },
					{ label: 'Warnings', value: pg.warning_count.toString(), color: pg.warning_count > 0 ? 'text-yellow-400' : 'text-muted-foreground' },
					{ label: 'Links', value: discoveredLinksCount.toString(), color: 'text-foreground' },
					{ label: 'Depth', value: pg.depth.toString(), color: 'text-foreground' }
				] as stat}
					<div class="bg-background px-4 py-3">
						<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">{stat.label}</div>
						<div class="font-mono text-2xl font-bold {stat.color}">{stat.value}</div>
					</div>
				{/each}
			</div>
		</div>

		{#if availableTabs.length > 0}
			<div class="mb-6 border border-border">
				<div class="flex flex-wrap gap-px bg-border">
					{#each availableTabs as tab}
						<button
							type="button"
							onclick={() => { activeTab = tab.id; }}
							class="border-0 px-4 py-2 text-xs font-bold tracking-widest uppercase transition-colors {activeTab === tab.id
								? 'bg-background text-primary'
								: 'bg-background/70 text-muted-foreground hover:text-foreground'}"
						>
							{tab.label}
						</button>
					{/each}
				</div>
			</div>

			<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
				{#if tabIssues.length > 0}
					<div class="border border-border lg:col-span-2">
						<div class="border-b border-border px-4 py-2">
							<span class="text-xs font-bold tracking-widest uppercase text-destructive">
								{activeTab === 'overview' ? 'All Issues' : `${TAB_LABELS[activeTab]} Issues`} ({tabIssues.length})
							</span>
						</div>
						<div class="divide-y divide-border">
							{#each [...tabIssuesByCategory.entries()] as [cat, catIssues]}
								<div class="p-4">
									<div class="mb-2 text-xs font-bold tracking-widest uppercase text-muted-foreground">
										{cat} ({catIssues.length})
									</div>
									<div class="space-y-1.5">
										{#each catIssues as issue}
											<div class="flex min-w-0 items-start gap-3 font-mono text-xs">
												<span class="shrink-0 font-bold {severityColor(issue.severity)}">
													[{issue.severity.toUpperCase()}]
												</span>
												<span class="min-w-0 break-words">{issue.message}</span>
											</div>
										{/each}
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				{#if activeTab === 'overview'}
					<div class="border border-border lg:col-span-2">
						<div class="border-b border-border px-4 py-2">
							<span class="text-xs font-bold tracking-widest uppercase text-primary">Overview</span>
						</div>
						<div class="grid grid-cols-1 gap-px bg-border md:grid-cols-3">
							<div class="bg-background px-4 py-3">
								<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Total Issues</div>
								<div class="font-mono text-xl font-bold {tabIssues.length > 0 ? 'text-destructive' : 'text-green-400'}">{tabIssues.length}</div>
							</div>
							<div class="bg-background px-4 py-3">
								<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Errors</div>
								<div class="font-mono text-xl font-bold text-destructive">
									{tabIssues.filter((issue) => issue.severity === 'error').length}
								</div>
							</div>
							<div class="bg-background px-4 py-3">
								<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Warnings</div>
								<div class="font-mono text-xl font-bold text-yellow-400">
									{tabIssues.filter((issue) => issue.severity === 'warning').length}
								</div>
							</div>
						</div>
					</div>
				{:else if activeTab === 'performance'}
					<!-- Performance Vitals -->
					{#if timing}
						<div class="border border-border">
							<div class="border-b border-border px-4 py-2">
								<span class="text-xs font-bold tracking-widest uppercase text-primary">Performance</span>
							</div>
							<div class="p-4">
								<div class="mb-4 grid grid-cols-2 gap-px bg-border">
									{#each [
										{ key: 'ttfb', label: 'TTFB', value: timing.ttfb },
										{ key: 'fcp', label: 'FCP', value: timing.fcp },
										{ key: 'lcp', label: 'LCP', value: timing.lcp },
										{ key: 'pageLoad', label: 'Load', value: timing.pageLoad }
									] as v}
										<div
											class="bg-background px-3 py-2"
											title={performanceMetricTooltip(v.key, v.value)}
										>
											<div class="mb-0.5 text-xs text-muted-foreground uppercase tracking-wider">{v.label}</div>
											<div class="font-mono text-lg font-bold {vitalsColor(v.key, v.value)}">
												{fmtMs(v.value)}
											</div>
										</div>
									{/each}
								</div>
								<div class="space-y-1 font-mono text-xs">
									{#each [
										{ label: 'DOM Interactive', value: fmtMs(timing.domInteractive) },
										{ label: 'DOM Content Loaded', value: fmtMs(timing.domLoaded) },
										{ label: 'First Paint', value: fmtMs(timing.firstPaint) },
										{ label: 'CLS', value: fmtCls(timing.cls) },
										{ label: 'INP', value: fmtMs(timing.inp) }
									] as row}
										<div class="flex justify-between">
											<span class="text-muted-foreground">{row.label}</span>
											<span>{row.value}</span>
										</div>
									{/each}
								</div>
							</div>
						</div>
					{/if}

					<!-- Network -->
					{#if pageSnapshot?.main_resource_network}
						{@const net = pageSnapshot.main_resource_network}
						<div class="border border-border">
							<div class="border-b border-border px-4 py-2">
								<span class="text-xs font-bold tracking-widest uppercase text-primary">Network</span>
							</div>
							<div class="p-4">
								<div class="mb-4 grid grid-cols-2 gap-px bg-border">
									<div class="bg-background px-3 py-2">
										<div class="mb-0.5 text-xs text-muted-foreground uppercase tracking-wider">Status</div>
										<div class="font-mono text-lg font-bold {net.status_code && net.status_code < 400 ? 'text-green-400' : 'text-destructive'}">
											{net.status_code ?? '—'}
										</div>
									</div>
									<div class="bg-background px-3 py-2">
										<div class="mb-0.5 text-xs text-muted-foreground uppercase tracking-wider">Requests</div>
										<div class="font-mono text-lg font-bold">
											{pageSnapshot.network_requests.length}
										</div>
									</div>
								</div>
								{#if net.final_url && net.final_url !== pg.url}
									<div class="mb-3 font-mono text-xs">
										<div class="mb-0.5 text-muted-foreground uppercase tracking-wider">Final URL</div>
										<div class="break-all text-primary">{net.final_url}</div>
									</div>
								{/if}
								{#if Object.keys(net.headers).length > 0}
									<div class="mb-2 text-xs font-bold tracking-widest text-muted-foreground uppercase">Headers</div>
									<div class="space-y-1 font-mono text-xs">
										{#each Object.entries(net.headers) as [k, v]}
											<div class="flex gap-2">
												<span class="w-40 shrink-0 truncate text-muted-foreground">{k}</span>
												<span class="break-all text-xs">{v}</span>
											</div>
										{/each}
									</div>
								{/if}
								{#if net.fetch_error}
									<div class="mt-2 font-mono text-xs text-destructive">Error: {net.fetch_error}</div>
								{/if}
							</div>
						</div>
					{/if}
				{:else if activeTab === 'seo'}
					{#if pageSeoReport}
						{@const seo = pageSeoReport}
						<!-- Headings structure -->
						{#if pageSeoReport?.headings}
							{@const h = pageSeoReport.headings}
							<div class="border border-border">
								<div class="border-b border-border px-4 py-2">
									<span class="text-xs font-bold tracking-widest uppercase text-primary">Headings</span>
								</div>
								<div class="p-4">
									<div class="mb-4 grid grid-cols-6 gap-px bg-border">
										{#each [
											{ tag: 'h1', count: h.h1_count },
											{ tag: 'h2', count: h.h2_count },
											{ tag: 'h3', count: h.h3_count },
											{ tag: 'h4', count: h.h4_count },
											{ tag: 'h5', count: h.h5_count },
											{ tag: 'h6', count: h.h6_count }
										] as { tag, count }}
											<div class="bg-background px-3 py-2 text-center">
												<div class="mb-0.5 text-xs text-muted-foreground uppercase tracking-wider">{tag.toUpperCase()}</div>
												<div class="font-mono text-lg font-bold {count > 0 ? '' : 'text-muted-foreground'}">
													{count}
												</div>
											</div>
										{/each}
									</div>
									{#if h.structure.length > 0}
										<div class="space-y-0.5 font-mono text-xs">
											{#each h.structure.slice(0, 20) as tag}
												<div class="text-muted-foreground" style="padding-left: {(parseInt(tag.replace('h', '')) - 1) * 12}px">
													<span class="text-primary">{tag}</span>
												</div>
											{/each}
											{#if h.structure.length > 20}
												<div class="text-muted-foreground/50">+{h.structure.length - 20} more</div>
											{/if}
										</div>
									{/if}
								</div>
							</div>
						{/if}

						<!-- Open Graph + Twitter -->
						<div class="border border-border">
							<div class="border-b border-border px-4 py-2">
								<span class="text-xs font-bold tracking-widest uppercase text-primary">Social Tags</span>
							</div>
							<div class="p-4">
								{#if ogImageAsset}
									<div class="mb-4">
										<div class="mb-2 text-xs tracking-wider text-muted-foreground uppercase">OG Image Preview</div>
										<img
											src={assetUrl(runId, ogImageAsset.local_path)}
											alt="og preview"
											class="max-h-48 max-w-full border border-border object-contain"
											onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
										/>
										<div class="mt-1 font-mono text-xs text-muted-foreground">
											{ogImageAsset.content_type ?? ''} · {(ogImageAsset.file_size / 1024).toFixed(1)} KB
										</div>
									</div>
								{:else if seo.open_graph.image}
									<div class="mb-4">
										<div class="mb-2 text-xs tracking-wider text-muted-foreground uppercase">OG Image</div>
										<img
											src={seo.open_graph.image}
											alt="og preview"
											class="max-h-48 max-w-full border border-border object-contain"
											onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
										/>
									</div>
								{/if}
								<div class="mb-3 text-xs tracking-wider text-muted-foreground uppercase">Open Graph</div>
								<div class="mb-4 space-y-1 font-mono text-xs">
									{#each Object.entries(seo.open_graph) as [k, v]}
										<div class="flex gap-3">
											<span class="w-24 shrink-0 text-muted-foreground">{k.replace('og_', 'og:')}</span>
											<span class="break-all">{v ?? '—'}</span>
										</div>
									{/each}
								</div>
								<div class="mb-3 text-xs tracking-wider text-muted-foreground uppercase">Twitter Card</div>
								<div class="space-y-1 font-mono text-xs">
									{#each Object.entries(seo.twitter_card) as [k, v]}
										<div class="flex gap-3">
											<span class="w-24 shrink-0 text-muted-foreground">{k}</span>
											<span class="break-all">{v ?? '—'}</span>
										</div>
									{/each}
								</div>
							</div>
						</div>

						<!-- SEO Metadata -->
						<div class="border border-border">
							<div class="border-b border-border px-4 py-2">
								<span class="text-xs font-bold tracking-widest uppercase text-primary">SEO Metadata</span>
							</div>
							<div class="space-y-1 p-4 font-mono text-xs">
								{#each [
									{ label: 'title', value: seo.meta.title },
									{ label: 'description', value: seo.meta.description },
									{ label: 'language', value: seo.meta.language },
									{ label: 'robots', value: seo.meta.robots },
									{ label: 'canonical', value: seo.canonical_url },
									{ label: 'charset', value: seo.meta.charset ? 'present' : 'missing' },
									{ label: 'viewport', value: seo.meta.viewport ? 'present' : 'missing' }
								] as row}
									<div class="flex gap-3">
										<span class="w-24 shrink-0 text-muted-foreground">{row.label}</span>
										<span class="{row.value === 'missing' ? 'text-destructive' : row.value === 'present' ? 'text-green-400' : ''} break-all">
											{row.value ?? '—'}
										</span>
									</div>
								{/each}
							</div>
						</div>

						{#if (pageSeoReport?.images?.length ?? 0) > 0}
							{@const imgs = pageSeoReport.images}
							<div class="border border-border lg:col-span-2">
								<div class="flex items-center justify-between border-b border-border px-4 py-2">
									<span class="text-xs font-bold tracking-widest uppercase text-primary">
										Images ({imgs.length})
									</span>
									<span class="text-xs text-muted-foreground">
										{imgs.filter((i: { has_alt: boolean }) => i.has_alt).length}/{imgs.length} with alt
									</span>
								</div>
								<div class="grid grid-cols-[24px_1fr_200px] gap-3 border-b border-border bg-secondary/40 px-4 py-2 text-xs font-bold tracking-widest text-muted-foreground uppercase">
									<div>Alt</div>
									<div>URL</div>
									<div>Alt Text</div>
								</div>
								<div class="max-h-64 divide-y divide-border overflow-y-auto">
									{#each imgs as img}
										<div class="grid grid-cols-[24px_1fr_200px] items-center gap-3 px-4 py-2 font-mono text-xs">
											<div class="font-bold {img.has_alt ? 'text-green-400' : 'text-destructive'}">
												{img.has_alt ? '✓' : '✗'}
											</div>
											<div class="truncate text-muted-foreground">{img.src}</div>
											<div class="truncate {img.alt ? '' : 'text-muted-foreground/40 italic'}">
												{img.alt ?? 'missing'}
											</div>
										</div>
									{/each}
								</div>
							</div>
						{/if}
					{/if}
				{:else if activeTab === 'html'}
					<div class="border border-border lg:col-span-2">
						<div class="flex items-center justify-between border-b border-border px-4 py-2">
							<span class="text-xs font-bold tracking-widest uppercase text-primary">HTML Snapshot</span>
							{#if htmlAsset}
								<a
									href={assetUrl(runId, htmlAsset.local_path)}
									target="_blank"
									rel="noopener noreferrer"
									class="border border-border px-2 py-1 text-xs font-bold tracking-widest uppercase text-muted-foreground transition-colors hover:border-primary hover:text-primary"
								>
									VIEW SAVED HTML ↗
								</a>
							{/if}
						</div>
						<div class="grid grid-cols-1 gap-px bg-border md:grid-cols-3">
							<div class="bg-background px-4 py-3">
								<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Saved HTML</div>
								<div class="font-mono text-sm {htmlAsset ? 'text-green-400' : 'text-muted-foreground'}">
									{htmlAsset ? 'available' : 'not cached'}
								</div>
							</div>
							<div class="bg-background px-4 py-3">
								<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Title</div>
								<div class="font-mono text-sm break-all">{pageSnapshot?.title ?? '—'}</div>
							</div>
							<div class="bg-background px-4 py-3">
								<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Language</div>
								<div class="font-mono text-sm">{pageSeoReport?.meta.language ?? '—'}</div>
							</div>
						</div>
						<div class="space-y-2 p-4 font-mono text-xs">
							<div class="flex justify-between gap-4">
								<span class="text-muted-foreground">Charset</span>
								<span class={pageSeoReport?.meta.charset ? 'text-green-400' : 'text-destructive'}>
									{pageSeoReport?.meta.charset ? 'present' : 'missing'}
								</span>
							</div>
							<div class="flex justify-between gap-4">
								<span class="text-muted-foreground">Viewport</span>
								<span class={pageSeoReport?.meta.viewport ? 'text-green-400' : 'text-destructive'}>
									{pageSeoReport?.meta.viewport ? 'present' : 'missing'}
								</span>
							</div>
							<div class="flex justify-between gap-4">
								<span class="text-muted-foreground">Canonical URL</span>
								<span class="min-w-0 break-all text-right">{pageSeoReport?.canonical_url ?? '—'}</span>
							</div>
						</div>
					</div>
				{:else if activeTab === 'assets'}
					{#if pageSnapshot?.referenced_assets}
						{@const refAssets = pageSnapshot.referenced_assets}
						<div class="lg:col-span-2">
							<div class="mb-2 text-xs font-bold tracking-widest uppercase text-primary">Referenced Assets</div>
						</div>
						{#each [
							{ label: 'JavaScript', items: refAssets.javascript },
							{ label: 'Stylesheets', items: refAssets.stylesheets },
							{ label: 'Media', items: refAssets.media },
							{ label: 'Fonts', items: refAssets.fonts ?? [] }
						] as group}
							<div class="border border-border">
								<div class="border-b border-border px-4 py-2">
									<span class="text-xs font-bold tracking-widest uppercase text-muted-foreground">
										{group.label} ({group.items.length})
									</span>
								</div>
								<div class="max-h-48 space-y-1 overflow-y-auto p-4 font-mono text-xs">
									{#each group.items as item}
										{@const linkedAsset = getReferencedAsset(item)}
										<div class="flex items-center gap-3">
											{#if linkedAsset?.local_path && !linkedAsset.download_error}
												<a
													href={assetViewerHref(linkedAsset.local_path)}
													class="min-w-0 flex-1 truncate text-primary hover:underline"
													title={item}
												>
													{item.replace(/^https?:\/\/[^/]+/, '')}
												</a>
											{:else}
												<a
													href={item}
													target="_blank"
													rel="noopener noreferrer"
													class="min-w-0 flex-1 truncate text-muted-foreground transition-colors hover:text-foreground"
													title={item}
												>
													{item.replace(/^https?:\/\/[^/]+/, '')}
												</a>
											{/if}
											<span class="shrink-0 text-muted-foreground">
												{linkedAsset ? fmtSize(linkedAsset.file_size) : '—'}
											</span>
										</div>
									{/each}
									{#if group.items.length === 0}
										<div class="text-muted-foreground/40">—</div>
									{/if}
								</div>
							</div>
						{/each}
					{/if}

					{#if assets.length > 0}
						<div class="border border-border lg:col-span-2">
							<div class="flex items-center justify-between border-b border-border px-4 py-2">
								<span class="text-xs font-bold tracking-widest uppercase text-primary">Cached Assets</span>
								{#if htmlAsset}
									<a
										href={assetUrl(runId, htmlAsset.local_path)}
										target="_blank"
										rel="noopener noreferrer"
										class="border border-border px-2 py-1 text-xs font-bold tracking-widest uppercase text-muted-foreground transition-colors hover:border-primary hover:text-primary"
									>
										VIEW SAVED HTML ↗
									</a>
								{/if}
							</div>
							<div class="max-h-64 divide-y divide-border overflow-y-auto">
								{#each assets as asset}
									<div class="flex items-center gap-3 px-4 py-2 font-mono text-xs">
										<span class="w-20 shrink-0 text-muted-foreground uppercase">{asset.asset_type}</span>
										{#if asset.local_path && !asset.download_error}
											<a
												href={assetViewerHref(asset.local_path)}
												class="min-w-0 flex-1 truncate text-primary hover:underline"
												title={asset.original_url}
											>
												{asset.original_url.replace(/^https?:\/\/[^/]+/, '') || asset.original_url}
											</a>
											<span class="shrink-0 text-muted-foreground">{fmtSize(asset.file_size)}</span>
										{:else}
											<span class="min-w-0 flex-1 truncate text-muted-foreground/50" title={asset.original_url}>
												{asset.original_url.replace(/^https?:\/\/[^/]+/, '') || asset.original_url}
											</span>
											<span class="shrink-0 text-xs text-destructive/70">failed</span>
										{/if}
									</div>
								{/each}
							</div>
						</div>
					{/if}
				{/if}
			</div>
		{:else}
			<div class="border border-border px-6 py-12 text-center text-xs text-muted-foreground uppercase tracking-widest">
				No grouped analysis data available for this page.
			</div>
		{/if}
	{/if}
</div>
