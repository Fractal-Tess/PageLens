<svelte:head>
	<title>PageLens Test App - Home</title>
	<meta name="description" content="Welcome to the PageLens Test Application - a comprehensive Svelte app for testing web analysis tools." />
	<meta name="keywords" content="pagelens, test, svelte, seo, web analysis" />
	<meta name="author" content="PageLens Team" />
	<meta property="og:title" content="PageLens Test App - Home" />
	<meta property="og:description" content="A comprehensive Svelte application for testing PageLens core functionality." />
	<meta property="og:type" content="website" />
	<meta property="og:url" content="https://example.com/" />
	<meta property="og:image" content="https://example.com/og-image.jpg" />
	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content="PageLens Test App" />
	<meta name="twitter:description" content="Test your web analysis tools with this complex Svelte application." />
	<link rel="canonical" href="https://example.com/" />
</svelte:head>

<!-- Hero Section -->
<section class="relative bg-indigo-600 text-white overflow-hidden">
	<div class="absolute inset-0">
		<div class="absolute inset-0 bg-gradient-to-r from-indigo-600 to-purple-600 opacity-90"></div>
		<!-- Decorative circles -->
		<div class="absolute top-0 left-0 w-72 h-72 bg-white opacity-10 rounded-full -translate-x-1/2 -translate-y-1/2"></div>
		<div class="absolute bottom-0 right-0 w-96 h-96 bg-white opacity-10 rounded-full translate-x-1/3 translate-y-1/3"></div>
	</div>
	<div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-24">
		<div class="text-center">
			<h1 class="text-4xl md:text-6xl font-bold mb-6">
				Welcome to <span class="text-yellow-300">PageLens</span> Test App
			</h1>
			<p class="text-xl md:text-2xl mb-8 text-indigo-100 max-w-3xl mx-auto">
				A complex Svelte application designed for testing web analysis tools, SEO extraction, and page performance metrics.
			</p>
			<div class="flex flex-col sm:flex-row gap-4 justify-center">
				<a href="/products" class="inline-flex items-center justify-center px-8 py-3 border border-transparent text-base font-medium rounded-md text-indigo-600 bg-white hover:bg-indigo-50 md:text-lg transition-colors">
					Explore Features
				</a>
				<a href="/slow-page" class="inline-flex items-center justify-center px-8 py-3 border border-white text-base font-medium rounded-md text-white hover:bg-white hover:text-indigo-600 md:text-lg transition-colors">
					Test Slow Loading
				</a>
			</div>
		</div>
	</div>
</section>

