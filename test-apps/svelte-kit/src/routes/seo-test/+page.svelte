<svelte:head>
	<!-- Primary Meta Tags -->
	<title>SEO Test Page - PageLens | Comprehensive SEO Testing Ground</title>
	<meta name="title" content="SEO Test Page - PageLens | Comprehensive SEO Testing Ground" />
	<meta name="description" content="This is a comprehensive SEO test page with full meta tag coverage including Open Graph, Twitter Cards, JSON-LD structured data, and more. Designed for testing web crawlers and SEO analysis tools." />
	<meta name="keywords" content="seo, test, meta tags, open graph, twitter cards, structured data, json-ld, pagelens, web crawler" />
	<meta name="author" content="PageLens Development Team" />
	<meta name="robots" content="index, follow, max-snippet:-1, max-image-preview:large, max-video-preview:-1" />
	<meta name="googlebot" content="index, follow" />
	<meta name="bingbot" content="index, follow" />
	<meta name="viewport" content="width=device-width, initial-scale=1.0" />
	<meta name="theme-color" content="#4f46e5" />
	<meta name="msapplication-TileColor" content="#4f46e5" />
	<meta name="apple-mobile-web-app-capable" content="yes" />
	<meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
	<meta name="apple-mobile-web-app-title" content="PageLens SEO Test" />
	<meta name="application-name" content="PageLens SEO Test" />
	<meta name="format-detection" content="telephone=no" />
	<meta name="generator" content="SvelteKit" />
	<meta http-equiv="X-UA-Compatible" content="IE=edge" />
	
	<!-- Canonical URL -->
	<link rel="canonical" href="https://example.com/seo-test" />
	
	<!-- Alternate Languages -->
	<link rel="alternate" hreflang="en" href="https://example.com/seo-test" />
	<link rel="alternate" hreflang="es" href="https://example.com/es/seo-test" />
	<link rel="alternate" hreflang="fr" href="https://example.com/fr/seo-test" />
	<link rel="alternate" hreflang="x-default" href="https://example.com/seo-test" />
	
	<!-- Open Graph / Facebook -->
	<meta property="og:type" content="article" />
	<meta property="og:url" content="https://example.com/seo-test" />
	<meta property="og:title" content="SEO Test Page - PageLens | Comprehensive SEO Testing Ground" />
	<meta property="og:description" content="This is a comprehensive SEO test page with full meta tag coverage including Open Graph, Twitter Cards, JSON-LD structured data, and more." />
	<meta property="og:site_name" content="PageLens Test App" />
	<meta property="og:locale" content="en_US" />
	<meta property="article:published_time" content="2026-02-28T00:00:00+00:00" />
	<meta property="article:modified_time" content="2026-02-28T12:00:00+00:00" />
	<meta property="article:author" content="https://example.com/about" />
	<meta property="article:section" content="SEO" />
	<meta property="article:tag" content="SEO" />
	<meta property="article:tag" content="Testing" />
	<meta property="article:tag" content="Meta Tags" />
	
	<!-- Twitter -->
	<meta property="twitter:card" content="summary_large_image" />
	<meta property="twitter:url" content="https://example.com/seo-test" />
	<meta property="twitter:title" content="SEO Test Page - PageLens | Comprehensive SEO Testing Ground" />
	<meta property="twitter:description" content="This is a comprehensive SEO test page with full meta tag coverage including Open Graph, Twitter Cards, JSON-LD structured data, and more." />
	<meta property="twitter:site" content="@pagelens" />
	<meta property="twitter:creator" content="@pagelens" />
	
	<!-- Pinterest -->
	<meta name="pinterest" content="nopin" />
	
	<!-- JSON-LD Structured Data: Article -->
	<script type="application/ld+json">
	{
		"@context": "https://schema.org",
		"@type": "TechArticle",
		"headline": "SEO Test Page - Comprehensive SEO Testing Ground",
		"description": "A comprehensive test page for SEO analysis tools with complete meta tag coverage.",
		"image": "https://example.com/images/seo-test-og.jpg",
		"author": {
			"@type": "Organization",
			"name": "PageLens Development Team",
			"url": "https://example.com/about"
		},
		"publisher": {
			"@type": "Organization",
			"name": "PageLens",
			"logo": {
				"@type": "ImageObject",
				"url": "https://example.com/logo.png"
			}
		},
		"datePublished": "2026-02-28T00:00:00+00:00",
		"dateModified": "2026-02-28T12:00:00+00:00",
		"mainEntityOfPage": {
			"@type": "WebPage",
			"@id": "https://example.com/seo-test"
		},
		"keywords": "SEO, meta tags, Open Graph, Twitter Cards, structured data, JSON-LD",
		"articleSection": "SEO Testing",
		"proficiencyLevel": "Expert"
	}
	</script>
	
	<!-- JSON-LD Structured Data: BreadcrumbList -->
	<script type="application/ld+json">
	{
		"@context": "https://schema.org",
		"@type": "BreadcrumbList",
		"itemListElement": [
			{
				"@type": "ListItem",
				"position": 1,
				"name": "Home",
				"item": "https://example.com/"
			},
			{
				"@type": "ListItem",
				"position": 2,
				"name": "SEO Test",
				"item": "https://example.com/seo-test"
			}
		]
	}
	</script>
	
	<!-- JSON-LD Structured Data: WebSite -->
	<script type="application/ld+json">
	{
		"@context": "https://schema.org",
		"@type": "WebSite",
		"name": "PageLens Test App",
		"url": "https://example.com",
		"potentialAction": {
			"@type": "SearchAction",
			"target": "https://example.com/search?q={search_term_string}",
			"query-input": "required name=search_term_string"
		}
	}
	</script>
	
	<!-- JSON-LD Structured Data: Organization -->
	<script type="application/ld+json">
	{
		"@context": "https://schema.org",
		"@type": "Organization",
		"name": "PageLens",
		"url": "https://example.com",
		"logo": "https://example.com/logo.png",
		"sameAs": [
			"https://twitter.com/pagelens",
			"https://github.com/pagelens",
			"https://linkedin.com/company/pagelens"
		],
		"contactPoint": {
			"@type": "ContactPoint",
			"telephone": "+1-555-123-4567",
			"contactType": "customer support",
			"availableLanguage": ["English", "Spanish"]
		}
	}
	</script>
