<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { onDestroy, untrack } from 'svelte';
	import {
		cancelRun,
		getRun,
		getRunPages,
		isHttpBenchmarkRunPayload,
		parseRunPayload,
		subscribeToEvents,
		startAnalysis
	} from '$lib/api';
	import type { AnalysisRun, AnalysisPageResult, RunEvent, RunPayload } from '$lib/api';

	const runId = $derived(page.params.run_id ?? '');

	let run = $state<AnalysisRun | null>(null);
	let runPayload = $state<RunPayload | null>(null);
	let pages = $state<AnalysisPageResult[]>([]);
	let error = $state('');
	let done = $state(false);
	let cancelling = $state(false);
	let es = $state<EventSource | null>(null);
	const initialLiveStats = {
		discovered: 0,
		queued: 0,
		running: 0,
		scanned: 0,
		success: 0,
		failed: 0,
		linksFoundTotal: 0
	};
	let liveStats = $state({ ...initialLiveStats });
	let benchmarkLiveStartedAt = $state<number | null>(null);
	const initialBenchmarkLive = {
		method: 'GET',
		connections: 0,
		targetDurationSecs: 0,
		elapsedMs: 0,
		successful: 0,
		failed: 0,
		requestsPerSec: 0,
		successRate: 0,
		latencyMinMs: 0,
		latencyAvgMs: 0,
		latencyP95Ms: 0,
		latencyMaxMs: 0,
		totalDataBytes: 0,
		avgSizePerRequestBytes: 0,
		dataPerSecBytes: 0,
		histogram: [] as Array<[number, number]>,
		p10: 0,
		p25: 0,
		p50: 0,
		p75: 0,
		p90: 0,
		p99: 0,
		p99_9: 0
	};
	let benchmarkLive = $state({ ...initialBenchmarkLive });
	let loadVersion = 0;

	// Re-run
	let rerunningUrl = $state<string | null>(null);
	let rerunningRun = $state(false);

	async function rerunPage(url: string) {
		if (rerunningUrl) return;
		rerunningUrl = url;
		try {
			const result = await startAnalysis({
				url,
				analysis_type: run?.analysis_type
			});
			await goto(`/run/${result.run_id}`);
		} catch {
			rerunningUrl = null;
		}
	}

	async function rerunCurrentRun() {
		if (!run || rerunningRun) return;
		rerunningRun = true;
		try {
			const benchmarkOptions =
				run.analysis_type === 'http_benchmark'
					? {
						duration_secs: benchmarkPayload?.target_duration_secs
							?? (benchmarkPayload ? (benchmarkPayload.duration_ms / 1000) : undefined),
						connections: benchmarkPayload?.connections,
						method: benchmarkPayload?.method
					}
					: undefined;

			const result = await startAnalysis({
				url: run.url,
				analysis_type: run.analysis_type,
				benchmark_options: benchmarkOptions
			});
			await goto(`/run/${result.run_id}`);
		} catch {
			/* noop */
		} finally {
			rerunningRun = false;
		}
	}

	function updateLiveStatsFromPages() {
		const success = pages.filter((entry) => entry.success).length;
		const failed = pages.length - success;
		const linksFoundTotal = pages.reduce((total, entry) => total + entry.links_found_count, 0);
		liveStats = {
			discovered: pages.length,
			queued: 0,
			running: run?.status === 'running' ? 1 : 0,
			scanned: pages.length,
			success,
			failed,
			linksFoundTotal
		};
	}

	async function fetchRun(targetRunId = runId, version = loadVersion) {
		try {
			const nextRun = await getRun(targetRunId);
			if (targetRunId !== runId || version !== loadVersion) return;
			if (
				run &&
				done &&
				(run.status === 'completed' || run.status === 'failed') &&
				(nextRun.status === 'pending' || nextRun.status === 'running')
			) {
				return;
			}
			run = nextRun;
			runPayload = run.payload_json ? parseRunPayload(run.payload_json) : null;
			if (run.analysis_type === 'http_benchmark' && runPayload && isHttpBenchmarkRunPayload(runPayload)) {
				benchmarkLive = {
					...benchmarkLive,
					method: runPayload.method,
					connections: runPayload.connections,
					targetDurationSecs: runPayload.target_duration_secs ?? benchmarkLive.targetDurationSecs,
					elapsedMs: runPayload.duration_ms,
					successful: runPayload.successful_requests,
					failed: runPayload.failed_requests,
					requestsPerSec: runPayload.requests_per_sec,
					successRate:
						runPayload.success_rate != null
							? runPayload.success_rate * 100
							: runPayload.requests > 0
								? (runPayload.successful_requests / runPayload.requests) * 100
								: 0,
					latencyMinMs: runPayload.latency.min_ms,
					latencyAvgMs: runPayload.latency.avg_ms,
					latencyP95Ms: runPayload.latency.p95_ms,
					latencyMaxMs: runPayload.latency.max_ms,
					totalDataBytes: runPayload.total_data_bytes ?? 0,
					avgSizePerRequestBytes: runPayload.avg_size_per_request_bytes ?? 0,
					dataPerSecBytes: runPayload.data_per_sec_bytes ?? 0,
					histogram: runPayload.latency_histogram,
					p10: runPayload.latency.p10_ms ?? 0,
					p25: runPayload.latency.p25_ms ?? 0,
					p50: runPayload.latency.p50_ms,
					p75: runPayload.latency.p75_ms ?? 0,
					p90: runPayload.latency.p90_ms ?? 0,
					p99: runPayload.latency.p99_ms,
					p99_9: runPayload.latency.p99_9_ms ?? 0
				};
			}
			if (run.analysis_type === 'http_benchmark' && benchmarkLiveStartedAt == null) {
				const startedAtMs = run.created_at ? Date.parse(run.created_at) : Number.NaN;
				if (Number.isFinite(startedAtMs)) {
					benchmarkLiveStartedAt = startedAtMs;
				}
			}
			done = !(run.status === 'pending' || run.status === 'running');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load run';
		}
	}

	async function fetchPages(targetRunId = runId, version = loadVersion) {
		try {
			const nextPages = await getRunPages(targetRunId);
			if (targetRunId !== runId || version !== loadVersion) return;
			pages = nextPages;
			updateLiveStatsFromPages();
		} catch {
			/* ignore */
		}
	}

	async function refreshRunData(targetRunId = runId, version = loadVersion) {
		await Promise.all([fetchRun(targetRunId, version), fetchPages(targetRunId, version)]);
	}

	function resetRunViewState() {
		error = '';
		run = null;
		runPayload = null;
		pages = [];
		done = false;
		cancelling = false;
		rerunningUrl = null;
		benchmarkLiveStartedAt = null;
		liveStats = { ...initialLiveStats };
		benchmarkLive = { ...initialBenchmarkLive };
	}

	function closeEventStream() {
		es?.close();
		es = null;
	}

	function connectSSE(targetRunId = runId, version = loadVersion) {
		closeEventStream();
		es = subscribeToEvents(
			targetRunId,
			async (event: RunEvent) => {
				if (targetRunId !== runId || version !== loadVersion) {
					closeEventStream();
					return;
				}
				if (event.kind === 'progress') {
					if (run && event.progress != null) {
						run = { ...run, progress: event.progress };
					}
					liveStats = {
						discovered: event.discovered_count ?? event.page_count ?? liveStats.discovered,
						queued: event.queued_count ?? liveStats.queued,
						running: event.running_count ?? liveStats.running,
						scanned: event.scanned_count ?? liveStats.scanned,
						success: event.success_count ?? liveStats.success,
						failed: event.failed_count ?? liveStats.failed,
						linksFoundTotal: event.links_found_total ?? liveStats.linksFoundTotal
					};

					if (run?.analysis_type === 'http_benchmark') {
						if (benchmarkLiveStartedAt == null) {
							benchmarkLiveStartedAt = Date.now();
						}
						const elapsedMs = Math.max(1, Date.now() - benchmarkLiveStartedAt);
						const successful = event.success_count ?? benchmarkLive.successful;
						const failed = event.failed_count ?? benchmarkLive.failed;
						const doneCount = successful + failed;
						const elapsedSec = elapsedMs / 1000;
						benchmarkLive = {
							method: event.live_method ?? benchmarkLive.method,
							connections: event.live_connections ?? benchmarkLive.connections,
							targetDurationSecs: event.live_target_duration_secs ?? benchmarkLive.targetDurationSecs,
							elapsedMs,
							successful,
							failed,
							requestsPerSec: elapsedSec > 0 ? doneCount / elapsedSec : 0,
							successRate: doneCount > 0 ? (successful / doneCount) * 100 : 0,
							latencyMinMs: event.live_latency_min_ms ?? benchmarkLive.latencyMinMs,
							latencyAvgMs: event.live_latency_avg_ms ?? benchmarkLive.latencyAvgMs,
							latencyP95Ms: event.live_latency_p95_ms ?? benchmarkLive.latencyP95Ms,
							latencyMaxMs: event.live_latency_max_ms ?? benchmarkLive.latencyMaxMs,
							totalDataBytes: event.live_total_data_bytes ?? benchmarkLive.totalDataBytes,
							avgSizePerRequestBytes: event.live_avg_size_per_request_bytes ?? benchmarkLive.avgSizePerRequestBytes,
							dataPerSecBytes: event.live_data_per_sec_bytes ?? benchmarkLive.dataPerSecBytes,
							histogram: event.live_histogram ?? benchmarkLive.histogram,
							p10: event.live_latency_p10_ms ?? benchmarkLive.p10,
							p25: event.live_latency_p25_ms ?? benchmarkLive.p25,
							p50: event.live_latency_p50_ms ?? benchmarkLive.p50,
							p75: event.live_latency_p75_ms ?? benchmarkLive.p75,
							p90: event.live_latency_p90_ms ?? benchmarkLive.p90,
							p99: event.live_latency_p99_ms ?? benchmarkLive.p99,
							p99_9: event.live_latency_p99_9_ms ?? benchmarkLive.p99_9
						};
					}
				}

				if (event.kind === 'complete' || event.kind === 'failed' || event.kind === 'cancelled') {
					if (run) {
						if (event.kind === 'complete') {
							run = {
								...run,
								status: 'completed',
								current_stage: 'complete',
								current_message: event.message ?? run.current_message,
								progress: 1.0
							};
						} else if (event.kind === 'cancelled') {
							run = {
								...run,
								status: 'failed',
								current_stage: 'cancelled',
								current_message:
									event.message ??
									run.current_message ??
									'Analysis was interrupted by user before completion.',
								progress: 1.0
							};
						} else {
							run = {
								...run,
								status: 'failed',
								current_stage: event.stage ?? run.current_stage,
								current_message: event.message ?? run.current_message,
								progress: event.progress ?? run.progress
							};
						}
					}
					done = true;
					benchmarkLiveStartedAt = null;
					closeEventStream();
					await refreshRunData();
				}
			},
			async () => {
				if (targetRunId !== runId || version !== loadVersion) {
					closeEventStream();
					return;
				}
				if (done) {
					closeEventStream();
					return;
				}
				await fetchRun(targetRunId, version);
				if (run && (run.status === 'pending' || run.status === 'running')) {
					return;
				}
				closeEventStream();
			}
		);
	}

	async function stopRun() {
		if (!run || cancelling) return;
		if (!(run.status === 'running' || run.status === 'pending')) return;
		cancelling = true;
		try {
			await cancelRun(runId);
			closeEventStream();
			await refreshRunData();
			if (run && (run.status === 'running' || run.status === 'pending')) {
				connectSSE(runId, loadVersion);
			}
		} catch {
			/* ignore */
		} finally {
			cancelling = false;
		}
	}

	$effect(() => {
		const targetRunId = runId;
		if (!targetRunId) {
			untrack(() => {
				closeEventStream();
			});
			return;
		}

		untrack(() => {
			const version = ++loadVersion;
			closeEventStream();
			resetRunViewState();

			void (async () => {
				await refreshRunData(targetRunId, version);
				if (version !== loadVersion) return;
				if (run && (run.status === 'pending' || run.status === 'running')) {
					connectSSE(targetRunId, version);
				}
			})();
		});
	});

	onDestroy(() => {
		closeEventStream();
	});

	function isInterruptedByUser(r?: AnalysisRun | null) {
		if (!r || r.status !== 'failed') return false;
		if (r.current_stage === 'cancelled') return true;
		const message = r.current_message?.toLowerCase() ?? '';
		return (
			message.includes('cancelled by user') ||
			message.includes('canceled by user') ||
			message.includes('interrupted by user')
		);
	}

	function isCancelledRun(r?: AnalysisRun | null) {
		return isInterruptedByUser(r);
	}

	function isCancelPending(r?: AnalysisRun | null) {
		if (!r) return false;
		if (r.current_stage === 'cancelling') return true;
		const message = r.current_message?.toLowerCase() ?? '';
		return message.includes('stopping benchmark') || message.includes('cancelling');
	}

	function statusLabel(r?: AnalysisRun | null) {
		if (!r?.status) return 'LOADING';
		if (isCancelPending(r)) return 'CANCELED';
		if (isCancelledRun(r)) return 'CANCELED';
		return r.status.toUpperCase();
	}

	function statusClass(r?: AnalysisRun | null) {
		if (r?.status === 'completed') return 'border-green-700 text-green-400';
		if (isCancelPending(r)) return 'border-yellow-600/70 text-yellow-400';
		if (isCancelledRun(r)) return 'border-yellow-600/70 text-yellow-400';
		if (r?.status === 'failed') return 'border-destructive/60 text-destructive';
		return 'border-primary/60 text-primary';
	}

	function effectiveProgress(r?: AnalysisRun | null) {
		if (!r) return null;
		if (isCancelledRun(r) || r.status === 'completed') return 1;
		return r.progress ?? null;
	}

	function scoreColor(score: number | null) {
		if (score == null) return 'text-muted-foreground';
		if (score >= 80) return 'text-green-400';
		if (score >= 50) return 'text-yellow-400';
		return 'text-destructive';
	}

	const benchmarkPayload = $derived(
		runPayload && isHttpBenchmarkRunPayload(runPayload) ? runPayload : null
	);

	// Benchmark display helpers
	function fmtMs(ms: number): string {
		if (ms < 1) return `${(ms * 1000).toFixed(0)}μs`;
		if (ms < 1000) return `${ms.toFixed(2)}ms`;
		return `${(ms / 1000).toFixed(3)}s`;
	}

	function fmtDuration(ms: number): string {
		if (ms < 1000) return `${ms}ms`;
		return `${(ms / 1000).toFixed(3)}s`;
	}

	function fmtBytes(bytes: number): string {
		if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
		const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
		let value = bytes;
		let idx = 0;
		while (value >= 1024 && idx < units.length - 1) {
			value /= 1024;
			idx += 1;
		}
		return `${value.toFixed(value >= 10 || idx === 0 ? 1 : 2)} ${units[idx]}`;
	}

	function latencyBarColor(ms: number): string {
		if (ms <= 100) return 'bg-green-500/70';
		if (ms <= 400) return 'bg-yellow-400/70';
		return 'bg-destructive/70';
	}

	function latencyTextColor(ms: number): string {
		if (ms <= 100) return 'text-green-400';
		if (ms <= 400) return 'text-yellow-400';
		return 'text-destructive';
	}

	function statusBgColor(code: string): string {
		const n = parseInt(code);
		if (n >= 200 && n < 300) return 'bg-green-500/60';
		if (n >= 400 && n < 500) return 'bg-yellow-400/60';
		if (n >= 500) return 'bg-destructive/60';
		return 'bg-muted-foreground/40';
	}

	function statusTextColor(code: string): string {
		const n = parseInt(code);
		if (n >= 200 && n < 300) return 'text-green-400';
		if (n >= 400 && n < 500) return 'text-yellow-400';
		if (n >= 500) return 'text-destructive';
		return 'text-muted-foreground';
	}

	const benchmarkMaxHistCount = $derived(
		benchmarkPayload
			? Math.max(...benchmarkPayload.latency_histogram.map(([, c]) => c), 1)
			: 1
	);

	const benchmarkSuccessRate = $derived(
		benchmarkPayload
			? (benchmarkPayload.success_rate != null
				? benchmarkPayload.success_rate * 100
				: (benchmarkPayload.successful_requests / benchmarkPayload.requests) * 100)
			: 0
	);

	const benchmarkMaxStatusCount = $derived(
		benchmarkPayload
			? Math.max(...Object.values(benchmarkPayload.status_code_distribution), 1)
			: 1
	);

	const benchmarkPercentiles = $derived(
		benchmarkPayload
			? [
				{ label: 'p10', value: benchmarkPayload.latency.p10_ms ?? null },
				{ label: 'p25', value: benchmarkPayload.latency.p25_ms ?? null },
				{ label: 'p50', value: benchmarkPayload.latency.p50_ms },
				{ label: 'p75', value: benchmarkPayload.latency.p75_ms ?? null },
				{ label: 'p90', value: benchmarkPayload.latency.p90_ms ?? null },
				{ label: 'p95', value: benchmarkPayload.latency.p95_ms },
				{ label: 'p99', value: benchmarkPayload.latency.p99_ms },
				{ label: 'p99.9', value: benchmarkPayload.latency.p99_9_ms ?? null }
			].filter(p => p.value != null) as { label: string; value: number }[]
			: []
	);

	const benchmarkLiveMaxHistCount = $derived(
		benchmarkLive.histogram.length > 0
			? Math.max(...benchmarkLive.histogram.map(([, c]) => c), 1)
			: 1
	);

	const benchmarkLivePercentiles = $derived(
		[
			{ label: 'p10', value: benchmarkLive.p10 },
			{ label: 'p25', value: benchmarkLive.p25 },
			{ label: 'p50', value: benchmarkLive.p50 },
			{ label: 'p75', value: benchmarkLive.p75 },
			{ label: 'p90', value: benchmarkLive.p90 },
			{ label: 'p95', value: benchmarkLive.latencyP95Ms },
			{ label: 'p99', value: benchmarkLive.p99 },
			{ label: 'p99.9', value: benchmarkLive.p99_9 }
		].filter((p) => p.value > 0)
	);

	const runMethod = $derived(
		run?.analysis_type === 'http_benchmark'
			? benchmarkPayload?.method ?? benchmarkLive.method ?? 'GET'
			: null
	);
