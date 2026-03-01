<svelte:head>
	<title>Blog - PageLens Test App</title>
	<meta name="description" content="Read the latest articles about web development, SEO, and testing strategies on the PageLens blog." />
	<meta name="keywords" content="blog, web development, seo, testing, articles" />
	<meta name="author" content="PageLens Team" />
	<meta property="og:title" content="Blog - PageLens Test App" />
	<meta property="og:description" content="Latest articles on web development and testing." />
	<meta property="og:type" content="website" />
	<meta property="og:url" content="https://example.com/blog" />
	<meta name="twitter:card" content="summary" />
	<link rel="canonical" href="https://example.com/blog" />

</svelte:head>

<script lang="ts">
	const posts = [
		{
			slug: 'getting-started-with-seo',
			title: 'Getting Started with SEO',
			excerpt: 'Learn the fundamentals of search engine optimization and how to structure your content for better visibility.',
			date: '2026-02-20',
			category: 'SEO',
			readTime: '5 min read',
			featured: true
		},
		{
			slug: 'modern-web-development',
			title: 'Modern Web Development Patterns',
			excerpt: 'Explore the latest patterns and best practices in building scalable web applications with Svelte and SvelteKit.',
			date: '2026-02-15',
			category: 'Development',
			readTime: '8 min read',
			featured: false
		},
		{
			slug: 'testing-web-applications',
			title: 'Comprehensive Testing Strategies',
			excerpt: 'A deep dive into testing methodologies for web applications including unit, integration, and e2e tests.',
			date: '2026-02-10',
			category: 'Testing',
			readTime: '12 min read',
			featured: false
		},
		{
			slug: 'accessibility-best-practices',
			title: 'Accessibility Best Practices',
			excerpt: 'Make your web applications accessible to everyone by following these WCAG guidelines and best practices.',
			date: '2026-02-05',
			category: 'Accessibility',
			readTime: '6 min read',
			featured: false
		},
		{
			slug: 'performance-optimization',
			title: 'Performance Optimization Techniques',
			excerpt: 'Learn how to optimize your web application for speed and improve Core Web Vitals scores.',
			date: '2026-02-01',
			category: 'Performance',
			readTime: '10 min read',
			featured: true
		},
		{
			slug: 'css-grid-flexbox',
			title: 'Mastering CSS Grid and Flexbox',
			excerpt: 'A comprehensive guide to modern CSS layout techniques with practical examples and use cases.',
			date: '2026-01-25',
			category: 'CSS',
			readTime: '7 min read',
			featured: false
		}
	];

	const categories = ['All', 'SEO', 'Development', 'Testing', 'Accessibility', 'Performance', 'CSS'];
	let selectedCategory = 'All';
	
	$: filteredPosts = selectedCategory === 'All' 
		? posts 
		: posts.filter(p => p.category === selectedCategory);
</script>

<div class="bg-gray-50 min-h-screen">
	<!-- Header -->
	<div class="bg-white shadow-sm">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
			<h1 class="text-4xl font-bold text-gray-900">Blog</h1>
			<p class="mt-4 text-lg text-gray-600 max-w-2xl">
				Insights, tutorials, and best practices for web development, SEO, and testing.
			</p>
		</div>
	</div>

	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
		<!-- Category Filter -->
		<div class="flex flex-wrap gap-2 mb-8">
			{#each categories as category}
				<button
					on:click={() => selectedCategory = category}
					class="px-4 py-2 rounded-full text-sm font-medium transition-colors {selectedCategory === category 
						? 'bg-indigo-600 text-white' 
						: 'bg-white text-gray-700 hover:bg-gray-100 border border-gray-300'}"
				>
					{category}
				</button>
			{/each}
		</div>

		<!-- Featured Posts (only show when All is selected) -->
		{#if selectedCategory === 'All'}
			<section class="mb-12" aria-label="Featured posts">
				<h2 class="text-2xl font-bold text-gray-900 mb-6">Featured Articles</h2>
				<div class="grid grid-cols-1 md:grid-cols-2 gap-8">
					{#each posts.filter(p => p.featured) as post}
						<article class="bg-white rounded-lg shadow-md overflow-hidden hover:shadow-lg transition-shadow">
							<div class="h-48 bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center">
								<span class="text-white text-6xl opacity-50">📝</span>
							</div>
							<div class="p-6">
								<div class="flex items-center gap-4 text-sm text-gray-500 mb-3">
									<span class="px-2 py-1 bg-indigo-100 text-indigo-700 rounded">{post.category}</span>
									<time datetime={post.date}>{new Date(post.date).toLocaleDateString('en-US', { 
										month: 'long', 
										day: 'numeric', 
										year: 'numeric' 
									})}</time>
									<span>{post.readTime}</span>
								</div>
								<h3 class="text-xl font-bold text-gray-900 mb-2">
									<span class="text-gray-900">
										{post.title}
									</span>
								</h3>
								<p class="text-gray-600 mb-4">{post.excerpt}</p>
								<a href="/blog" class="text-indigo-600 font-medium hover:text-indigo-800">
									Read more &rarr;
								</a>
							</div>
						</article>
					{/each}
				</div>
			</section>
		{/if}

		<!-- All Posts -->
		<section aria-label="Blog posts">
			<h2 class="text-2xl font-bold text-gray-900 mb-6">
				{selectedCategory === 'All' ? 'All Articles' : `${selectedCategory} Articles`}
			</h2>
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
				{#each filteredPosts as post}
					<article class="bg-white rounded-lg shadow-sm overflow-hidden hover:shadow-md transition-shadow border border-gray-200">
						<div class="p-6">
							<div class="flex items-center gap-3 text-sm text-gray-500 mb-3">
								<span class="px-2 py-1 bg-gray-100 text-gray-700 rounded text-xs">{post.category}</span>
								<time datetime={post.date}>{new Date(post.date).toLocaleDateString()}</time>
							</div>
							<h3 class="text-lg font-bold text-gray-900 mb-2">
								<a href="/blog" class="hover:text-indigo-600 transition-colors">
									{post.title}
								</a>
							</h3>
							<p class="text-gray-600 text-sm mb-4 line-clamp-2">{post.excerpt}</p>
							<div class="flex items-center justify-between">
								<span class="text-sm text-gray-500">{post.readTime}</span>
								<span class="text-indigo-600 text-sm font-medium">
									Read &rarr;
								</span>
							</div>
						</div>
					</article>
				{/each}
			</div>
		</section>

		<!-- Pagination -->
		<nav class="mt-12 flex justify-center" aria-label="Pagination">
			<div class="flex items-center gap-2">
				<button class="px-4 py-2 border border-gray-300 rounded-md text-gray-500 hover:bg-gray-50 disabled:opacity-50" disabled>
					Previous
				</button>
				<a href="/blog?page=1" class="px-4 py-2 bg-indigo-600 text-white rounded-md" aria-current="page">1</a>
				<a href="/blog?page=2" class="px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-50">2</a>
				<a href="/blog?page=3" class="px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-50">3</a>
				<span class="px-2 text-gray-500">...</span>
				<a href="/blog?page=2" class="px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-50">
					Next
				</a>
			</div>
		</nav>
	</div>
</div>
