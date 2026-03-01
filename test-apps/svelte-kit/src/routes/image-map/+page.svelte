<svelte:head>
	<title>Image Map - PageLens Test App</title>
	<meta name="description" content="Test HTML image maps with clickable areas. Features complex coordinate systems and interactive hotspots." />
	<meta name="keywords" content="image map, area tags, clickable areas, hotspots" />
	<link rel="canonical" href="https://example.com/image-map" />
</svelte:head>

<script lang="ts">
	let activeArea: string | null = null;
	
	const areas = [
		{ id: 'north-america', coords: '120,80,200,140', shape: 'rect', label: 'North America', info: 'Population: 592 million | Area: 24.7 million km2' },
		{ id: 'south-america', coords: '220,180,280,280', shape: 'rect', label: 'South America', info: 'Population: 430 million | Area: 17.8 million km2' },
		{ id: 'europe', coords: '380,70,430,110', shape: 'rect', label: 'Europe', info: 'Population: 746 million | Area: 10.1 million km2' },
		{ id: 'africa', coords: '360,140,440,260', shape: 'rect', label: 'Africa', info: 'Population: 1.3 billion | Area: 30.3 million km2' },
		{ id: 'asia', coords: '460,60,600,180', shape: 'rect', label: 'Asia', info: 'Population: 4.6 billion | Area: 44.5 million km2' },
		{ id: 'australia', coords: '520,240,600,320', shape: 'rect', label: 'Australia', info: 'Population: 26 million | Area: 7.6 million km2' },
	];

	const techAreas = [
		{ id: 'frontend', coords: '150,150,80', shape: 'circle', label: 'Frontend', tech: 'React, Vue, Svelte, Angular' },
		{ id: 'backend', coords: '350,150,80', shape: 'circle', label: 'Backend', tech: 'Node.js, Python, Go, Rust' },
		{ id: 'database', coords: '550,150,80', shape: 'circle', label: 'Database', tech: 'PostgreSQL, MongoDB, Redis' },
		{ id: 'devops', coords: '250,300,80', shape: 'circle', label: 'DevOps', tech: 'Docker, K8s, CI/CD, AWS' },
		{ id: 'mobile', coords: '450,300,80', shape: 'circle', label: 'Mobile', tech: 'iOS, Android, React Native' },
	];

	function handleAreaClick(area: typeof areas[0]) {
		activeArea = activeArea === area.id ? null : area.id;
	}
</script>