</script>

<svelte:head>
	<title>RUN {runId.slice(0, 8).toUpperCase()} — PAGELENS</title>
</svelte:head>

<div class="mx-auto max-w-6xl px-6 py-10">
	<!-- Run header -->
	<div class="mb-8 border border-border bg-background">
		<div class="grid gap-px bg-border lg:grid-cols-[1fr_auto]">
			<div class="bg-background p-6 md:p-7">
				<div class="mb-3 text-[10px] tracking-[0.35em] text-muted-foreground uppercase">Run overview</div>
				<div class="mb-4 break-all font-mono text-sm leading-relaxed text-foreground/95">{run?.url ?? 'Loading URL…'}</div>
				<div class="flex flex-wrap items-center gap-2">
					<div class="border border-border bg-secondary/40 px-2.5 py-1 text-[10px] font-bold tracking-widest uppercase text-muted-foreground">
						{run?.analysis_type ?? '—'}
					</div>
					{#if runMethod}
						<div class="border border-primary/50 bg-primary/10 px-2.5 py-1 font-mono text-[10px] font-bold tracking-widest uppercase text-primary">
							{runMethod}
						</div>
					{/if}
					<div class="border border-border px-2.5 py-1 font-mono text-[10px] font-bold tracking-widest uppercase text-foreground/80">
						{effectiveProgress(run) != null
							? Math.round((effectiveProgress(run) ?? 0) * 100) + '%'
							: '—'} complete
					</div>
				</div>
			</div>

			<div class="bg-background p-6 md:p-7 lg:min-w-[272px]">
				<div class="flex flex-wrap items-center justify-end gap-2">
					{#if run}
						<button
							type="button"
							onclick={rerunCurrentRun}
							disabled={rerunningRun}
							class="h-9 border border-primary/60 px-3 text-xs font-bold tracking-widest uppercase text-primary transition-colors hover:bg-primary/10 disabled:opacity-40"
						>
							{rerunningRun ? 'RE-RUNNING…' : 'RE-RUN'}
						</button>
					{/if}
					{#if run && (run.status === 'running' || run.status === 'pending') && !isCancelPending(run) && !isCancelledRun(run)}
						<button
							type="button"
							onclick={stopRun}
							disabled={cancelling}
							class="h-9 border border-destructive/60 px-3 text-xs font-bold tracking-widest uppercase text-destructive transition-colors hover:bg-destructive/10 disabled:opacity-40"
						>
							{cancelling ? 'CANCELLING…' : 'CANCEL RUN'}
						</button>
					{/if}
					<div class="h-9 border px-3 inline-flex items-center text-xs font-bold tracking-widest uppercase {statusClass(run)}">
						{statusLabel(run)}
					</div>
				</div>
			</div>
		</div>

		{#if run}
			{#if isCancelledRun(run)}
				<div class="mb-4 border border-yellow-500/50 bg-yellow-500/10 px-4 py-2 text-[10px] tracking-widest text-yellow-300 uppercase">
					Run canceled by user before completion. Showing finalized partial results.
				</div>
			{/if}
			<div class="grid grid-cols-2 gap-px bg-border md:grid-cols-4">
				<div class="bg-background p-4">
					<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">URL</div>
					<div class="truncate font-mono text-xs">{run.url}</div>
				</div>
				<div class="bg-background p-4">
					<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Method</div>
					<div class="font-mono text-xs">{runMethod ?? '—'}</div>
				</div>
				<div class="bg-background p-4">
					<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Status</div>
					<div class="font-mono text-xs uppercase">{statusLabel(run)}</div>
				</div>
				{#if run.analysis_type === 'crawl'}
					<div class="bg-background p-4">
						<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Discovered</div>
						<div class="font-mono text-xs">{liveStats.discovered}</div>
					</div>
					<div class="bg-background p-4">
						<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Scanned</div>
						<div class="font-mono text-xs">{liveStats.scanned}</div>
					</div>
					<div class="bg-background p-4">
						<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Queued/Running</div>
						<div class="font-mono text-xs">{liveStats.queued}/{liveStats.running}</div>
					</div>
					<div class="bg-background p-4">
						<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">Links Found</div>
						<div class="font-mono text-xs">{liveStats.linksFoundTotal}</div>
					</div>
				{/if}
			</div>

			{#if run.status === 'running' || run.status === 'pending'}
				<div class="mt-4 h-px w-full bg-border">
					<div
						class="h-px bg-primary transition-all duration-500"
						style="width: {effectiveProgress(run) != null ? Math.round((effectiveProgress(run) ?? 0) * 100) : 0}%"
					></div>
				</div>
			{/if}
		{:else if error}
			<div class="border border-destructive/40 bg-destructive/10 px-4 py-3 text-xs text-destructive">
				ERROR: {error}
			</div>
		{:else}
			<div class="text-xs text-muted-foreground">Loading...</div>
		{/if}
	</div>

	<!-- Analysis summary (non-benchmark only) -->
	{#if run?.analysis_type !== 'http_benchmark'}
		<div class="mb-6">
			{#if run?.summary}
				<div class="border border-border">
					<div class="border-b border-border px-4 py-2">
						<span class="text-xs font-bold tracking-widest uppercase">Summary</span>
					</div>
					<div class="grid grid-cols-2 gap-px bg-border md:grid-cols-3 h-56">
						{#each [
							{ label: 'SEO Score', value: run.summary.seo_score != null ? Math.round(run.summary.seo_score).toString() : '—', color: scoreColor(run.summary.seo_score) },
							{ label: 'Pages', value: run.summary.page_count.toString(), color: 'text-foreground' },
							{ label: 'Issues', value: run.summary.total_issues.toString(), color: run.summary.total_issues > 0 ? 'text-destructive' : 'text-green-400' },
							{ label: 'Errors', value: run.summary.error_count.toString(), color: run.summary.error_count > 0 ? 'text-destructive' : 'text-muted-foreground' },
							{ label: 'Warnings', value: run.summary.warning_count.toString(), color: run.summary.warning_count > 0 ? 'text-yellow-400' : 'text-muted-foreground' },
							{ label: 'Duration', value: run.summary.duration_ms > 0 ? run.summary.duration_ms + 'ms' : '—', color: 'text-foreground' }
						] as stat}
							<div class="bg-background flex flex-col justify-center px-4 py-3">
								<div class="mb-1 text-xs tracking-widest text-muted-foreground uppercase">{stat.label}</div>
								<div class="font-mono text-xl font-bold {stat.color}">{stat.value}</div>
							</div>
						{/each}
					</div>
				</div>
			{:else}
				<div class="border border-border">
					<div class="border-b border-border px-4 py-2">
						<span class="text-xs font-bold tracking-widest uppercase">Summary</span>
					</div>
					<div class="flex h-56 items-center justify-center">
						<span class="text-xs text-muted-foreground">
							{done ? 'No summary data' : 'Available when analysis completes'}
						</span>
					</div>
				</div>
			{/if}
		</div>
	{/if}

	{#if run?.analysis_type === 'http_benchmark'}
		{#if (run.status === 'running' || run.status === 'pending') && !(benchmarkPayload && (done || isCancelledRun(run)))}
			<div class="mb-8 border border-border font-mono text-xs">
				<div class="border-b border-border bg-secondary/30 px-6 py-3">
					<div class="mb-2 flex items-center gap-3">
						<span class="font-black text-primary tracking-widest">{benchmarkLive.method}</span>
						<span class="text-muted-foreground break-all">{run.url}</span>
					</div>
					<div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-[10px] tracking-wider text-muted-foreground/60 uppercase">
						{#if benchmarkLive.targetDurationSecs > 0}
							<span>{benchmarkLive.targetDurationSecs.toFixed(0)}s target</span>
						{:else}
							<span>benchmark run</span>
						{/if}
						<span>·</span>
						<span>{benchmarkLive.connections || '—'} connections</span>
						<span>·</span>
						<span>{fmtDuration(benchmarkLive.elapsedMs)} total</span>
					</div>
				</div>

				<div class="grid grid-cols-2 gap-px bg-border border-b border-border md:grid-cols-4 lg:grid-cols-5">
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Success</div>
						<div class="text-2xl font-black tabular-nums {benchmarkLive.successRate >= 100 ? 'text-green-400' : benchmarkLive.successRate >= 99 ? 'text-yellow-400' : 'text-destructive'}">
							{benchmarkLive.successRate.toFixed(2)}%
						</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Req/sec</div>
						<div class="text-2xl font-black tabular-nums text-primary">{benchmarkLive.requestsPerSec.toFixed(2)}</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Fastest</div>
						<div class="text-2xl font-black tabular-nums text-green-400">{fmtMs(benchmarkLive.latencyMinMs)}</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Average</div>
						<div class="text-2xl font-black tabular-nums text-foreground">{fmtMs(benchmarkLive.latencyAvgMs)}</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">p95</div>
						<div class="text-2xl font-black tabular-nums {latencyTextColor(benchmarkLive.latencyP95Ms)}">{fmtMs(benchmarkLive.latencyP95Ms)}</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Slowest</div>
						<div class="text-2xl font-black tabular-nums text-yellow-400">{fmtMs(benchmarkLive.latencyMaxMs)}</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Total data</div>
						<div class="text-xl font-black tabular-nums text-foreground">{fmtBytes(benchmarkLive.totalDataBytes)}</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Size/req</div>
						<div class="text-xl font-black tabular-nums text-foreground">{fmtBytes(benchmarkLive.avgSizePerRequestBytes)}</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Size/sec</div>
						<div class="text-xl font-black tabular-nums text-primary">{fmtBytes(benchmarkLive.dataPerSecBytes)}/s</div>
					</div>
				</div>

				<div class="grid grid-cols-1 gap-px bg-border md:grid-cols-2">
					<div class="bg-background p-6">
						<div class="mb-5 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Response Time Histogram</div>
						{#if benchmarkLive.histogram.length > 0}
							<div class="space-y-2">
								{#each benchmarkLive.histogram as [ms, count]}
									{@const liveBarPct = (count / benchmarkLiveMaxHistCount * 100).toFixed(1)}
									<div class="flex items-center gap-3">
										<div class="w-20 shrink-0 text-right tabular-nums {latencyTextColor(ms)}">{fmtMs(ms)}</div>
										<div class="w-10 shrink-0 text-right text-muted-foreground/60 tabular-nums">[{count}]</div>
										<div class="flex-1 h-3.5 bg-secondary/30 relative overflow-hidden border border-border/30">
											<div class="absolute inset-y-0 left-0 {latencyBarColor(ms)}" style="width: {liveBarPct}%"></div>
										</div>
									</div>
								{/each}
							</div>
						{:else}
							<div class="text-[10px] text-muted-foreground/50">Waiting for first latency samples...</div>
						{/if}
					</div>
					<div class="bg-background p-6 flex flex-col gap-7">
						<div>
							<div class="mb-4 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Latency Distribution</div>
							{#if benchmarkLivePercentiles.length > 0}
								<div class="space-y-2.5">
									{#each benchmarkLivePercentiles as pct}
										<div class="flex items-baseline gap-3">
											<span class="w-8 shrink-0 text-[10px] tracking-widest text-muted-foreground/60 uppercase">{pct.label}</span>
											<div class="flex-1 border-b border-dotted border-border/30"></div>
											<span class="tabular-nums font-bold {latencyTextColor(pct.value)}">{fmtMs(pct.value)}</span>
										</div>
									{/each}
								</div>
							{:else}
								<div class="text-[10px] text-muted-foreground/50">Waiting for percentile data...</div>
							{/if}
						</div>
						<div>
							<div class="mb-4 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Status Code Distribution</div>
							<div class="text-[10px] text-muted-foreground/50">Status buckets become available from persisted payload after completion.</div>
						</div>
					</div>
				</div>
			</div>
		{/if}

		{#if benchmarkPayload && (done || isCancelledRun(run))}
			<div class="mb-8 border border-border font-mono text-xs">
				<!-- Config bar -->
				<div class="border-b border-border bg-secondary/30 px-6 py-3">
					{#if isCancelledRun(run)}
						<div class="mb-3 border border-yellow-500/50 bg-yellow-500/10 px-3 py-2 text-[10px] tracking-wide text-yellow-300 uppercase">
							Benchmark stopped early — showing finalized partial results.
						</div>
					{/if}
					<div class="mb-2 flex items-center gap-3">
						<span class="font-black text-primary tracking-widest">{benchmarkPayload.method}</span>
						<span class="text-muted-foreground break-all">{benchmarkPayload.url}</span>
					</div>
					<div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-[10px] tracking-wider text-muted-foreground/60 uppercase">
						{#if benchmarkPayload.is_duration_mode}
							<span>{benchmarkPayload.target_duration_secs ?? '—'}s target</span>
						{:else}
							<span>{benchmarkPayload.requests} requests</span>
						{/if}
						<span>·</span>
						<span>{benchmarkPayload.connections} connections</span>
						<span>·</span>
						<span>{fmtDuration(benchmarkPayload.duration_ms)} total</span>
					</div>
				</div>

				<div class="grid grid-cols-2 gap-px bg-border border-b border-border md:grid-cols-4 lg:grid-cols-5">
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Success</div>
						<div class="text-2xl font-black tabular-nums {benchmarkSuccessRate >= 100 ? 'text-green-400' : benchmarkSuccessRate >= 99 ? 'text-yellow-400' : 'text-destructive'}">
							{benchmarkSuccessRate.toFixed(2)}%
						</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Req/sec</div>
						<div class="text-2xl font-black tabular-nums text-primary">
							{benchmarkPayload.requests_per_sec.toFixed(2)}
						</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Fastest</div>
						<div class="text-2xl font-black tabular-nums text-green-400">
							{fmtMs(benchmarkPayload.latency.min_ms)}
						</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Average</div>
						<div class="text-2xl font-black tabular-nums text-foreground">
							{fmtMs(benchmarkPayload.latency.avg_ms)}
						</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">p95</div>
						<div class="text-2xl font-black tabular-nums {latencyTextColor(benchmarkPayload.latency.p95_ms)}">
							{fmtMs(benchmarkPayload.latency.p95_ms)}
						</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Slowest</div>
						<div class="text-2xl font-black tabular-nums text-yellow-400">
							{fmtMs(benchmarkPayload.latency.max_ms)}
						</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Total data</div>
						<div class="text-xl font-black tabular-nums text-foreground">
							{fmtBytes(benchmarkPayload.total_data_bytes ?? 0)}
						</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Size/req</div>
						<div class="text-xl font-black tabular-nums text-foreground">
							{fmtBytes(benchmarkPayload.avg_size_per_request_bytes ?? 0)}
						</div>
					</div>
					<div class="bg-background p-5">
						<div class="mb-2 text-[10px] tracking-widest text-muted-foreground uppercase">Size/sec</div>
						<div class="text-xl font-black tabular-nums text-primary">
							{fmtBytes(benchmarkPayload.data_per_sec_bytes ?? 0)}/s
						</div>
					</div>
				</div>

				<!-- Histogram + Distribution split -->
				<div class="grid grid-cols-1 gap-px bg-border md:grid-cols-2">

					<!-- Left: Response time histogram -->
					<div class="bg-background p-6">
						<div class="mb-5 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
							Response Time Histogram
						</div>
						{#if benchmarkPayload.latency_histogram.length > 0}
							<div class="space-y-2">
								{#each benchmarkPayload.latency_histogram as [ms, count]}
									{@const barPct = (count / benchmarkMaxHistCount * 100).toFixed(1)}
									<div class="flex items-center gap-3">
										<div class="w-20 shrink-0 text-right tabular-nums {latencyTextColor(ms)}">
											{fmtMs(ms)}
										</div>
										<div class="w-10 shrink-0 text-right text-muted-foreground/60 tabular-nums">
											[{count}]
										</div>
										<div class="flex-1 h-3.5 bg-secondary/30 relative overflow-hidden border border-border/30">
											<div
												class="absolute inset-y-0 left-0 {latencyBarColor(ms)}"
												style="width: {barPct}%"
											></div>
										</div>
									</div>
								{/each}
							</div>
						{:else}
							<div class="text-[10px] text-muted-foreground/50">No histogram buckets yet.</div>
						{/if}
					</div>

					<!-- Right: Percentile distribution + status codes -->
					<div class="bg-background p-6 flex flex-col gap-7">

					<!-- Percentile table -->
					<div>
						<div class="mb-4 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
							Latency Distribution
						</div>
						<div class="space-y-2.5">
							{#each benchmarkPercentiles as pct}
								<div class="flex items-baseline gap-3">
									<span class="w-8 shrink-0 text-[10px] tracking-widest text-muted-foreground/60 uppercase">
										{pct.label}
									</span>
									<div class="flex-1 border-b border-dotted border-border/30"></div>
									<span class="tabular-nums font-bold {latencyTextColor(pct.value)}">
										{fmtMs(pct.value)}
									</span>
								</div>
							{/each}
						</div>
					</div>

						<!-- Status code distribution -->
						<div>
							<div class="mb-4 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
								Status Code Distribution
							</div>
							{#if Object.keys(benchmarkPayload.status_code_distribution).length > 0}
								<div class="space-y-2">
									{#each Object.entries(benchmarkPayload.status_code_distribution).sort(([, a], [, b]) => b - a) as [code, count]}
										<div class="flex items-center gap-3">
											<span class="w-12 shrink-0 text-right font-bold tabular-nums {statusTextColor(code)}">
												[{code}]
											</span>
											<span class="w-8 shrink-0 text-right text-muted-foreground/60 tabular-nums">
												{count}
											</span>
											<div class="flex-1 h-2.5 bg-secondary/30 relative overflow-hidden border border-border/30">
												<div
													class="absolute inset-y-0 left-0 {statusBgColor(code)}"
													style="width: {(count / benchmarkMaxStatusCount * 100).toFixed(1)}%"
												></div>
											</div>
										</div>
									{/each}
								</div>
							{:else}
								<div class="text-[10px] text-muted-foreground/50">No status code data yet.</div>
							{/if}
						</div>
					</div>
				</div>

				<!-- Error distribution (if any) -->
				{#if Object.keys(benchmarkPayload.error_distribution ?? {}).length > 0}
					<div class="border-t border-border bg-background p-6">
						<div class="mb-4 text-[10px] tracking-[0.4em] text-destructive uppercase">
							Error Distribution
						</div>
						<div class="space-y-2">
							{#each Object.entries(benchmarkPayload.error_distribution ?? {}).sort(([, a], [, b]) => b - a) as [err, count]}
								<div class="flex items-baseline gap-4">
									<span class="shrink-0 font-bold tabular-nums text-destructive">[{count}]</span>
									<span class="text-muted-foreground break-all">{err}</span>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<div class="border-t border-border bg-secondary/20 flex items-center gap-6 px-6 py-3 text-[10px] text-muted-foreground/60 tracking-wider">
					<span>
						<span class="text-green-400 font-bold">{benchmarkPayload.successful_requests}</span> successful
					</span>
					{#if benchmarkPayload.failed_requests > 0}
						<span>
							<span class="text-destructive font-bold">{benchmarkPayload.failed_requests}</span> failed
						</span>
					{/if}
					<span>
						<span class="text-foreground/60">{benchmarkPayload.requests}</span> total
					</span>
				</div>
			</div>
		{/if}
	{/if}

	<!-- Pages table -->
	{#if run?.analysis_type !== 'http_benchmark'}
		<div class="border border-border">
			<div class="border-b border-border px-4 py-2">
				<span class="text-xs font-bold tracking-widest uppercase">
					Pages ({pages.length})
				</span>
			</div>

			<!-- Table header -->
			<div class="grid grid-cols-[24px_1fr_80px_64px_56px_56px_140px_56px_32px] gap-3 border-b border-border bg-secondary/40 px-4 py-2 text-xs font-bold tracking-widest text-muted-foreground uppercase">
				<div></div>
				<div>URL</div>
				<div class="text-right">SEO</div>
				<div class="text-right">Issues</div>
				<div class="text-right">Err</div>
				<div class="text-right">Warn</div>
				<div>Analysed</div>
				<div></div>
				<div></div>
			</div>

			<div class="divide-y divide-border">
				{#if pages.length === 0}
					<div class="px-4 py-6 font-mono text-xs text-muted-foreground">
						{run?.status === 'running' || run?.status === 'pending'
							? 'Waiting for first analyzed page...'
							: 'No page results found.'}
					</div>
				{/if}
				{#each pages as pg}
					<div
						onclick={() => goto(`/run/${runId}/page/${pg.id}`)}
						class="grid grid-cols-[24px_1fr_80px_64px_56px_56px_140px_56px_32px] cursor-pointer items-center gap-3 px-4 py-3 font-mono text-xs hover:bg-secondary/40 transition-colors group"
					>
						<div class="{pg.success ? 'text-green-400' : 'text-destructive'} font-bold">
							{pg.success ? '✓' : '✗'}
						</div>

						<div class="min-w-0">
							<div class="truncate group-hover:text-primary transition-colors">{pg.url}</div>
						</div>

						<div class="text-right font-bold {scoreColor(pg.seo_score)}">
							{pg.seo_score != null ? Math.round(pg.seo_score) : '—'}
						</div>

						<div class="text-right {pg.total_issues > 0 ? 'text-destructive' : 'text-muted-foreground'}">
							{pg.total_issues}
						</div>

						<div class="text-right {pg.error_count > 0 ? 'text-destructive' : 'text-muted-foreground'}">
							{pg.error_count}
						</div>

						<div class="text-right {pg.warning_count > 0 ? 'text-yellow-400' : 'text-muted-foreground'}">
							{pg.warning_count}
						</div>

						<div class="text-muted-foreground truncate">
							{new Date(pg.analyzed_at).toLocaleTimeString()}
						</div>

						<button
							onclick={(e) => { e.stopPropagation(); rerunPage(pg.url); }}
							disabled={rerunningUrl != null}
							title="Re-run analysis for this page"
							class="border border-border px-2 py-0.5 text-xs text-muted-foreground hover:border-primary hover:text-primary transition-colors disabled:opacity-40"
						>
							{rerunningUrl === pg.url ? '…' : '↻'}
						</button>

						<div class="text-muted-foreground group-hover:text-primary transition-colors text-right">
							→
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>
