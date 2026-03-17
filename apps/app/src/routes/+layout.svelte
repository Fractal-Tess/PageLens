<script lang="ts">
	import './layout.css';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { cancelRun, runResultPath, type AnalysisRun } from '$lib/api';
	import { activeRunsStore } from '$lib/active-runs-store';

	let { children } = $props();

	let activeRuns = $state<AnalysisRun[]>([]);
	let activeRunError = $state('');
	let bootstrappedActiveRun = $state(false);
	let cancellingActiveRun = $state(false);
	let activeRunActionError = $state('');
	let activeRunsUnsubscribe: (() => void) | null = null;
	const activeRun = $derived(activeRuns[0] ?? null);

	function isRunActive(run: AnalysisRun | null | undefined): boolean {
		if (!run) return false;
		return run.status === 'pending' || run.status === 'running';
	}

	function analysisTypeLabel(run: AnalysisRun): string {
		if (run.analysis_type === 'single') return 'Single Page';
		if (run.analysis_type === 'crawl') return 'Full Crawl';
		if (run.analysis_type === 'http_benchmark') return 'HTTP Benchmark';
		if (run.analysis_type === 'favicon') return 'Favicon Analyzer';
		return run.analysis_type;
	}

	function progressLabel(run: AnalysisRun): string {
		if (run.progress == null) return run.status.toUpperCase();
		return `${Math.round(run.progress * 100)}%`;
	}
	const runActive = $derived(isRunActive(activeRun));

	async function openActiveRun(): Promise<void> {
		if (!activeRun) return;
		await goto(runResultPath(activeRun.id, activeRun.analysis_type));
	}

	async function stopActiveRun(): Promise<void> {
		const run = activeRun;
		if (!run || cancellingActiveRun) return;
		if (!isRunActive(run)) return;
		cancellingActiveRun = true;
		activeRunActionError = '';
		try {
			await cancelRun(run.id);
			await activeRunsStore.refresh();
		} catch (err) {
			activeRunActionError = err instanceof Error ? err.message : 'Failed to cancel active job';
		} finally {
			cancellingActiveRun = false;
		}
	}

	onMount(() => {
		activeRunsStore.start();
		activeRunsUnsubscribe = activeRunsStore.subscribe((state) => {
			activeRuns = state.runs;
			activeRunError = state.error;
			bootstrappedActiveRun = state.bootstrapped;
		});

		return () => {
			if (activeRunsUnsubscribe) {
				activeRunsUnsubscribe();
				activeRunsUnsubscribe = null;
			}
			activeRunsStore.stop();
		};
	});
</script>

<div class="relative flex min-h-screen flex-col bg-background text-foreground">
	<!-- Scan line overlay for atmosphere -->
	<div class="scan-line pointer-events-none fixed inset-0 z-50 opacity-30"></div>

	<!-- Nav -->
	<header class="sticky top-0 z-40 border-b border-border bg-background/95 backdrop-blur-sm">
		<div class="mx-auto flex min-h-12 max-w-6xl items-center justify-between gap-3 px-6 py-2">
			<a href="/" class="flex items-center gap-2 text-sm font-bold tracking-widest uppercase">
				<span class="text-primary">◈</span>
				<span>PAGELENS</span>
			</a>
			<nav class="flex flex-wrap items-center justify-end gap-3 md:gap-4">
				<a
					href="/analyse/page"
					class="text-[10px] tracking-widest uppercase text-muted-foreground hover:text-foreground transition-colors"
				>
					PAGE
				</a>
				<a
					href="/analyse/crawl"
					class="text-[10px] tracking-widest uppercase text-muted-foreground hover:text-foreground transition-colors"
				>
					CRAWL
				</a>
				<a
					href="/analyse/benchmark"
					class="text-[10px] tracking-widest uppercase text-muted-foreground hover:text-foreground transition-colors"
				>
					BENCHMARK
				</a>
				<a
					href="/analyse/favicon"
					class="text-[10px] tracking-widest uppercase text-muted-foreground hover:text-foreground transition-colors"
				>
					FAVICON
				</a>
				<a
					href="/analyse/pwa"
					class="text-[10px] tracking-widest uppercase text-muted-foreground hover:text-foreground transition-colors"
				>
					PWA
				</a>
				<a
					href="/zellij"
					class="text-[10px] tracking-widest uppercase text-muted-foreground hover:text-foreground transition-colors"
				>
					ZELLIJ
				</a>
				<span class="text-border">|</span>
				<a
					href="/history"
					class="text-[10px] tracking-widest uppercase text-muted-foreground hover:text-foreground transition-colors"
				>
					HISTORY
				</a>

				<div
					class="ml-1 inline-flex items-center gap-2"
					title={activeRun
						? `${analysisTypeLabel(activeRun)} · ${progressLabel(activeRun)} · ${activeRun.url}`
						: 'No active jobs'}
				>
					<span
						class="h-2 w-2 rounded-full {runActive ? 'bg-green-400 shadow-[0_0_8px_rgba(74,222,128,0.6)]' : 'bg-destructive/80'}"
					></span>
					{#if activeRun}
						<span class="font-mono text-[9px] font-bold text-green-400">{progressLabel(activeRun)}</span>
						<button
							type="button"
							onclick={openActiveRun}
							class="h-6 border border-primary/50 px-1.5 text-[9px] font-bold tracking-wider uppercase text-primary transition-colors hover:bg-primary/10"
						>
							OPEN
						</button>
						<button
							type="button"
							onclick={stopActiveRun}
							disabled={cancellingActiveRun || !runActive}
							class="h-6 border border-destructive/50 px-1.5 text-[9px] font-bold tracking-wider uppercase text-destructive transition-colors hover:bg-destructive/10 disabled:opacity-40"
						>
							{cancellingActiveRun ? '...' : 'STOP'}
						</button>
					{/if}
				</div>
			</nav>
		</div>

		{#if bootstrappedActiveRun && activeRunError}
			<div class="border-t border-destructive/30 bg-destructive/10 px-6 py-2 text-[10px] text-destructive/90">
				Failed to fetch active job: {activeRunError}
			</div>
		{/if}
		{#if activeRunActionError}
			<div class="border-t border-destructive/30 bg-destructive/10 px-6 py-2 text-[10px] text-destructive/90">
				Failed to cancel active job: {activeRunActionError}
			</div>
		{/if}
	</header>

	<main>
		{@render children()}
	</main>

	<!-- Footer -->
	<footer class="border-t border-border mt-auto">
		<div class="mx-auto flex h-10 max-w-6xl items-center justify-between px-6">
			<span class="text-xs text-muted-foreground tracking-wider">PAGELENS / WEB ANALYSIS</span>
			<span class="text-xs text-muted-foreground">http://127.0.0.1:8787</span>
		</div>
	</footer>
</div>
