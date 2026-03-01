<svelte:head>
	<title>Complex Layout - PageLens Test App</title>
	<meta name="description" content="Test complex nested layouts with grids, flexbox, and deeply nested components." />
	<meta name="keywords" content="layout, grid, flexbox, nested components" />
	<link rel="canonical" href="https://example.com/complex-layout" />
</svelte:head>

<script lang="ts">
	let sidebarOpen = true;
	let activeTab = 'overview';
	
	const tabs = [
		{ id: 'overview', label: 'Overview', icon: '📊' },
		{ id: 'analytics', label: 'Analytics', icon: '📈' },
		{ id: 'reports', label: 'Reports', icon: '📄' },
		{ id: 'settings', label: 'Settings', icon: '⚙️' },
	];

	const dashboardCards = [
		{ title: 'Total Revenue', value: '$54,230', change: '+12.5%', positive: true },
		{ title: 'Active Users', value: '2,345', change: '+8.2%', positive: true },
		{ title: 'Bounce Rate', value: '42.3%', change: '-3.1%', positive: true },
		{ title: 'Avg. Session', value: '4m 32s', change: '-1.2%', positive: false },
	];
</script>

<div class="min-h-screen bg-gray-100">
	<!-- Dashboard Header -->
	<header class="bg-white shadow-sm border-b border-gray-200">
		<div class="px-4 sm:px-6 lg:px-8">
			<div class="flex items-center justify-between h-16">
				<div class="flex items-center gap-4">
					<button 
						on:click={() => sidebarOpen = !sidebarOpen}
						class="p-2 rounded-md text-gray-400 hover:text-gray-500 hover:bg-gray-100"
						aria-label="Toggle sidebar"
					>
						<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
						</svg>
					</button>
					<h1 class="text-xl font-semibold text-gray-900">Dashboard</h1>
				</div>
				<div class="flex items-center gap-4">
					<button class="p-2 rounded-md text-gray-400 hover:text-gray-500 hover:bg-gray-100 relative" aria-label="Notifications">
						<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
						</svg>
						<span class="absolute top-1 right-1 w-2 h-2 bg-red-500 rounded-full"></span>
					</button>
					<div class="flex items-center gap-3">
						<div class="w-8 h-8 bg-indigo-500 rounded-full flex items-center justify-center text-white font-medium">
							JD
						</div>
						<span class="hidden md:block text-sm font-medium text-gray-700">John Doe</span>
					</div>
				</div>
			</div>
		</div>
	</header>

	<div class="flex">
		<!-- Sidebar -->
		<aside class="{sidebarOpen ? 'w-64' : 'w-0'} transition-all duration-300 bg-white shadow-md overflow-hidden">
			<nav class="p-4 space-y-1" aria-label="Sidebar navigation">
				<a href="/complex-layout" class="flex items-center gap-3 px-3 py-2 rounded-md bg-indigo-50 text-indigo-700 font-medium">
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
					</svg>
					Dashboard
				</a>
				<a href="/products" class="flex items-center gap-3 px-3 py-2 rounded-md text-gray-600 hover:bg-gray-50">
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 11V7a4 4 0 00-8 0v4M5 9h14l1 12H4L5 9z" />
					</svg>
					Products
				</a>
				<a href="/forms" class="flex items-center gap-3 px-3 py-2 rounded-md text-gray-600 hover:bg-gray-50">
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
					</svg>
					Orders
				</a>
				<a href="/blog" class="flex items-center gap-3 px-3 py-2 rounded-md text-gray-600 hover:bg-gray-50">
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
					</svg>
					Customers
				</a>
			</nav>
		</aside>

		<!-- Main Content -->
		<main class="flex-1 p-4 sm:p-6 lg:p-8">
			<!-- Tabs -->
			<div class="border-b border-gray-200 mb-6">
				<nav class="-mb-px flex space-x-8" aria-label="Tabs">
					{#each tabs as tab}
						<button
							on:click={() => activeTab = tab.id}
							class="{activeTab === tab.id 
								? 'border-indigo-500 text-indigo-600' 
								: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} 
								whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm flex items-center gap-2"
							aria-current={activeTab === tab.id ? 'page' : undefined}
						>
							<span>{tab.icon}</span>
							{tab.label}
						</button>
					{/each}
				</nav>
			</div>

			<!-- Stats Cards -->
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
				{#each dashboardCards as card}
					<div class="bg-white rounded-lg shadow-sm p-6 border border-gray-200">
						<div class="flex items-center justify-between">
							<div>
								<p class="text-sm font-medium text-gray-500">{card.title}</p>
								<p class="text-2xl font-bold text-gray-900 mt-1">{card.value}</p>
							</div>
							<div class="text-right">
								<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {card.positive ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}">
									{card.change}
								</span>
							</div>
						</div>
					</div>
				{/each}
			</div>

			<!-- Complex Nested Grid Layout -->
			<div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
				<!-- Main Content Area -->
				<div class="lg:col-span-2 space-y-6">
					<!-- Chart Section -->
					<div class="bg-white rounded-lg shadow-sm p-6 border border-gray-200">
						<h3 class="text-lg font-semibold text-gray-900 mb-4">Revenue Overview</h3>
						<div class="h-64 bg-gradient-to-b from-indigo-50 to-white rounded-lg flex items-end justify-around p-4">
							{#each [40, 65, 45, 80, 55, 70, 90, 60, 75, 50, 85, 95] as height, i}
								<div class="flex flex-col items-center gap-2">
									<div 
										class="w-8 bg-indigo-500 rounded-t-md hover:bg-indigo-600 transition-colors"
										style="height: {height * 2}px"
									></div>
									<span class="text-xs text-gray-500">{['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'][i]}</span>
								</div>
							{/each}
						</div>
					</div>

					<!-- Nested Grid -->
					<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
						<div class="bg-white rounded-lg shadow-sm p-4 border border-gray-200">
							<h4 class="font-medium text-gray-900 mb-3">Top Products</h4>
							<div class="space-y-3">
								{#each ['Wireless Headphones', 'Smart Watch', 'Running Shoes', 'Yoga Mat'] as product, i}
									<div class="flex items-center justify-between">
										<div class="flex items-center gap-3">
											<span class="w-6 h-6 rounded-full bg-gray-100 flex items-center justify-center text-xs font-medium text-gray-600">{i + 1}</span>
											<span class="text-sm text-gray-700">{product}</span>
										</div>
										<span class="text-sm font-medium text-gray-900">${(1000 - i * 200).toLocaleString()}</span>
									</div>
								{/each}
							</div>
						</div>
						<div class="bg-white rounded-lg shadow-sm p-4 border border-gray-200">
							<h4 class="font-medium text-gray-900 mb-3">Recent Activity</h4>
							<div class="space-y-3">
								{#each ['New order received', 'Payment processed', 'Product restocked', 'Review submitted'] as activity, i}
									<div class="flex items-start gap-3">
										<div class="w-2 h-2 mt-1.5 rounded-full bg-indigo-500"></div>
										<div>
											<p class="text-sm text-gray-700">{activity}</p>
											<p class="text-xs text-gray-500">{i + 1} hour{i !== 0 ? 's' : ''} ago</p>
										</div>
									</div>
								{/each}
							</div>
						</div>
					</div>
				</div>

				<!-- Sidebar Content -->
				<div class="space-y-6">
					<!-- Progress Widget -->
					<div class="bg-white rounded-lg shadow-sm p-6 border border-gray-200">
						<h3 class="text-lg font-semibold text-gray-900 mb-4">Goals Progress</h3>
						<div class="space-y-4">
							{#each [
								{ label: 'Monthly Revenue', progress: 75 },
								{ label: 'New Customers', progress: 60 },
								{ label: 'Product Sales', progress: 90 },
								{ label: 'Customer Satisfaction', progress: 85 },
							] as goal}
								<div>
									<div class="flex justify-between text-sm mb-1">
										<span class="text-gray-700">{goal.label}</span>
										<span class="text-gray-900 font-medium">{goal.progress}%</span>
									</div>
									<div class="w-full bg-gray-200 rounded-full h-2">
										<div class="bg-indigo-600 h-2 rounded-full" style="width: {goal.progress}%"></div>
									</div>
								</div>
							{/each}
						</div>
					</div>

					<!-- Team Widget -->
					<div class="bg-white rounded-lg shadow-sm p-6 border border-gray-200">
						<h3 class="text-lg font-semibold text-gray-900 mb-4">Team Members</h3>
						<div class="space-y-3">
							{#each [
								{ name: 'Alice Johnson', role: 'Developer', status: 'online' },
								{ name: 'Bob Smith', role: 'Designer', status: 'away' },
								{ name: 'Carol White', role: 'Manager', status: 'offline' },
							] as member}
								<div class="flex items-center gap-3">
									<div class="w-10 h-10 rounded-full bg-gradient-to-br from-indigo-400 to-purple-500 flex items-center justify-center text-white font-medium text-sm">
										{member.name.split(' ').map(n => n[0]).join('')}
									</div>
									<div class="flex-1">
										<p class="text-sm font-medium text-gray-900">{member.name}</p>
										<p class="text-xs text-gray-500">{member.role}</p>
									</div>
									<span class="w-2.5 h-2.5 rounded-full {member.status === 'online' ? 'bg-green-500' : member.status === 'away' ? 'bg-yellow-500' : 'bg-gray-400'}"></span>
								</div>
							{/each}
						</div>
					</div>
				</div>
			</div>

			<!-- Deeply Nested Component -->
			<div class="mt-6 bg-white rounded-lg shadow-sm p-6 border border-gray-200">
				<h3 class="text-lg font-semibold text-gray-900 mb-4">Nested Component Structure</h3>
				<div class="border border-gray-200 rounded-lg p-4">
					<div class="border border-dashed border-gray-300 rounded p-3 mb-3">
						<p class="text-sm font-medium text-gray-700 mb-2">Level 1 Container</p>
						<div class="border border-dashed border-gray-300 rounded p-3 mb-3 ml-4">
							<p class="text-sm font-medium text-gray-600 mb-2">Level 2 Wrapper</p>
							<div class="border border-dashed border-gray-300 rounded p-3 ml-4">
								<p class="text-sm font-medium text-gray-500 mb-2">Level 3 Content</p>
								<div class="grid grid-cols-3 gap-2 ml-4">
									{#each [1, 2, 3] as i}
										<div class="bg-indigo-50 rounded p-2 text-center">
											<p class="text-xs text-indigo-700">Item {i}</p>
										</div>
									{/each}
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		</main>
	</div>
</div>
