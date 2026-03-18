<script lang="ts">
	import { goto } from '$app/navigation';
	import { startAnalysis } from '$lib/api';
	import { Button } from '@pagelens/ui/shadcn/button';
	import { Input } from '@pagelens/ui/shadcn/input';
	import { z } from 'zod';

	// ── form state ──────────────────────────────────────────────────────────────
	let url = $state('');
	let durationSecs = $state(10);
	let connections = $state(50);
	let method = $state('GET');
	let qps = $state('');
	let showAdvanced = $state(false);
	let headersInput = $state('');
	let cookiesInput = $state('');
	let requestBody = $state('');
	let followRedirects = $state(true);

	const urlSchema = z.string().trim().url().startsWith('http');
	function isValidUrl(s: string) { return urlSchema.safeParse(s).success; }

	// ── run phase ────────────────────────────────────────────────────────────────
	type Phase = 'idle' | 'starting' | 'error';
	let phase = $state<Phase>('idle');
	let formError = $state('');

	function parseTupleList(raw: string): Array<[string, string]> {
		return raw
			.split('\n')
			.map((line) => line.trim())
			.filter((line) => line.length > 0)
			.map((line) => {
				const idx = line.indexOf(':');
				if (idx < 0) return null;
				const key = line.slice(0, idx).trim();
				const value = line.slice(idx + 1).trim();
				if (!key) return null;
				return [key, value] as [string, string];
			})
			.filter((item): item is [string, string] => item !== null);
	}

	// ── submit ───────────────────────────────────────────────────────────────────
	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!url.trim()) return;
		formError = '';
		phase = 'starting';

		try {
			const parsedHeaders = parseTupleList(headersInput);
			const parsedCookies = parseTupleList(cookiesInput);
			const trimmedQps = qps.trim();
			const parsedQps = trimmedQps.length > 0 ? Number(trimmedQps) : undefined;
			const { run_id } = await startAnalysis({
				url: url.trim(),
				analysis_type: 'http_benchmark',
				benchmark_options: {
					duration_secs: Number.isFinite(durationSecs) && durationSecs > 0 ? durationSecs : 10,
					connections,
					method,
					qps: Number.isFinite(parsedQps) ? parsedQps : undefined,
					headers: parsedHeaders.length > 0 ? parsedHeaders : undefined,
					cookies: parsedCookies.length > 0 ? parsedCookies : undefined,
					body: requestBody.trim().length > 0 ? requestBody : undefined,
					follow_redirects: followRedirects,
				}
			});
			await goto(`/run/${run_id}`);
		} catch (err) {
			formError = err instanceof Error ? err.message : 'Unknown error';
			phase = 'error';
		}
	}

	const httpMethods = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'];
</script>

<svelte:head>
	<title>PAGELENS — HTTP Benchmark</title>
</svelte:head>

