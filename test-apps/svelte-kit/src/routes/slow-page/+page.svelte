<svelte:head>
	<title>Slow Loading Page - PageLens Test App</title>
	<meta name="description" content="This page simulates slow loading conditions with artificial delays for testing loading performance and timeout handling." />
	<meta name="robots" content="noindex, nofollow" />
	<link rel="canonical" href="https://example.com/slow-page" />
</svelte:head>

<script lang="ts">
	import { onMount } from 'svelte';
	
	let { data } = $props();
	
	let clientDelayComplete = $state(false);
	let heavyImagesLoaded = $state(false);
	let progress = $state(0);
	
	// Simulate client-side loading delay
	onMount(() => {
		const interval = setInterval(() => {
			progress += Math.random() * 15;
			if (progress >= 100) {
				progress = 100;
				clearInterval(interval);
				setTimeout(() => {
					clientDelayComplete = true;
					setTimeout(() => {
						heavyImagesLoaded = true;
					}, 2000);
				}, 500);
			}
		}, 300);
		
		return () => clearInterval(interval);
	});
	
	// Generate data-uri placeholder images of various sizes
	const largePlaceholders = Array.from({ length: 12 }, (_, i) => ({
		id: i,
		width: 400 + (i % 3) * 100,
		height: 300 + (i % 2) * 100,
		color: ['from-blue-400', 'from-green-400', 'from-purple-400', 'from-orange-400', 'from-pink-400', 'from-teal-400'][i % 6]
	}));
</script>

<div class="bg-gray-50 min-h-screen">
	<!-- Header -->
	<div class="bg-red-600 py-16">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="flex items-center justify-center gap-3 mb-4">
				<svg class="w-10 h-10 text-white animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
				</svg>
				<h1 class="text-4xl font-bold text-white">Slow Loading Page</h1>
			</div>
			<p class="text-xl text-red-100 text-center max-w-3xl mx-auto">
				This page has intentional delays to test loading performance, timeout handling, and progressive rendering.
			</p>
		</div>
	</div>

	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
		<!-- Timing Info -->
		<div class="bg-white rounded-lg shadow-md p-6 mb-8">
			<h2 class="text-xl font-semibold text-gray-900 mb-4">Loading Timeline</h2>
			<div class="space-y-4">
				<div class="flex items-center gap-4">
					<div class="w-8 h-8 bg-green-500 rounded-full flex items-center justify-center text-white text-sm font-bold">1</div>
					<div class="flex-1">
						<p class="font-medium text-gray-900">Server Response Delay</p>
						<p class="text-sm text-gray-600">3 seconds artificial delay in +page.server.ts load function</p>
					</div>
					<span class="text-green-600 font-medium">Complete</span>
				</div>
				<div class="flex items-center gap-4">
					<div class="w-8 h-8 {clientDelayComplete ? 'bg-green-500' : 'bg-yellow-500 animate-pulse'} rounded-full flex items-center justify-center text-white text-sm font-bold">2</div>
					<div class="flex-1">
						<p class="font-medium text-gray-900">Client-Side Initialization</p>
						<p class="text-sm text-gray-600">Simulated JavaScript loading with progress bar</p>
						<!-- Progress Bar -->
						<div class="w-full bg-gray-200 rounded-full h-2 mt-2">
							<div class="bg-yellow-500 h-2 rounded-full transition-all duration-300" style="width: {progress}%"></div>
						</div>
					</div>
					<span class="{clientDelayComplete ? 'text-green-600' : 'text-yellow-600'} font-medium">
						{clientDelayComplete ? 'Complete' : 'Loading...'}
					</span>
				</div>
				<div class="flex items-center gap-4">
					<div class="w-8 h-8 {heavyImagesLoaded ? 'bg-green-500' : 'bg-gray-300'} rounded-full flex items-center justify-center text-white text-sm font-bold">3</div>
					<div class="flex-1">
						<p class="font-medium text-gray-900">Heavy Content Load</p>
						<p class="text-sm text-gray-600">Multiple large placeholders simulating heavy assets</p>
					</div>
					<span class="{heavyImagesLoaded ? 'text-green-600' : 'text-gray-500'} font-medium">
						{heavyImagesLoaded ? 'Complete' : 'Pending'}
					</span>
				</div>
			</div>
			
			{#if data}
				<div class="mt-6 pt-6 border-t border-gray-200">
					<p class="text-sm text-gray-600">
						<strong>Server loaded at:</strong> {new Date(data.loadedAt).toLocaleTimeString()} 
						<span class="text-gray-400">(delayed {data.delayMs}ms)</span>
					</p>
				</div>
			{/if}
		</div>

		<!-- Heavy Content Section -->
		{#if clientDelayComplete}
			<section class="mb-12" aria-labelledby="heavy-content-heading">
				<h2 id="heavy-content-heading" class="text-2xl font-bold text-gray-900 mb-6">
					Heavy Content Gallery
					{#if !heavyImagesLoaded}
						<span class="inline-flex items-center ml-3">
							<svg class="animate-spin h-5 w-5 text-indigo-600" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
							<span class="ml-2 text-sm text-indigo-600">Loading assets...</span>
						</span>
					{/if}
				</h2>
				<div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
					{#each largePlaceholders as img (img.id)}
						<div 
							class="bg-gradient-to-br {img.color} to-gray-300 rounded-lg overflow-hidden transition-opacity duration-500 {heavyImagesLoaded ? 'opacity-100' : 'opacity-50'}"
							style="aspect-ratio: {img.width}/{img.height}"
						>
							<div class="w-full h-full flex items-center justify-center text-white text-opacity-50 text-4xl">
								{heavyImagesLoaded ? '📷' : '⏳'}
							</div>
						</div>
					{/each}
				</div>
			</section>
		{:else}
			<!-- Loading Skeleton -->
			<div class="space-y-4">
				<div class="h-8 bg-gray-200 rounded w-1/3 animate-pulse"></div>
				<div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
					{#each Array(12) as _}
						<div class="bg-gray-200 rounded-lg aspect-[4/3] animate-pulse"></div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Performance Tips -->
		<section class="mt-12 bg-blue-50 rounded-lg p-6 border border-blue-200">
			<h2 class="text-xl font-semibold text-blue-900 mb-4">Performance Testing Notes</h2>
			<ul class="space-y-2 text-blue-800">
				<li class="flex items-start gap-2">
					<svg class="w-5 h-5 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
						<path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
					</svg>
					<span>This page intentionally delays server response by 3 seconds</span>
				</li>
				<li class="flex items-start gap-2">
					<svg class="w-5 h-5 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
						<path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
					</svg>
					<span>Client-side JavaScript adds additional loading simulation</span>
				</li>
				<li class="flex items-start gap-2">
					<svg class="w-5 h-5 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
						<path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
					</svg>
					<span>Use this page to test timeout handling and loading indicators</span>
				</li>
			</ul>
		</section>
	</div>
</div>
