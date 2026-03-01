<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';
	
	let { children } = $props();
	
	const navItems = [
		{ href: '/', label: 'Home' },
		{ href: '/blog', label: 'Blog' },
		{ href: '/products', label: 'Products' },
		{ href: '/image-map', label: 'Image Map' },
		{ href: '/complex-layout', label: 'Complex Layout' },
		{ href: '/forms', label: 'Forms' },
		{ href: '/seo-test', label: 'SEO Test' },
		{ href: '/slow-page', label: 'Slow Page' },
	];
</script>

<div class="min-h-screen bg-gray-50">
	<!-- Navigation -->
	<nav class="bg-white shadow-md sticky top-0 z-50">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="flex justify-between h-16">
				<div class="flex items-center">
					<a href="/" class="text-xl font-bold text-indigo-600">
						PageLens Test App
					</a>
				</div>
				<div class="hidden md:flex items-center space-x-1">
					{#each navItems as item}
						<a
							href={item.href}
							class="px-3 py-2 rounded-md text-sm font-medium transition-colors {page.url.pathname === item.href
								? 'bg-indigo-100 text-indigo-700'
								: 'text-gray-600 hover:bg-gray-100 hover:text-gray-900'}"
						>
							{item.label}
						</a>
					{/each}
				</div>
				<!-- Mobile menu button -->
				<div class="flex items-center md:hidden">
					<button
						type="button"
						class="inline-flex items-center justify-center p-2 rounded-md text-gray-400 hover:text-gray-500 hover:bg-gray-100"
						aria-controls="mobile-menu"
						aria-expanded="false"
					>
						<span class="sr-only">Open main menu</span>
						<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
						</svg>
					</button>
				</div>
			</div>
		</div>
	</nav>

	<!-- Main content -->
	<main>
		{@render children()}
	</main>

	<!-- Footer -->
	<footer class="bg-gray-800 text-white mt-12">
		<div class="max-w-7xl mx-auto py-8 px-4 sm:px-6 lg:px-8">
			<div class="grid grid-cols-1 md:grid-cols-4 gap-8">
				<div>
					<h3 class="text-lg font-semibold mb-4">PageLens Test App</h3>
					<p class="text-gray-400 text-sm">
						A complex Svelte application for testing PageLens core functionality.
					</p>
				</div>
				<div>
					<h4 class="text-md font-medium mb-4">Quick Links</h4>
					<ul class="space-y-2 text-sm text-gray-400">
						<li><a href="/" class="hover:text-white">Home</a></li>
						<li><a href="/blog" class="hover:text-white">Blog</a></li>
						<li><a href="/products" class="hover:text-white">Products</a></li>
					</ul>
				</div>
				<div>
					<h4 class="text-md font-medium mb-4">Features</h4>
					<ul class="space-y-2 text-sm text-gray-400">
						<li><a href="/forms" class="hover:text-white">Forms</a></li>
						<li><a href="/image-map" class="hover:text-white">Image Map</a></li>
						<li><a href="/seo-test" class="hover:text-white">SEO Test</a></li>
					</ul>
				</div>
				<div>
					<h4 class="text-md font-medium mb-4">Connect</h4>
					<div class="flex space-x-4">
						<a href="https://twitter.com" class="text-gray-400 hover:text-white">
							<span class="sr-only">Twitter</span>
							<svg class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24">
								<path d="M8.29 20.251c7.547 0 11.675-6.253 11.675-11.675 0-.178 0-.355-.012-.53A8.348 8.348 0 0022 5.92a8.19 8.19 0 01-2.357.646 4.118 4.118 0 001.804-2.27 8.224 8.224 0 01-2.605.996 4.107 4.107 0 00-6.993 3.743 11.65 11.65 0 01-8.457-4.287 4.106 4.106 0 001.27 5.477A4.072 4.072 0 012.8 9.713v.052a4.105 4.105 0 003.292 4.022 4.095 4.095 0 01-1.853.07 4.108 4.108 0 003.834 2.85A8.233 8.233 0 012 18.407a11.616 11.616 0 006.29 1.84" />
							</svg>
						</a>
						<a href="https://github.com" class="text-gray-400 hover:text-white">
							<span class="sr-only">GitHub</span>
							<svg class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24">
								<path fill-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clip-rule="evenodd" />
							</svg>
						</a>
					</div>
				</div>
			</div>
			<div class="mt-8 pt-8 border-t border-gray-700 text-center text-sm text-gray-400">
				<p>&copy; 2026 PageLens Test App. All rights reserved.</p>
			</div>
		</div>
	</footer>
</div>