<div class="mx-auto max-w-6xl px-6 py-8">

	<!-- Breadcrumb -->
	<div class="animate-fade-in mb-6 flex items-center gap-2 text-[10px] tracking-widest text-muted-foreground uppercase">
		<a href="/" class="transition-colors hover:text-foreground">PAGELENS</a>
		<span class="text-border">/</span>
		<span class="text-primary">HTTP BENCHMARK</span>
	</div>

	<!-- ── IDLE / FORM ─────────────────────────────────────────────────────── -->
	{#if phase === 'idle' || phase === 'error'}

		<!-- Header block -->
		<div class="animate-fade-in-up stagger-1 mb-8 grid grid-cols-1 gap-px bg-border md:grid-cols-[2fr_1fr]">
			<div class="bg-background p-5 md:p-6">
				<div class="mb-2 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
					03 · Analysis mode
				</div>
				<h1 class="text-3xl font-black leading-none tracking-tighter md:text-4xl">
					HTTP <span class="text-primary">BENCHMARK</span>
				</h1>
				<p class="mt-3 max-w-sm text-xs leading-relaxed text-muted-foreground">
					Send a controlled burst of concurrent HTTP requests to any endpoint.
					Measure raw throughput, latency at every percentile, and failure characteristics under load.
				</p>
			</div>
			<div class="bg-background p-5 md:p-6">
				<div class="mb-3 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Output</div>
				<div class="space-y-1.5">
					{#each ['Requests/sec (RPS)', 'p10 / p25 / p50 / p99.9 latency', 'Fastest / average / slowest', 'Total data + size/sec', 'Status code distribution', 'Error breakdown'] as item}
						<div class="flex items-baseline gap-2 text-[11px] text-muted-foreground">
							<span class="shrink-0 text-primary/60">—</span>
							<span>{item}</span>
						</div>
					{/each}
				</div>
			</div>
		</div>

		<!-- Form -->
		<form onsubmit={handleSubmit} class="animate-fade-in-up stagger-2 mb-8">
			<div class="border border-border bg-background">

				<!-- URL + method -->
				<div class="border-b border-border p-8">
					<div class="mb-1 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
						Target endpoint
					</div>
					<p class="mb-6 text-xs text-muted-foreground">
						The URL that receives all benchmark requests.
					</p>
					<div class="flex flex-col gap-3 md:flex-row md:gap-0">
						<Input
							bind:value={url}
							type="url"
							placeholder="https://api.example.com/endpoint"
							required
							class="h-14 flex-1 border border-border bg-transparent px-4 font-mono text-sm focus-visible:ring-0 focus-visible:border-primary rounded-none placeholder:text-muted-foreground/30 md:border-r-0"
						/>
						<div class="flex h-14 items-center border border-border bg-secondary px-4 md:border-l-0">
							<select
								bind:value={method}
								class="bg-transparent text-xs font-bold tracking-widest uppercase text-foreground focus:outline-none cursor-pointer"
							>
								{#each httpMethods as m}
									<option value={m} class="bg-background text-foreground">{m}</option>
								{/each}
							</select>
						</div>
						<Button
							type="submit"
							disabled={!isValidUrl(url)}
							class="h-14 rounded-none border border-primary bg-primary px-8 text-xs font-bold tracking-widest uppercase text-primary-foreground hover:bg-primary/90 disabled:opacity-40 md:border-l-0"
						>
							BENCHMARK URL →
						</Button>
					</div>
				</div>

				<!-- Parameters -->
				<div class="p-8">
					<div class="mb-5 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">
						Load parameters
					</div>
					<div class="grid grid-cols-1 gap-px bg-border md:grid-cols-2 lg:grid-cols-3">
						<div class="bg-background p-5">
							<label class="block space-y-3">
								<div>
									<div class="text-[10px] font-bold tracking-widest uppercase">Duration (sec)</div>
									<div class="mt-0.5 text-[10px] text-muted-foreground">Time-based run window</div>
								</div>
								<Input type="number" min="1" step="1" bind:value={durationSecs}
									class="h-10 rounded-none border-border bg-transparent font-mono text-sm focus-visible:ring-0 focus-visible:border-primary" />
							</label>
						</div>
						<div class="bg-background p-5">
							<label class="block space-y-3">
								<div>
									<div class="text-[10px] font-bold tracking-widest uppercase">Connections</div>
									<div class="mt-0.5 text-[10px] text-muted-foreground">Concurrent connections</div>
								</div>
								<Input type="number" min="1" bind:value={connections}
									class="h-10 rounded-none border-border bg-transparent font-mono text-sm focus-visible:ring-0 focus-visible:border-primary" />
							</label>
						</div>
						<div class="bg-background p-5">
							<label class="block space-y-3">
								<div>
									<div class="text-[10px] font-bold tracking-widest uppercase">
										QPS <span class="text-muted-foreground normal-case font-normal">(optional)</span>
									</div>
									<div class="mt-0.5 text-[10px] text-muted-foreground">Rate limit</div>
								</div>
								<Input type="number" min="0" step="0.1" bind:value={qps}
									placeholder="Unlimited"
									class="h-10 rounded-none border-border bg-transparent font-mono text-sm focus-visible:ring-0 focus-visible:border-primary placeholder:text-muted-foreground/30" />
							</label>
						</div>
					</div>

					<div class="mt-4 text-[10px] text-muted-foreground/60">
						<span class="text-foreground/60">{durationSecs}s</span> duration ·
						<span class="text-foreground/60">{connections}</span> concurrent ·
						<span class="text-primary/80">{method}</span> ·
						{#if qps.trim()} · <span class="text-foreground/60">{qps}</span> QPS{:else} · unlimited rate{/if}
					</div>

					<div class="mt-5 border-t border-border/40 pt-5">
						<button
							type="button"
							onclick={() => (showAdvanced = !showAdvanced)}
							class="text-[10px] font-bold tracking-[0.3em] uppercase text-primary transition-colors hover:text-primary/80"
						>
							{showAdvanced ? 'Hide advanced settings' : 'Show advanced settings'}
						</button>
						{#if showAdvanced}
							<div class="mt-4 grid grid-cols-1 gap-px bg-border lg:grid-cols-2">
								<div class="bg-background p-5">
									<div class="mb-2 text-[10px] font-bold tracking-widest uppercase">Headers</div>
									<p class="mb-3 text-[10px] text-muted-foreground">One header per line: <span class="font-mono">Name: value</span></p>
									<textarea bind:value={headersInput} rows="6" class="w-full resize-y border border-border bg-transparent p-3 font-mono text-xs focus:border-primary focus:outline-none" placeholder="Authorization: Bearer token"></textarea>
								</div>
								<div class="bg-background p-5">
									<div class="mb-2 text-[10px] font-bold tracking-widest uppercase">Cookies</div>
									<p class="mb-3 text-[10px] text-muted-foreground">One cookie per line: <span class="font-mono">name: value</span></p>
									<textarea bind:value={cookiesInput} rows="6" class="w-full resize-y border border-border bg-transparent p-3 font-mono text-xs focus:border-primary focus:outline-none" placeholder="session_id: abc123"></textarea>
								</div>
								<div class="bg-background p-5 lg:col-span-2">
									<div class="mb-2 text-[10px] font-bold tracking-widest uppercase">Request body</div>
									<textarea bind:value={requestBody} rows="5" class="w-full resize-y border border-border bg-transparent p-3 font-mono text-xs focus:border-primary focus:outline-none" placeholder="key=value"></textarea>
									<label class="mt-3 flex items-center gap-2 text-[10px] tracking-widest uppercase text-muted-foreground">
										<input type="checkbox" bind:checked={followRedirects} class="h-3.5 w-3.5 rounded border-border bg-transparent" />
										<span>Follow redirects</span>
									</label>
								</div>
							</div>
						{/if}
					</div>
				</div>

			</div>

			{#if formError}
				<div class="mt-2 border border-destructive/40 bg-destructive/10 px-4 py-3 text-xs text-destructive">
					ERROR: {formError}
				</div>
			{/if}
		</form>



	<!-- ── STARTING ────────────────────────────────────────────────────────── -->
	{:else if phase === 'starting'}
		<div class="border border-border bg-background px-8 py-16 text-center">
			<div class="mb-4 text-[10px] tracking-[0.4em] text-muted-foreground uppercase">Initialising</div>
			<div class="text-2xl font-black text-primary">CONNECTING...</div>
		</div>
	{/if}
</div>
