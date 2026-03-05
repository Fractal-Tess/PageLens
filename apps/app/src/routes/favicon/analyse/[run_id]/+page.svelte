<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { onDestroy } from 'svelte';
	import {
		getRun,
		isFaviconRunPayload,
		parseRunPayload,
		startAnalysis,
		type AnalysisRun,
		type FaviconAnalysisResult,
		type RunPayload
	} from '$lib/api';
	import { Button } from '@pagelens/ui/shadcn/button';
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from '@pagelens/ui/shadcn/table';

	const runId = $derived(page.params.run_id ?? '');

	let loading = $state(true);
	let error = $state('');
	let run = $state<AnalysisRun | null>(null);
	let payload = $state<RunPayload | null>(null);
	let result = $state<FaviconAnalysisResult | null>(null);
	let copiedSnippet = $state('');
	let pollTimer: ReturnType<typeof setInterval> | null = null;
	let rerunning = $state(false);

	function clearPoll(): void {
		if (pollTimer) {
			clearInterval(pollTimer);
			pollTimer = null;
		}
	}

	onDestroy(() => {
		clearPoll();
	});

	$effect(() => {
		const targetRunId = runId;
		if (!targetRunId) return;
		clearPoll();
		void loadRun(targetRunId);
	});

	async function loadRun(targetRunId = runId): Promise<void> {
		loading = true;
		error = '';
		try {
			const current = await getRun(targetRunId);
			if (targetRunId !== runId) return;
			run = current;
			payload = current.payload_json ? parseRunPayload(current.payload_json) : null;

			if (current.analysis_type !== 'favicon') {
				error = `Run ${targetRunId} is not a favicon analysis run.`;
				result = null;
				clearPoll();
				return;
			}

			if (payload && isFaviconRunPayload(payload)) {
				result = payload;
			} else {
				result = null;
			}

			if (current.status === 'pending' || current.status === 'running') {
				if (!pollTimer) {
					pollTimer = setInterval(() => {
						void loadRun(targetRunId);
					}, 1200);
				}
			} else {
				clearPoll();
				if (!result && current.status === 'completed') {
					error = 'Run completed but payload is not a favicon report.';
				}
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load run';
		} finally {
			loading = false;
		}
	}

	async function rerun(): Promise<void> {
		if (!run || rerunning) return;
		rerunning = true;
		error = '';
		try {
			const { run_id } = await startAnalysis({
				url: run.url,
				analysis_type: 'favicon'
			});
			result = null;
			payload = null;
			loading = true;
			await goto(`/favicon/analyse/${run_id}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to re-run favicon analysis';
		} finally {
			rerunning = false;
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

	function isReachabilityMessage(value: string | null | undefined): boolean {
		if (!value) return false;
		const normalized = value.toLowerCase();
		return (
			normalized.includes('not reachable') ||
			normalized.includes('timed out') ||
			normalized.includes('connection failed') ||
			normalized.includes('returned http') ||
			normalized.includes('failed to fetch')
		);
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

	const issueAndFixRows = $derived.by(() => {
		if (!reportV2)
			return [] as Array<{ severity: string; status: string; details: string }>;

		const recommendationById = new Map(reportV2.recommendations.map((item) => [item.recommendation_id, item]));
		const checkRows = reportV2.checks
			.filter((check) => check.status === 'warn' || check.status === 'fail')
			.map((check) => {
				const recommendationTitles = check.recommendation_ids
					.map((id) => recommendationById.get(id)?.title)
					.filter((value): value is string => Boolean(value))
					.join(' | ');
				return {
					severity: check.severity,
					status: check.status,
					details: recommendationTitles.length > 0
						? `${check.details} | ${recommendationTitles}`
						: check.details
				};
			});

		const standaloneRecommendations = reportV2.recommendations
			.filter((recommendation) => recommendation.related_check_ids.length === 0)
			.map((recommendation) => ({
				severity: recommendation.priority,
				status: 'suggested',
				details: [recommendation.title, recommendation.why.slice(0, 2).join(' | ')]
					.filter((value) => value.length > 0)
					.join(' | ')
			}));

		return [...checkRows, ...standaloneRecommendations];
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

		return result.candidate_reports.map((report) => {
			const theme = report.preferred_theme ?? 'none';
			return {
				url: report.url,
				source: report.source,
				theme,
				media: report.media ?? '—',
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
	});

	const unreachableResourceCount = $derived.by(() => {
		if (!result) return 0;
		return result.candidate_reports.filter((report) =>
			report.issues.some((issue) => isReachabilityMessage(issue))
		).length;
	});

	const reachabilityContextMessage = $derived.by(() => {
		const candidates = [
			error,
			run?.current_message ?? '',
			...(result?.warnings ?? []),
			...((result?.candidate_reports ?? []).flatMap((report) => report.issues))
		];
		return candidates.find((value) => isReachabilityMessage(value)) ?? null;
	});
</script>

<svelte:head>
	<title>PAGELENS — Favicon Run {runId}</title>
</svelte:head>

<div class="mx-auto max-w-6xl px-6 py-16">
	<div class="mb-12 flex items-center gap-2 text-[10px] tracking-widest text-muted-foreground uppercase">
		<a href="/" class="transition-colors hover:text-foreground">PAGELENS</a>
		<span class="text-border">/</span>
		<a href="/analyse/favicon" class="transition-colors hover:text-foreground">FAVICON ANALYZER</a>
		<span class="text-border">/</span>
		<span class="text-primary">RUN {runId.slice(0, 8)}</span>
	</div>

	{#if error}
		<div class="mb-4 border border-destructive/40 bg-destructive/10 px-4 py-3 text-xs text-destructive">ERROR: {error}</div>
	{/if}

	{#if reachabilityContextMessage}
		<div class="mb-4 border border-yellow-500/40 bg-yellow-500/10 px-4 py-3 text-xs text-yellow-200">
			<div class="mb-1 text-[10px] tracking-widest text-yellow-300 uppercase">Resource reachability</div>
			<div>{reachabilityContextMessage}</div>
			<div class="mt-1 text-yellow-200/80">Some checks may be incomplete because one or more favicon resources could not be reached.</div>
		</div>
	{/if}

	{#if loading}
		<div class="border border-border bg-background p-8 text-xs text-muted-foreground">Loading run...</div>
	{:else if run && (run.status === 'pending' || run.status === 'running')}
		<div class="border border-border bg-background p-8 text-xs text-muted-foreground">
			<div class="mb-2 font-mono uppercase">Run is in progress</div>
			<div>{run.current_message ?? 'Analyzing favicon assets...'}</div>
		</div>
	{:else if result}
		<div class="mb-6 flex items-center justify-between border border-border bg-background px-6 py-4">
			<div>
				<div class="text-[10px] tracking-widest text-muted-foreground uppercase">Input URL</div>
				<div class="font-mono text-xs">{result.input_url}</div>
			</div>
			<Button type="button" class="h-9 rounded-none border border-primary bg-transparent px-4 text-[10px] tracking-widest uppercase text-primary hover:bg-primary/10 disabled:opacity-40" onclick={rerun} disabled={rerunning}>{rerunning ? 'RE-RUNNING…' : 'RE-RUN'}</Button>
		</div>

		{#if unreachableResourceCount > 0}
			<div class="mb-4 border border-yellow-500/40 bg-yellow-500/10 px-4 py-3 text-xs text-yellow-200">
				Detected unreachable icon resources: {unreachableResourceCount}. Review the per-icon issue list below for exact URLs and failure reasons.
			</div>
		{/if}

		{#if reportV2}
			<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Overview</div>
			<div class="mb-6 grid grid-cols-1 gap-px bg-border md:grid-cols-3">
				<div class="bg-background p-5">
					<div class="mb-1 text-[10px] tracking-widest text-muted-foreground uppercase">Score</div>
					<div class="font-mono text-2xl {reportV2.score.score_0_100 >= 80 ? 'text-green-400' : reportV2.score.score_0_100 >= 60 ? 'text-yellow-300' : 'text-destructive'}">{reportV2.score.score_0_100} / 100 · {reportV2.score.grade}</div>
				</div>
				<div class="bg-background p-5 md:col-span-2">
					<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Stats</div>
					<div class="grid grid-cols-2 gap-2 md:grid-cols-4">
						{#each overviewStats as stat}
							<div class="border border-border bg-secondary/20 p-3">
								<div class="text-[10px] uppercase tracking-wide text-muted-foreground">{stat.label}</div>
								<div class="font-mono text-xl {stat.tone === 'critical' ? 'text-destructive' : stat.tone === 'warn' ? 'text-yellow-300' : stat.tone === 'good' ? 'text-green-400' : 'text-foreground'}">{stat.value}</div>
							</div>
						{/each}
					</div>
				</div>
			</div>

			<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Issues and suggested fixes</div>
			<div class="mb-6 border border-border bg-background p-0">
				<Table class="w-full table-fixed">
					<TableHeader>
						<TableRow>
							<TableHead class="w-28">Severity</TableHead>
							<TableHead class="w-24">Status</TableHead>
							<TableHead class="w-[40rem]">Details</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#if issueAndFixRows.length > 0}
							{#each issueAndFixRows as row}
								<TableRow>
									<TableCell class="font-mono text-xs uppercase {severityTone(row.severity)}">{row.severity}</TableCell>
									<TableCell class="font-mono text-xs uppercase {statusTone(row.status)}">{row.status}</TableCell>
									<TableCell class="max-w-0 whitespace-normal break-words text-xs leading-relaxed text-muted-foreground">{row.details}</TableCell>
								</TableRow>
							{/each}
						{:else}
							<TableRow>
								<TableCell colspan={3} class="p-4 text-xs text-green-400">No failing or warning checks found.</TableCell>
							</TableRow>
						{/if}
					</TableBody>
				</Table>
			</div>
		{/if}

		<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Favicons</div>
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
	{:else if run?.status === 'failed'}
		<div class="border border-yellow-500/40 bg-yellow-500/10 p-8 text-xs text-yellow-200">
			<div class="mb-2 text-[10px] tracking-widest text-yellow-300 uppercase">Analysis failed</div>
			<div class="mb-4">{run.current_message ?? 'The favicon analysis did not complete successfully.'}</div>
			<Button type="button" class="h-9 rounded-none border border-yellow-300/70 bg-transparent px-4 text-[10px] tracking-widest uppercase text-yellow-300 hover:bg-yellow-500/10" onclick={rerun} disabled={rerunning}>{rerunning ? 'RE-RUNNING…' : 'TRY AGAIN'}</Button>
		</div>
	{:else}
		<div class="border border-border bg-background p-8 text-xs text-muted-foreground">No favicon payload available for this run.</div>
	{/if}

	<div class="mt-px flex items-center justify-between border border-border bg-background px-6 py-3.5">
		<a href="/analyse/favicon" class="text-[10px] tracking-widest text-muted-foreground uppercase transition-colors hover:text-foreground">← NEW FAVICON ANALYSIS</a>
		<a href="/history" class="text-[10px] tracking-widest text-primary uppercase transition-colors hover:text-primary/80">HISTORY →</a>
	</div>
</div>
