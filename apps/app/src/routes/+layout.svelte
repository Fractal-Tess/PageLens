<script lang="ts">
	import './layout.css';

	let { children } = $props();
</script>

<div class="relative flex min-h-screen flex-col bg-background text-foreground">
	<!-- Scan line overlay for atmosphere -->
	<div class="scan-line pointer-events-none fixed inset-0 z-50 opacity-30"></div>

	<!-- Nav -->
	<header class="sticky top-0 z-40 border-b border-border bg-background/95 backdrop-blur-sm">
		<div class="mx-auto flex h-12 max-w-6xl items-center justify-between px-6">
			<a href="/" class="flex items-center gap-2 text-sm font-bold tracking-widest uppercase">
				<span class="text-primary">◈</span>
				<span>PAGELENS</span>
			</a>
			<nav class="flex items-center gap-5">
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
				<span class="text-border">|</span>
				<a
					href="/history"
					class="text-[10px] tracking-widest uppercase text-muted-foreground hover:text-foreground transition-colors"
				>
					HISTORY
				</a>
			</nav>
		</div>
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
