<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { getHistory, runResultPath, startAnalysis } from '$lib/api';
	import type { HistoryListItem } from '$lib/api';
	import { Button } from '@pagelens/ui/shadcn/button';

	let history = $state<HistoryListItem[]>([]);
	let loading = $state(true);
	let error = $state('');
	let rerunning = $state<string | null>(null);

	onMount(async () => {
		try {
			history = await getHistory(100, 0);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load history';
		} finally {
			loading = false;
		}
	});

	async function rerun(item: HistoryListItem) {
		rerunning = item.id;
		try {
			const { run_id } = await startAnalysis({
				url: item.url,
				name: item.name,
				analysis_type: item.analysis_type
			});
			goto(runResultPath(run_id, item.analysis_type));
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to re-run analysis';
			rerunning = null;
		}
	}

	function statusColor(status: string) {
		if (status === 'completed') return 'text-green-400 border-green-800';
		if (status === 'failed') return 'text-destructive border-destructive/40';
		if (status === 'running') return 'text-primary border-primary/40';
		return 'text-muted-foreground border-border';
	}

	function fmtDate(s?: string) {
		if (!s) return '—';
		const d = new Date(s);
		return (
			d.toLocaleDateString() +
			' ' +
			d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
		);
	}

	function scoreColor(score: number | null) {
		if (score == null) return 'text-muted-foreground';
		if (score >= 80) return 'text-green-400';
		if (score >= 50) return 'text-yellow-400';
		return 'text-destructive';
	}
</script>

<svelte:head>
	<title>HISTORY — PAGELENS</title>
</svelte:head>

<div class="mx-auto max-w-6xl px-6 py-10">
	<div class="mb-10 flex items-end justify-between">
		<div>
			<div class="mb-2 text-xs tracking-[0.3em] text-muted-foreground uppercase">
				Analysis history
			</div>
			<h1 class="text-4xl font-bold leading-none">PAST RUNS</h1>
		</div>
		<Button
			href="/"
			variant="outline"
			class="rounded-none border-border text-xs tracking-widest uppercase hover:border-primary hover:text-primary"
		>
			NEW ANALYSIS →
		</Button>
	</div>

	{#if error}
		<div
			class="mb-6 border border-destructive/40 bg-destructive/10 px-4 py-3 text-xs text-destructive"
		>
			ERROR: {error}
		</div>
	{/if}

	{#if loading}
		<div
			class="border border-border px-6 py-20 text-center text-xs tracking-widest text-muted-foreground"
		>
			LOADING HISTORY...
		</div>
	{:else if history.length === 0}
		<div class="border border-border px-6 py-20 text-center">
			<div class="mb-2 text-2xl font-bold text-muted-foreground/30">NO RUNS YET</div>
			<div class="text-xs text-muted-foreground">
				Start your first analysis from the <a href="/" class="text-primary underline underline-offset-2"
					>home page</a
				>.
			</div>
		</div>
	{:else}
		<div class="border border-border">
			<!-- Table header -->
			<div class="grid grid-cols-[1fr_110px_80px_80px_64px_64px_160px_80px] border-b border-border bg-secondary/50 px-4 py-2 text-xs font-bold tracking-widest text-muted-foreground uppercase">
				<div>URL / Label</div>
				<div>Type</div>
				<div>Status</div>
				<div class="text-right">SEO</div>
				<div class="text-right">Issues</div>
				<div class="text-right">Pages</div>
				<div>Started</div>
				<div></div>
			</div>

			<div class="divide-y divide-border">
				{#each history as item}
					<div class="grid grid-cols-[1fr_110px_80px_80px_64px_64px_160px_80px] items-center px-4 py-3 hover:bg-secondary/30 transition-colors">
						<div class="min-w-0 pr-4">
							<a href={runResultPath(item.id, item.analysis_type)} class="group block">
								<div
									class="truncate font-mono text-xs group-hover:text-primary transition-colors"
								>
									{item.url}
								</div>
								{#if item.name}
									<div class="mt-0.5 text-xs text-muted-foreground">{item.name}</div>
								{/if}
								<div class="mt-0.5 font-mono text-xs text-muted-foreground/40">
									{item.id.slice(0, 12)}…
								</div>
							</a>
						</div>

						<div class="font-mono text-xs text-muted-foreground uppercase">
							{item.analysis_type.replace('_', ' ')}
						</div>

						<div>
							<span
								class="border px-2 py-0.5 text-xs font-bold tracking-wider uppercase {statusColor(item.status)}"
							>
								{item.status === 'completed' ? 'DONE' : item.status.toUpperCase()}
							</span>
						</div>

						<div class="text-right font-mono text-xs font-bold {scoreColor(item.summary?.seo_score ?? null)}">
							{item.summary?.seo_score != null ? Math.round(item.summary.seo_score) : '—'}
						</div>

						<div class="text-right font-mono text-xs {item.summary?.total_issues > 0 ? 'text-destructive' : 'text-muted-foreground'}">
							{item.summary?.total_issues ?? '—'}
						</div>

						<div class="text-right font-mono text-xs text-muted-foreground">
							{item.summary?.page_count ?? '—'}
						</div>

						<div class="font-mono text-xs text-muted-foreground">
							{fmtDate(item.created_at)}
						</div>

						<div class="flex justify-end">
							<button
								onclick={() => rerun(item)}
								disabled={rerunning === item.id}
								class="border border-border px-2 py-1 text-xs tracking-wider text-muted-foreground uppercase hover:border-primary hover:text-primary transition-colors disabled:opacity-40"
							>
								{rerunning === item.id ? '...' : 'RE-RUN'}
							</button>
						</div>
					</div>
				{/each}
			</div>

			<div class="border-t border-border px-4 py-2 text-xs text-muted-foreground">
				{history.length} run{history.length !== 1 ? 's' : ''} total
			</div>
		</div>
	{/if}
</div>