<div class="bg-gray-50 min-h-screen">
	<!-- Header -->
	<div class="bg-white shadow-sm">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
			<h1 class="text-3xl font-bold text-gray-900">HTML Image Maps</h1>
			<p class="mt-2 text-gray-600">
				Test pages with interactive image maps and clickable area elements. 
				Click on different regions to see how PageLens handles complex coordinate-based interactions.
			</p>
		</div>
	</div>

	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
		<!-- World Map Section -->
		<section class="mb-16" aria-labelledby="world-map-heading">
			<h2 id="world-map-heading" class="text-2xl font-bold text-gray-900 mb-6">Interactive World Map</h2>
			<div class="bg-white rounded-lg shadow-md p-6">
				<div class="flex flex-col lg:flex-row gap-8">
					<!-- Image Map -->
					<div class="flex-1">
						<div class="relative inline-block">
							<!-- SVG World Map -->
							<svg viewBox="0 0 700 400" class="w-full max-w-2xl rounded-lg border border-gray-200" aria-label="World map with clickable continents">
								<rect fill="#e0f2fe" width="700" height="400"/>
								<!-- North America -->
								<a href="#north-america" on:click|preventDefault={() => handleAreaClick(areas[0])}>
									<path fill={activeArea === 'north-america' ? '#ef4444' : '#fca5a5'} d="M80 50 L240 50 L240 160 L120 160 L80 120 Z" class="hover:opacity-80 transition-opacity cursor-pointer"/>
									<text x="140" y="110" font-family="Arial" font-size="14" fill="#7f1d1d">North America</text>
								</a>
								<!-- South America -->
								<a href="#south-america" on:click|preventDefault={() => handleAreaClick(areas[1])}>
									<path fill={activeArea === 'south-america' ? '#ef4444' : '#fdba74'} d="M200 170 L300 170 L280 300 L220 300 Z" class="hover:opacity-80 transition-opacity cursor-pointer"/>
									<text x="230" y="240" font-family="Arial" font-size="14" fill="#7c2d12">South America</text>
								</a>
								<!-- Europe -->
								<a href="#europe" on:click|preventDefault={() => handleAreaClick(areas[2])}>
									<rect fill={activeArea === 'europe' ? '#ef4444' : '#fcd34d'} x="360" y="50" width="90" height="70" class="hover:opacity-80 transition-opacity cursor-pointer"/>
									<text x="390" y="90" font-family="Arial" font-size="14" fill="#78350f">Europe</text>
								</a>
								<!-- Africa -->
								<a href="#africa" on:click|preventDefault={() => handleAreaClick(areas[3])}>
									<path fill={activeArea === 'africa' ? '#ef4444' : '#bef264'} d="M340 130 L460 130 L440 280 L360 280 Z" class="hover:opacity-80 transition-opacity cursor-pointer"/>
									<text x="380" y="210" font-family="Arial" font-size="14" fill="#3f6212">Africa</text>
								</a>
								<!-- Asia -->
								<a href="#asia" on:click|preventDefault={() => handleAreaClick(areas[4])}>
									<path fill={activeArea === 'asia' ? '#ef4444' : '#86efac'} d="M460 40 L640 40 L620 200 L470 200 Z" class="hover:opacity-80 transition-opacity cursor-pointer"/>
									<text x="530" y="120" font-family="Arial" font-size="14" fill="#14532d">Asia</text>
								</a>
								<!-- Australia -->
								<a href="#australia" on:click|preventDefault={() => handleAreaClick(areas[5])}>
									<ellipse fill={activeArea === 'australia' ? '#ef4444' : '#c4b5fd'} cx="560" cy="280" rx="60" ry="50" class="hover:opacity-80 transition-opacity cursor-pointer"/>
									<text x="560" y="285" text-anchor="middle" font-family="Arial" font-size="14" fill="#5b21b6">Australia</text>
								</a>
							</svg>
						</div>
						<p class="mt-4 text-sm text-gray-500">
							<em>Click on any continent to see details. This demonstrates interactive SVG areas.</em>
						</p>
					</div>

					<!-- Info Panel -->
					<div class="lg:w-80">
						<div class="bg-gray-50 rounded-lg p-6 border border-gray-200">
							<h3 class="text-lg font-semibold text-gray-900 mb-4">Region Information</h3>
							{#if activeArea}
								{@const area = areas.find(a => a.id === activeArea)}
								<div class="animate-fade-in">
									<h4 class="text-xl font-bold text-indigo-600 mb-2">{area?.label}</h4>
									<p class="text-gray-600">{area?.info}</p>
									<div class="mt-4 pt-4 border-t border-gray-200">
										<p class="text-sm text-gray-500">
											<strong>Area ID:</strong> {area?.id}<br/>
											<strong>Shape:</strong> {area?.shape}<br/>
											<strong>Coords:</strong> {area?.coords}
										</p>
									</div>
								</div>
							{:else}
								<p class="text-gray-500 italic">Click on a continent to view details.</p>
							{/if}
						</div>
					</div>
				</div>
			</div>
		</section>

		<!-- Tech Stack Map Section -->
		<section aria-labelledby="tech-map-heading">
			<h2 id="tech-map-heading" class="text-2xl font-bold text-gray-900 mb-6">Technology Stack Map</h2>
			<div class="bg-white rounded-lg shadow-md p-6">
				<div class="flex flex-col lg:flex-row gap-8">
					<!-- Circle Image Map -->
					<div class="flex-1">
						<div class="relative inline-block">
							<svg viewBox="0 0 700 400" class="w-full max-w-2xl rounded-lg border border-gray-200" aria-label="Technology stack diagram">
								<rect fill="#f3f4f6" width="700" height="400"/>
								<!-- Frontend -->
								<circle cx="150" cy="150" r="80" fill="#60a5fa" opacity="0.8"/>
								<text x="150" y="155" text-anchor="middle" font-family="Arial" font-size="16" fill="white" font-weight="bold">Frontend</text>
								<!-- Backend -->
								<circle cx="350" cy="150" r="80" fill="#a78bfa" opacity="0.8"/>
								<text x="350" y="155" text-anchor="middle" font-family="Arial" font-size="16" fill="white" font-weight="bold">Backend</text>
								<!-- Database -->
								<circle cx="550" cy="150" r="80" fill="#fbbf24" opacity="0.8"/>
								<text x="550" y="155" text-anchor="middle" font-family="Arial" font-size="16" fill="white" font-weight="bold">Database</text>
								<!-- DevOps -->
								<circle cx="250" cy="300" r="80" fill="#34d399" opacity="0.8"/>
								<text x="250" y="305" text-anchor="middle" font-family="Arial" font-size="16" fill="white" font-weight="bold">DevOps</text>
								<!-- Mobile -->
								<circle cx="450" cy="300" r="80" fill="#f87171" opacity="0.8"/>
								<text x="450" y="305" text-anchor="middle" font-family="Arial" font-size="16" fill="white" font-weight="bold">Mobile</text>
							</svg>
						</div>
						<p class="mt-4 text-sm text-gray-500">
							<em>This demonstrates circular area visualization with overlapping regions.</em>
						</p>
					</div>

					<!-- Tech Info -->
					<div class="lg:w-80">
						<div class="bg-gray-50 rounded-lg p-6 border border-gray-200">
							<h3 class="text-lg font-semibold text-gray-900 mb-4">Tech Stack Areas</h3>
							<ul class="space-y-3">
								{#each techAreas as area}
									<li class="flex items-center gap-3">
										<span class="w-3 h-3 rounded-full bg-indigo-500"></span>
										<div>
											<strong class="text-gray-900">{area.label}</strong>
											<p class="text-sm text-gray-500">{area.tech}</p>
										</div>
									</li>
								{/each}
							</ul>
						</div>
					</div>
				</div>
			</div>
		</section>

		<!-- Documentation -->
		<section class="mt-12" aria-labelledby="docs-heading">
			<h2 id="docs-heading" class="text-2xl font-bold text-gray-900 mb-6">Area Tag Reference</h2>
			<div class="bg-white rounded-lg shadow-md overflow-hidden">
				<table class="min-w-full divide-y divide-gray-200">
					<thead class="bg-gray-50">
						<tr>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Attribute</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Description</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Example</th>
						</tr>
					</thead>
					<tbody class="bg-white divide-y divide-gray-200">
						<tr>
							<td class="px-6 py-4 text-sm font-mono text-indigo-600">shape</td>
							<td class="px-6 py-4 text-sm text-gray-600">Defines the shape of the area</td>
							<td class="px-6 py-4 text-sm text-gray-900">rect, circle, poly, default</td>
						</tr>
						<tr>
							<td class="px-6 py-4 text-sm font-mono text-indigo-600">coords</td>
							<td class="px-6 py-4 text-sm text-gray-600">Coordinates defining the area</td>
							<td class="px-6 py-4 text-sm text-gray-900">x1,y1,x2,y2 (rect) or x,y,r (circle)</td>
						</tr>
						<tr>
							<td class="px-6 py-4 text-sm font-mono text-indigo-600">href</td>
							<td class="px-6 py-4 text-sm text-gray-600">Destination URL</td>
							<td class="px-6 py-4 text-sm text-gray-900">/page#section</td>
						</tr>
						<tr>
							<td class="px-6 py-4 text-sm font-mono text-indigo-600">alt</td>
							<td class="px-6 py-4 text-sm text-gray-600">Alternative text for accessibility</td>
							<td class="px-6 py-4 text-sm text-gray-900">North America</td>
						</tr>
						<tr>
							<td class="px-6 py-4 text-sm font-mono text-indigo-600">target</td>
							<td class="px-6 py-4 text-sm text-gray-600">Where to open the linked document</td>
							<td class="px-6 py-4 text-sm text-gray-900">_blank, _self, _parent, _top</td>
						</tr>
					</tbody>
				</table>
			</div>
		</section>
	</div>
</div>

<style>
	@keyframes fade-in {
		from { opacity: 0; transform: translateY(10px); }
		to { opacity: 1; transform: translateY(0); }
	}
	.animate-fade-in {
		animation: fade-in 0.3s ease-out;
	}
</style>