<!-- Features Section -->
<section class="py-16 bg-white">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
		<div class="text-center mb-12">
			<h2 class="text-3xl font-bold text-gray-900">Test Features</h2>
			<p class="mt-4 text-lg text-gray-600">Explore the various features designed to test different aspects of web analysis.</p>
		</div>
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
			<!-- Feature Card 1 -->
			<article class="bg-gray-50 rounded-lg p-6 hover:shadow-lg transition-shadow">
				<div class="w-12 h-12 bg-indigo-100 rounded-lg flex items-center justify-center mb-4">
					<svg class="w-6 h-6 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
					</svg>
				</div>
				<h3 class="text-xl font-semibold text-gray-900 mb-2">SEO Testing</h3>
				<p class="text-gray-600 mb-4">Pages with comprehensive meta tags, Open Graph, Twitter Cards, and structured data.</p>
				<a href="/seo-test" class="text-indigo-600 font-medium hover:text-indigo-800">Learn more &rarr;</a>
			</article>

			<!-- Feature Card 2 -->
			<article class="bg-gray-50 rounded-lg p-6 hover:shadow-lg transition-shadow">
				<div class="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center mb-4">
					<svg class="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
					</svg>
				</div>
				<h3 class="text-xl font-semibold text-gray-900 mb-2">Image Maps</h3>
				<p class="text-gray-600 mb-4">Interactive image maps with clickable areas and complex coordinate systems.</p>
				<a href="/image-map" class="text-green-600 font-medium hover:text-green-800">Learn more &rarr;</a>
			</article>

			<!-- Feature Card 3 -->
			<article class="bg-gray-50 rounded-lg p-6 hover:shadow-lg transition-shadow">
				<div class="w-12 h-12 bg-yellow-100 rounded-lg flex items-center justify-center mb-4">
					<svg class="w-6 h-6 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
					</svg>
				</div>
				<h3 class="text-xl font-semibold text-gray-900 mb-2">Slow Loading</h3>
				<p class="text-gray-600 mb-4">Pages with artificial delays to test loading performance and timeout handling.</p>
				<a href="/slow-page" class="text-yellow-600 font-medium hover:text-yellow-800">Learn more &rarr;</a>
			</article>

			<!-- Feature Card 4 -->
			<article class="bg-gray-50 rounded-lg p-6 hover:shadow-lg transition-shadow">
				<div class="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center mb-4">
					<svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
					</svg>
				</div>
				<h3 class="text-xl font-semibold text-gray-900 mb-2">Complex Layouts</h3>
				<p class="text-gray-600 mb-4">Nested grids, flexbox layouts, and complex component hierarchies.</p>
				<a href="/complex-layout" class="text-purple-600 font-medium hover:text-purple-800">Learn more &rarr;</a>
			</article>

			<!-- Feature Card 5 -->
			<article class="bg-gray-50 rounded-lg p-6 hover:shadow-lg transition-shadow">
				<div class="w-12 h-12 bg-red-100 rounded-lg flex items-center justify-center mb-4">
					<svg class="w-6 h-6 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
					</svg>
				</div>
				<h3 class="text-xl font-semibold text-gray-900 mb-2">Rich Forms</h3>
				<p class="text-gray-600 mb-4">Various form elements, validation states, and input types.</p>
				<a href="/forms" class="text-red-600 font-medium hover:text-red-800">Learn more &rarr;</a>
			</article>

			<!-- Feature Card 6 -->
			<article class="bg-gray-50 rounded-lg p-6 hover:shadow-lg transition-shadow">
				<div class="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center mb-4">
					<svg class="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 11V7a4 4 0 00-8 0v4M5 9h14l1 12H4L5 9z" />
					</svg>
				</div>
				<h3 class="text-xl font-semibold text-gray-900 mb-2">Product Catalog</h3>
				<p class="text-gray-600 mb-4">E-commerce style product listings with filters and sorting.</p>
				<a href="/products" class="text-blue-600 font-medium hover:text-blue-800">Learn more &rarr;</a>
			</article>
		</div>
	</div>
</section>

<!-- Stats Section -->
<section class="py-16 bg-gray-900 text-white">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
		<div class="grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
			<div>
				<div class="text-4xl font-bold text-indigo-400">9+</div>
				<div class="mt-2 text-gray-400">Pages</div>
			</div>
			<div>
				<div class="text-4xl font-bold text-green-400">50+</div>
				<div class="mt-2 text-gray-400">Components</div>
			</div>
			<div>
				<div class="text-4xl font-bold text-yellow-400">100+</div>
				<div class="mt-2 text-gray-400">Test Elements</div>
			</div>
			<div>
				<div class="text-4xl font-bold text-purple-400">∞</div>
				<div class="mt-2 text-gray-400">Possibilities</div>
			</div>
		</div>
	</div>
</section>

<!-- CTA Section -->
<section class="py-16 bg-indigo-50">
	<div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
		<h2 class="text-3xl font-bold text-gray-900 mb-4">Ready to Test?</h2>
		<p class="text-lg text-gray-600 mb-8">
			Start exploring the application to test your web analysis tools. Each page is designed with specific features to challenge different aspects of page analysis.
		</p>
		<div class="flex flex-col sm:flex-row gap-4 justify-center">
			<a href="/blog" class="inline-flex items-center justify-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 transition-colors">
				Read the Blog
			</a>
			<a href="/seo-test" class="inline-flex items-center justify-center px-6 py-3 border border-gray-300 text-base font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 transition-colors">
				SEO Test Page
			</a>
		</div>
	</div>
</section>
