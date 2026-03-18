<script lang="ts">
	import { goto } from '$app/navigation';
	import { startAnalysis, type FaviconAnalysisResult } from '$lib/api';
	import { Button } from '@pagelens/ui/shadcn/button';
	import { Input } from '@pagelens/ui/shadcn/input';
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from '@pagelens/ui/shadcn/table';
	import { z } from 'zod';

	let url = $state('');
	let loading = $state(false);
	let error = $state('');
	let result = $state<FaviconAnalysisResult | null>(null);
	let phase = $state<'idle' | 'starting' | 'error'>('idle');
	let copiedSnippet = $state('');

	const urlSchema = z.string().trim().url().startsWith('http');

	function isValidUrl(str: string): boolean {
		return urlSchema.safeParse(str).success;
	}

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!url.trim()) return;
		error = '';
		loading = true;
		phase = 'starting';
		try {
			const { run_id } = await startAnalysis({
				url: url.trim(),
				analysis_type: 'favicon'
			});
			await goto(`/favicon/analyse/${run_id}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Unknown error';
			loading = false;
			phase = 'error';
		}
	}

	function formatBytes(bytes: number | null): string {
		if (bytes == null) return '—';
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
	}

	function contrastLabel(ratio: number | null): string {
		if (ratio == null) return 'Unknown';
		if (ratio < 2.2) return 'Low';
		if (ratio < 3) return 'Fair';
		return 'Good';
	}

	function contrastTone(ratio: number | null): string {
		if (ratio == null) return 'text-muted-foreground';
		if (ratio < 2.2) return 'text-destructive';
		if (ratio < 3) return 'text-yellow-300';
		return 'text-green-400';
	}

	function severityTone(severity: string): string {
		if (severity === 'critical') return 'text-red-400';
		if (severity === 'high') return 'text-destructive';
		if (severity === 'medium') return 'text-yellow-300';
		if (severity === 'low') return 'text-sky-300';
		return 'text-muted-foreground';
	}

	function statusTone(status: string): string {
		if (status === 'fail') return 'text-destructive';
		if (status === 'warn') return 'text-yellow-300';
		if (status === 'pass') return 'text-green-400';
		if (status === 'not_applicable') return 'text-muted-foreground';
		return 'text-foreground/80';
	}

	function compactUrl(value: string): string {
		try {
			const parsed = new URL(value);
			return `${parsed.hostname}${parsed.pathname}`;
		} catch {
			return value;
		}
	}

	function contrastLines(theme: string, lightContrast: string, darkContrast: string): string[] {
		if (theme === 'light') return [`Light: ${lightContrast}`];
		if (theme === 'dark') return [`Dark: ${darkContrast}`];
		if (theme === 'light+dark') return [`L: ${lightContrast}`, `D: ${darkContrast}`];
		return [`L: ${lightContrast}`, `D: ${darkContrast}`];
	}

	function previewModes(theme: string): Array<'light' | 'dark'> {
		if (theme === 'light') return ['light'];
		if (theme === 'dark') return ['dark'];
		if (theme === 'light+dark') return ['light', 'dark'];
		return ['light', 'dark'];
	}

	async function copySnippet(key: string, text: string): Promise<void> {
		try {
			await navigator.clipboard.writeText(text);
			copiedSnippet = key;
			setTimeout(() => {
				if (copiedSnippet === key) copiedSnippet = '';
			}, 1200);
		} catch {
			copiedSnippet = '';
		}
	}

	const reportV2 = $derived(result?.report_v2 ?? null);
	type FaviconCheck = FaviconAnalysisResult['report_v2']['checks'][number];
	const severityOrder = ['critical', 'high', 'medium', 'low', 'info'];
	const groupedChecks = $derived.by(() => {
		if (!reportV2) return [] as Array<{ severity: string; checks: FaviconCheck[] }>;
		const issueStatuses = new Set<FaviconCheck['status']>(['warn', 'fail']);
		return severityOrder
			.map((severity) => ({
				severity,
				checks: reportV2.checks.filter(
					(check) => check.severity === severity && issueStatuses.has(check.status)
				)
			}))
			.filter((entry) => entry.checks.length > 0);
	});
	const overviewStats = $derived.by(() => {
		if (!reportV2)
			return [] as Array<{ label: string; value: string; tone: 'neutral' | 'warn' | 'critical' | 'good' }>;

		const failingChecks = reportV2.checks.filter((check) => check.status === 'fail').length;
		const warningChecks = reportV2.checks.filter((check) => check.status === 'warn').length;

		return [
			{ label: 'Favicon assets', value: String(reportV2.inventory.length), tone: 'good' as const },
			{ label: 'Failing checks', value: String(failingChecks), tone: failingChecks > 0 ? ('critical' as const) : ('good' as const) },
			{ label: 'Warnings', value: String(warningChecks), tone: warningChecks > 0 ? ('warn' as const) : ('neutral' as const) },
			{
				label: 'Recommendations',
				value: String(reportV2.recommendations.length),
				tone: reportV2.recommendations.length > 0 ? ('warn' as const) : ('good' as const)
			}
		];
	});

	const perIconRows = $derived.by(() => {
		if (!result)
			return [] as Array<{
				url: string;
				source: string;
				theme: string;
				media: string;
				format: string;
				fileSize: string;
				lightContrastValue: number | null;
				darkContrastValue: number | null;
				lightContrast: string;
				darkContrast: string;
				issues: number;
				recommendations: number;
				issueMessages: string[];
				recommendationMessages: string[];
			}>;

		const rows = result.candidate_reports.map((report) => {
			const theme = report.preferred_theme ?? 'none';
			const media = report.media ?? '—';
			return {
				url: report.url,
				source: report.source,
				theme,
				media,
				format: (report.format ?? '—').toUpperCase(),
				fileSize: formatBytes(report.file_size_bytes),
				lightContrastValue: report.contrast_on_light,
				darkContrastValue: report.contrast_on_dark,
				lightContrast: `${report.contrast_on_light != null ? report.contrast_on_light.toFixed(2) : '—'} (${contrastLabel(report.contrast_on_light)})`,
				darkContrast: `${report.contrast_on_dark != null ? report.contrast_on_dark.toFixed(2) : '—'} (${contrastLabel(report.contrast_on_dark)})`,
				issues: report.issues.length,
				recommendations: report.recommendations.length,
				issueMessages: report.issues,
				recommendationMessages: report.recommendations
			};
		});

		return rows;
	});
</script>

<svelte:head>
	<title>PAGELENS — Favicon Analyzer</title>
</svelte:head>

<div class="mx-auto max-w-6xl px-6 py-8">
	<div class="animate-fade-in mb-6 flex items-center gap-2 text-[10px] tracking-widest text-muted-foreground uppercase">
		<a href="/" class="transition-colors hover:text-foreground">PAGELENS</a>
		<span class="text-border">/</span>
		<span class="text-primary">FAVICON ANALYZER</span>
	</div>

	<div class="animate-fade-in-up stagger-1 mb-8 grid grid-cols-1 gap-px bg-border md:grid-cols-[2fr_1fr]">
		<div class="bg-background p-5 md:p-6">
			<div class="mb-2 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">04 · Analysis mode</div>
			<h1 class="text-3xl font-black leading-none tracking-tighter md:text-4xl">
				FAVICON <span class="text-primary">ANALYZER</span>
			</h1>
			<p class="mt-3 max-w-sm text-xs leading-relaxed text-muted-foreground">
				Extract and review all favicon candidates for a URL: HTML-declared icons, sizes,
				MIME hints, and the default <span class="font-mono text-foreground/80">/favicon.ico</span> fallback.
			</p>
		</div>
		<div class="bg-background p-5 md:p-6">
			<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Checks</div>
			<div class="space-y-1.5">
				{#each ['HTML icon declarations', 'Resolved absolute icon URLs', 'Declared sizes and MIME types', 'Default /favicon.ico fallback', 'Duplicate candidate suppression'] as item}
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

		{#if reportV2}
			<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Overview</div>
			<div class="mb-6 grid grid-cols-1 gap-px bg-border md:grid-cols-3">
				<div class="bg-background p-5">
					<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Score</div>
					<div class="font-mono text-2xl {reportV2.score.score_0_100 >= 80 ? 'text-green-400' : reportV2.score.score_0_100 >= 60 ? 'text-yellow-300' : 'text-destructive'}">
						{reportV2.score.score_0_100} / 100 · {reportV2.score.grade}
					</div>
				</div>
				<div class="bg-background p-5 md:col-span-2">
					<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Stats</div>
					<div class="grid grid-cols-2 gap-2 md:grid-cols-4">
						{#each overviewStats as stat}
							<div class="border border-border bg-secondary/20 p-3">
								<div class="text-[10px] uppercase tracking-wide text-muted-foreground">{stat.label}</div>
								<div class="font-mono text-xl {stat.tone === 'critical' ? 'text-destructive' : stat.tone === 'warn' ? 'text-yellow-300' : stat.tone === 'good' ? 'text-green-400' : 'text-foreground'}">
									{stat.value}
								</div>
							</div>
						{/each}
					</div>
				</div>
			</div>

			<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Issues by severity</div>
			<div class="mb-6 border border-border bg-background p-0">
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead class="w-28">Severity</TableHead>
							<TableHead class="w-24">Status</TableHead>
							<TableHead class="w-44">Rule</TableHead>
							<TableHead>Details</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#if groupedChecks.length > 0}
							{#each groupedChecks as bucket}
								{#each bucket.checks as check}
									<TableRow>
										<TableCell class="font-mono text-xs uppercase {severityTone(check.severity)}">{check.severity}</TableCell>
										<TableCell class="font-mono text-xs uppercase {statusTone(check.status)}">{check.status}</TableCell>
										<TableCell class="font-mono text-[11px] text-muted-foreground">{check.rule_id}</TableCell>
										<TableCell class="text-xs text-muted-foreground">{check.details}</TableCell>
									</TableRow>
								{/each}
							{/each}
						{:else}
							<TableRow>
								<TableCell colspan={4} class="p-4 text-xs text-green-400">No failing or warning checks found.</TableCell>
							</TableRow>
						{/if}
					</TableBody>
				</Table>
			</div>

			<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Suggested fixes</div>
			<div class="mb-6 border border-border bg-background p-0">
				{#if reportV2.recommendations.length > 0}
					<div class="divide-y divide-border">
						{#each reportV2.recommendations as recommendation}
							<div class="p-5">
								<div class="mb-2 flex items-center justify-between gap-3">
									<div class="font-mono text-xs uppercase {severityTone(recommendation.priority)}">{recommendation.priority} · {recommendation.title}</div>
									<div class="font-mono text-[10px] text-muted-foreground">{recommendation.recommendation_id}</div>
								</div>
								<div class="mb-2 space-y-1 text-[11px] text-muted-foreground">
									{#each recommendation.why.slice(0, 2) as reason}
										<div>• {reason}</div>
									{/each}
								</div>
								{#if recommendation.snippets.length > 0}
									{#each recommendation.snippets as snippet, index}
										<div class="mb-2 mt-2 border border-border bg-secondary/20">
											<div class="flex items-center justify-between border-b border-border px-3 py-2">
												<div class="font-mono text-[10px] uppercase text-muted-foreground">{snippet.language}</div>
												<Button
													type="button"
													class="h-7 rounded-none border border-border bg-transparent px-2 text-[10px] font-mono text-muted-foreground hover:text-foreground"
													onclick={() => copySnippet(`${recommendation.recommendation_id}-${index}`, snippet.content)}
												>
													{copiedSnippet === `${recommendation.recommendation_id}-${index}` ? 'COPIED' : 'COPY'}
												</Button>
											</div>
											<pre class="overflow-x-auto p-3 font-mono text-[11px] leading-relaxed text-muted-foreground">{snippet.content}</pre>
										</div>
									{/each}
								{/if}
							</div>
						{/each}
					</div>
				{:else}
					<div class="p-4 text-xs text-green-400">No suggested fixes. Current configuration looks healthy.</div>
				{/if}
			</div>
		{/if}

		<div class="mt-6 mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Favicons</div>
		<div class="border border-border bg-background p-0">
			{#if perIconRows.length > 0}
				<Table class="w-full table-fixed">
					<TableHeader>
							<TableRow>
								<TableHead class="w-72">Icon preview</TableHead>
							<TableHead class="w-28">Source</TableHead>
							<TableHead class="w-20">Theme</TableHead>
							<TableHead class="w-20">Format</TableHead>
							<TableHead class="w-24">Size</TableHead>
							<TableHead class="w-40">Contrast</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#each perIconRows as row}
							<TableRow>
								<TableCell class="align-top">
									<div class="min-w-0 space-y-1.5">
										<div class="flex items-center gap-2">
											{#each previewModes(row.theme) as mode}
												<div class="flex h-8 w-8 items-center justify-center border border-border p-1 {mode === 'light' ? 'bg-white' : 'bg-zinc-900'} {row.theme === mode ? 'ring-1 ring-primary' : ''}">
													<img src={row.url} alt={mode === 'light' ? 'favicon on light background' : 'favicon on dark background'} class="h-full w-full object-contain" loading="lazy" />
												</div>
											{/each}
											<span class="font-mono text-[10px] text-muted-foreground">{row.theme === 'none' ? 'L / D' : row.theme.toUpperCase()}</span>
										</div>
										<div class="font-mono text-[9px] tracking-wide text-muted-foreground uppercase">{row.issues} issues · {row.recommendations} recommendations</div>
										<div class="max-w-[220px] truncate font-mono text-[10px] text-muted-foreground">{compactUrl(row.url)}</div>
										<div class="max-w-[220px] truncate font-mono text-[10px] text-muted-foreground">{row.media}</div>

										{#if row.issueMessages.length > 0 || row.recommendationMessages.length > 0}
											<div class="mt-1 space-y-1 border-l border-border pl-2">
												{#if row.issueMessages.length > 0}
													{#each row.issueMessages as issue}
														<div class="max-w-full break-words text-[10px] leading-tight text-destructive/90">Issue: {issue}</div>
													{/each}
												{/if}
												{#if row.recommendationMessages.length > 0}
													{#each row.recommendationMessages as recommendation}
														<div class="max-w-full break-words text-[10px] leading-tight text-muted-foreground">Recommendation: {recommendation}</div>
													{/each}
												{/if}
											</div>
										{/if}
									</div>
								</TableCell>
								<TableCell class="font-mono text-[11px] uppercase">{row.source}</TableCell>
								<TableCell class="font-mono text-xs uppercase {row.theme === 'dark' ? 'text-indigo-300' : row.theme === 'light' ? 'text-yellow-200' : 'text-muted-foreground'}">{row.theme}</TableCell>
								<TableCell class="font-mono text-xs">{row.format}</TableCell>
								<TableCell class="font-mono text-xs">{row.fileSize}</TableCell>
								<TableCell class="font-mono text-[11px] text-muted-foreground">
									{#each contrastLines(row.theme, row.lightContrast, row.darkContrast) as line}
										{#if line.startsWith('Light:') || line.startsWith('L:')}
											<div class={contrastTone(row.lightContrastValue)}>{line}</div>
										{:else}
											<div class={contrastTone(row.darkContrastValue)}>{line}</div>
										{/if}
									{/each}
								</TableCell>
							</TableRow>
						{/each}
					</TableBody>
				</Table>
			{:else}
				<div class="p-4 text-xs text-muted-foreground">No per-icon findings available.</div>
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