</svelte:head>

<script lang="ts">
	const metaTags = [
		{ category: 'Basic Meta', tags: ['title', 'description', 'keywords', 'author', 'viewport', 'robots'] },
		{ category: 'Open Graph', tags: ['og:type', 'og:url', 'og:title', 'og:description', 'og:site_name', 'og:locale', 'article:published_time'] },
		{ category: 'Twitter Cards', tags: ['twitter:card', 'twitter:url', 'twitter:title', 'twitter:description', 'twitter:site', 'twitter:creator'] },
		{ category: 'Technical', tags: ['canonical', 'theme-color', 'generator', 'X-UA-Compatible'] },
	];

	const structuredDataTypes = [
		{ type: 'TechArticle', purpose: 'Main article content markup' },
		{ type: 'BreadcrumbList', purpose: 'Navigation breadcrumb structure' },
		{ type: 'WebSite', purpose: 'Site information and search' },
		{ type: 'Organization', purpose: 'Company/organization details' },
	];
</script>

<div class="bg-gray-50 min-h-screen">
	<!-- Header -->
	<div class="bg-gradient-to-r from-indigo-600 to-purple-600 py-16">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<h1 class="text-4xl font-bold text-white text-center">SEO Test Page</h1>
			<p class="mt-4 text-xl text-indigo-100 text-center max-w-3xl mx-auto">
				A comprehensive testing ground for SEO analysis tools with complete meta tag coverage
			</p>
		</div>
	</div>

	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
		<!-- Overview Cards -->
		<div class="grid grid-cols-1 md:grid-cols-4 gap-6 mb-12">
			<div class="bg-white rounded-lg shadow-md p-6 text-center">
				<div class="text-4xl font-bold text-indigo-600">30+</div>
				<div class="text-gray-600 mt-1">Meta Tags</div>
			</div>
			<div class="bg-white rounded-lg shadow-md p-6 text-center">
				<div class="text-4xl font-bold text-green-600">4</div>
				<div class="text-gray-600 mt-1">Structured Data Types</div>
			</div>
			<div class="bg-white rounded-lg shadow-md p-6 text-center">
				<div class="text-4xl font-bold text-purple-600">3</div>
				<div class="text-gray-600 mt-1">Hreflang Versions</div>
			</div>
			<div class="bg-white rounded-lg shadow-md p-6 text-center">
				<div class="text-4xl font-bold text-orange-600">100%</div>
				<div class="text-gray-600 mt-1">SEO Coverage</div>
			</div>
		</div>

		<!-- Meta Tags Section -->
		<section class="mb-12" aria-labelledby="meta-tags-heading">
			<h2 id="meta-tags-heading" class="text-2xl font-bold text-gray-900 mb-6">Meta Tags Implemented</h2>
			<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
				{#each metaTags as group}
					<div class="bg-white rounded-lg shadow-md p-6">
						<h3 class="text-lg font-semibold text-gray-900 mb-4 flex items-center gap-2">
							<span class="w-2 h-2 bg-indigo-500 rounded-full"></span>
							{group.category}
						</h3>
						<ul class="space-y-2">
							{#each group.tags as tag}
								<li class="flex items-center gap-2 text-sm">
									<svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
										<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
									</svg>
									<code class="bg-gray-100 px-2 py-0.5 rounded text-indigo-600">{tag}</code>
								</li>
							{/each}
						</ul>
					</div>
				{/each}
			</div>
		</section>

		<!-- Structured Data Section -->
		<section class="mb-12" aria-labelledby="structured-data-heading">
			<h2 id="structured-data-heading" class="text-2xl font-bold text-gray-900 mb-6">JSON-LD Structured Data</h2>
			<div class="bg-white rounded-lg shadow-md overflow-hidden">
				<table class="min-w-full divide-y divide-gray-200">
					<thead class="bg-gray-50">
						<tr>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Schema Type</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Purpose</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
						</tr>
					</thead>
					<tbody class="bg-white divide-y divide-gray-200">
						{#each structuredDataTypes as item}
							<tr>
								<td class="px-6 py-4 text-sm font-mono text-indigo-600">{item.type}</td>
								<td class="px-6 py-4 text-sm text-gray-900">{item.purpose}</td>
								<td class="px-6 py-4">
									<span class="px-2 py-1 text-xs font-medium bg-green-100 text-green-800 rounded-full">Active</span>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</section>

		<!-- Heading Structure Test -->
		<section class="mb-12" aria-labelledby="headings-heading">
			<h2 id="headings-heading" class="text-2xl font-bold text-gray-900 mb-6">Heading Hierarchy Test</h2>
			<div class="bg-white rounded-lg shadow-md p-6">
				<h1 class="text-3xl font-bold text-gray-900 mb-4">H1: Main Page Title</h1>
				<p class="text-gray-600 mb-6">This demonstrates proper heading hierarchy structure.</p>
				
				<h2 class="text-2xl font-bold text-gray-800 mb-3">H2: Section Heading</h2>
				<p class="text-gray-600 mb-4">Subsection content goes here.</p>
				
				<h3 class="text-xl font-bold text-gray-700 mb-2">H3: Subsection Heading</h3>
				<p class="text-gray-600 mb-4">Nested content description.</p>
				
				<h4 class="text-lg font-bold text-gray-600 mb-2">H4: Detailed Heading</h4>
				<p class="text-gray-600 mb-4">More detailed information.</p>
				
				<h5 class="text-base font-bold text-gray-500 mb-2">H5: Minor Heading</h5>
				<p class="text-gray-600 mb-4">Minor section details.</p>
				
				<h6 class="text-sm font-bold text-gray-400 mb-2">H6: Smallest Heading</h6>
				<p class="text-gray-600">Final level of hierarchy.</p>
			</div>
		</section>

		<!-- Image SEO Test -->
		<section class="mb-12" aria-labelledby="images-heading">
			<h2 id="images-heading" class="text-2xl font-bold text-gray-900 mb-6">Image SEO Test</h2>
			<div class="bg-white rounded-lg shadow-md p-6">
				<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
					<figure>
						<div class="h-48 bg-gradient-to-br from-indigo-400 to-purple-500 rounded-lg flex items-center justify-center">
							<span class="text-white text-4xl">🖼️</span>
						</div>
						<figcaption class="mt-2 text-sm text-gray-600">
							<strong>Image with alt text:</strong> Decorative gradient placeholder
						</figcaption>
					</figure>
					<figure>
						<div class="h-48 bg-gradient-to-br from-green-400 to-teal-500 rounded-lg flex items-center justify-center">
							<span class="text-white text-4xl">📊</span>
						</div>
						<figcaption class="mt-2 text-sm text-gray-600">
							<strong>Another image:</strong> Analytics visualization example
						</figcaption>
					</figure>
					<figure>
						<div class="h-48 bg-gradient-to-br from-orange-400 to-red-500 rounded-lg flex items-center justify-center">
							<span class="text-white text-4xl">🎯</span>
						</div>
						<figcaption class="mt-2 text-sm text-gray-600">
							<strong>Third image:</strong> Target and goals illustration
						</figcaption>
					</figure>
				</div>
			</div>
		</section>

		<!-- Link Types -->
		<section class="mb-12" aria-labelledby="links-heading">
			<h2 id="links-heading" class="text-2xl font-bold text-gray-900 mb-6">Link Relationship Types</h2>
			<div class="bg-white rounded-lg shadow-md p-6">
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					<div class="p-4 bg-gray-50 rounded-lg">
						<code class="text-indigo-600 font-mono">rel="canonical"</code>
						<p class="text-sm text-gray-600 mt-1">Specifies the preferred URL for this content</p>
					</div>
					<div class="p-4 bg-gray-50 rounded-lg">
						<code class="text-indigo-600 font-mono">rel="alternate" hreflang</code>
						<p class="text-sm text-gray-600 mt-1">Links to language variants of this page</p>
					</div>
					<div class="p-4 bg-gray-50 rounded-lg">
						<code class="text-indigo-600 font-mono">rel="next" / rel="prev"</code>
						<p class="text-sm text-gray-600 mt-1">Pagination relationship markers</p>
					</div>
					<div class="p-4 bg-gray-50 rounded-lg">
						<code class="text-indigo-600 font-mono">rel="nofollow"</code>
						<p class="text-sm text-gray-600 mt-1">Indicates search engines should not follow this link</p>
					</div>
				</div>
			</div>
		</section>

		<!-- Testing Recommendations -->
		<section aria-labelledby="recommendations-heading">
			<h2 id="recommendations-heading" class="text-2xl font-bold text-gray-900 mb-6">Testing Checklist</h2>
			<div class="bg-white rounded-lg shadow-md p-6">
				<ul class="space-y-3">
					<li class="flex items-start gap-3">
						<input type="checkbox" checked disabled class="mt-1 w-5 h-5 text-indigo-600 rounded" />
						<span class="text-gray-700">All meta tags are properly formatted and accessible</span>
					</li>
					<li class="flex items-start gap-3">
						<input type="checkbox" checked disabled class="mt-1 w-5 h-5 text-indigo-600 rounded" />
						<span class="text-gray-700">JSON-LD structured data validates without errors</span>
					</li>
					<li class="flex items-start gap-3">
						<input type="checkbox" checked disabled class="mt-1 w-5 h-5 text-indigo-600 rounded" />
						<span class="text-gray-700">Canonical URL and hreflang tags are correct</span>
					</li>
					<li class="flex items-start gap-3">
						<input type="checkbox" checked disabled class="mt-1 w-5 h-5 text-indigo-600 rounded" />
						<span class="text-gray-700">Heading hierarchy follows logical structure (H1 → H6)</span>
					</li>
					<li class="flex items-start gap-3">
						<input type="checkbox" checked disabled class="mt-1 w-5 h-5 text-indigo-600 rounded" />
						<span class="text-gray-700">Open Graph and Twitter Card tags are complete</span>
					</li>
					<li class="flex items-start gap-3">
						<input type="checkbox" checked disabled class="mt-1 w-5 h-5 text-indigo-600 rounded" />
						<span class="text-gray-700">Robots meta directives are properly configured</span>
					</li>
				</ul>
			</div>
		</section>
	</div>
</div>
