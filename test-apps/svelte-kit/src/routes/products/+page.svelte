<svelte:head>
	<title>Products - PageLens Test App</title>
	<meta name="description" content="Browse our collection of products. Test e-commerce style layouts, filters, and product cards." />
	<meta name="keywords" content="products, e-commerce, catalog, shop" />
	<meta property="og:title" content="Products - PageLens Test App" />
	<meta property="og:description" content="Browse our product catalog with filters and sorting." />
	<meta property="og:type" content="website" />
	<link rel="canonical" href="https://example.com/products" />
</svelte:head>

<script lang="ts">
	interface Product {
		id: number;
		name: string;
		price: number;
		category: string;
		rating: number;
		reviews: number;
		image: string;
		badge?: string;
		description: string;
		inStock: boolean;
	}

	const products: Product[] = [
		{ id: 1, name: 'Wireless Headphones Pro', price: 299.99, category: 'Electronics', rating: 4.8, reviews: 234, image: '🎧', badge: 'Best Seller', description: 'Premium noise-canceling wireless headphones with 30-hour battery life.', inStock: true },
		{ id: 2, name: 'Smart Watch Series 5', price: 399.99, category: 'Electronics', rating: 4.6, reviews: 189, image: '⌚', description: 'Advanced fitness tracking and health monitoring smartwatch.', inStock: true },
		{ id: 3, name: 'Organic Cotton T-Shirt', price: 29.99, category: 'Clothing', rating: 4.5, reviews: 456, image: '👕', badge: 'Eco-Friendly', description: '100% organic cotton t-shirt available in multiple colors.', inStock: true },
		{ id: 4, name: 'Running Shoes Elite', price: 149.99, category: 'Footwear', rating: 4.7, reviews: 312, image: '👟', description: 'Professional running shoes with advanced cushioning technology.', inStock: false },
		{ id: 5, name: 'Laptop Stand Adjustable', price: 79.99, category: 'Accessories', rating: 4.4, reviews: 128, image: '💻', description: 'Ergonomic aluminum laptop stand with adjustable height.', inStock: true },
		{ id: 6, name: 'Bluetooth Speaker Mini', price: 49.99, category: 'Electronics', rating: 4.3, reviews: 567, image: '🔊', description: 'Portable waterproof Bluetooth speaker with 360° sound.', inStock: true },
		{ id: 7, name: 'Yoga Mat Premium', price: 69.99, category: 'Fitness', rating: 4.9, reviews: 892, image: '🧘', badge: 'Top Rated', description: 'Extra thick non-slip yoga mat with carrying strap.', inStock: true },
		{ id: 8, name: 'Coffee Maker Deluxe', price: 199.99, category: 'Home', rating: 4.6, reviews: 234, image: '☕', description: 'Programmable coffee maker with thermal carafe.', inStock: true },
		{ id: 9, name: 'Backpack Travel Pro', price: 119.99, category: 'Accessories', rating: 4.5, reviews: 178, image: '🎒', description: 'Water-resistant travel backpack with laptop compartment.', inStock: true },
		{ id: 10, name: 'Desk Lamp LED', price: 39.99, category: 'Home', rating: 4.2, reviews: 445, image: '💡', description: 'Dimmable LED desk lamp with USB charging port.', inStock: true },
		{ id: 11, name: 'Fitness Tracker Band', price: 59.99, category: 'Fitness', rating: 4.4, reviews: 667, image: '💪', description: 'Affordable fitness tracker with heart rate monitor.', inStock: true },
		{ id: 12, name: 'Leather Wallet Slim', price: 45.99, category: 'Accessories', rating: 4.7, reviews: 234, image: '👛', description: 'Genuine leather slim wallet with RFID blocking.', inStock: false },
	];

	const categories = ['All', 'Electronics', 'Clothing', 'Footwear', 'Accessories', 'Fitness', 'Home'];
	const sortOptions = [
		{ value: 'featured', label: 'Featured' },
		{ value: 'price-low', label: 'Price: Low to High' },
		{ value: 'price-high', label: 'Price: High to Low' },
		{ value: 'rating', label: 'Highest Rated' },
		{ value: 'newest', label: 'Newest' }
	];

	let selectedCategory = 'All';
	let selectedSort = 'featured';
	let priceRange = 500;
	let showInStockOnly = false;
	let searchQuery = '';

	$: filteredProducts = products
		.filter(p => selectedCategory === 'All' || p.category === selectedCategory)
		.filter(p => p.price <= priceRange)
		.filter(p => !showInStockOnly || p.inStock)
		.filter(p => searchQuery === '' || p.name.toLowerCase().includes(searchQuery.toLowerCase()))
		.sort((a, b) => {
			switch (selectedSort) {
				case 'price-low': return a.price - b.price;
				case 'price-high': return b.price - a.price;
				case 'rating': return b.rating - a.rating;
				default: return a.id - b.id;
			}
		});

	function formatPrice(price: number): string {
		return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(price);
	}
</script>

<div class="bg-gray-50 min-h-screen">
	<!-- Header -->
	<div class="bg-white shadow-sm">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
			<h1 class="text-3xl font-bold text-gray-900">Products</h1>
			<p class="mt-2 text-gray-600">Browse our collection of {products.length} products</p>
		</div>
	</div>

	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
		<div class="flex flex-col lg:flex-row gap-8">
			<!-- Sidebar Filters -->
			<aside class="lg:w-64 flex-shrink-0">
				<div class="bg-white rounded-lg shadow-sm p-6 sticky top-24">
					<h2 class="text-lg font-semibold text-gray-900 mb-4">Filters</h2>
					
					<!-- Search -->
					<div class="mb-6">
						<label for="search" class="block text-sm font-medium text-gray-700 mb-2">Search</label>
						<input
							type="search"
							id="search"
							bind:value={searchQuery}
							placeholder="Search products..."
							class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500"
						/>
					</div>

					<!-- Categories -->
					<div class="mb-6">
						<h3 class="text-sm font-medium text-gray-700 mb-3">Category</h3>
						<div class="space-y-2">
							{#each categories as category}
								<label class="flex items-center cursor-pointer">
									<input
										type="radio"
										name="category"
										value={category}
										bind:group={selectedCategory}
										class="h-4 w-4 text-indigo-600 focus:ring-indigo-500"
									/>
									<span class="ml-2 text-sm text-gray-700">{category}</span>
								</label>
							{/each}
						</div>
					</div>

					<!-- Price Range -->
					<div class="mb-6">
						<h3 class="text-sm font-medium text-gray-700 mb-3">
							Max Price: {formatPrice(priceRange)}
						</h3>
						<input
							type="range"
							min="0"
							max="500"
							bind:value={priceRange}
							class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-indigo-600"
						/>
						<div class="flex justify-between text-xs text-gray-500 mt-1">
							<span>$0</span>
							<span>$500</span>
						</div>
					</div>

					<!-- In Stock -->
					<div class="mb-6">
						<label class="flex items-center cursor-pointer">
							<input
								type="checkbox"
								bind:checked={showInStockOnly}
								class="h-4 w-4 text-indigo-600 rounded focus:ring-indigo-500"
							/>
							<span class="ml-2 text-sm text-gray-700">In Stock Only</span>
						</label>
					</div>

					<button
						on:click={() => {
							selectedCategory = 'All';
							priceRange = 500;
							showInStockOnly = false;
							searchQuery = '';
						}}
						class="w-full py-2 px-4 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
					>
						Clear Filters
					</button>
				</div>
			</aside>

			<!-- Product Grid -->
			<div class="flex-1">
				<!-- Sort Bar -->
				<div class="bg-white rounded-lg shadow-sm p-4 mb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
					<p class="text-sm text-gray-600">
						Showing <strong>{filteredProducts.length}</strong> of <strong>{products.length}</strong> products
					</p>
					<div class="flex items-center gap-2">
						<label for="sort" class="text-sm text-gray-600">Sort by:</label>
						<select
							id="sort"
							bind:value={selectedSort}
							class="px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-indigo-500 focus:border-indigo-500"
						>
							{#each sortOptions as option}
								<option value={option.value}>{option.label}</option>
							{/each}
						</select>
					</div>
				</div>

				<!-- Products -->
				{#if filteredProducts.length > 0}
					<div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-6">
						{#each filteredProducts as product (product.id)}
							<article class="bg-white rounded-lg shadow-sm overflow-hidden hover:shadow-lg transition-shadow border border-gray-200 flex flex-col">
								<div class="relative h-48 bg-gradient-to-br from-gray-100 to-gray-200 flex items-center justify-center">
									<span class="text-6xl">{product.image}</span>
									{#if product.badge}
										<span class="absolute top-3 left-3 px-2 py-1 bg-indigo-600 text-white text-xs font-semibold rounded">
											{product.badge}
										</span>
									{/if}
									{#if !product.inStock}
										<span class="absolute top-3 right-3 px-2 py-1 bg-gray-800 text-white text-xs font-semibold rounded">
											Out of Stock
										</span>
									{/if}
								</div>
								<div class="p-4 flex-1 flex flex-col">
									<div class="flex items-start justify-between mb-2">
										<span class="text-xs font-medium text-indigo-600 uppercase tracking-wide">{product.category}</span>
										<div class="flex items-center gap-1">
											<svg class="w-4 h-4 text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
												<path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
											</svg>
											<span class="text-sm text-gray-600">{product.rating}</span>
											<span class="text-sm text-gray-400">({product.reviews})</span>
										</div>
									</div>
									<h3 class="text-lg font-semibold text-gray-900 mb-2">
										<span class="text-gray-900">
											{product.name}
										</span>
									</h3>
									<p class="text-sm text-gray-600 mb-4 flex-1">{product.description}</p>
									<div class="flex items-center justify-between pt-4 border-t border-gray-100">
										<span class="text-xl font-bold text-gray-900">{formatPrice(product.price)}</span>
										<button
											disabled={!product.inStock}
											class="px-4 py-2 bg-indigo-600 text-white text-sm font-medium rounded-md hover:bg-indigo-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
										>
											{product.inStock ? 'Add to Cart' : 'Out of Stock'}
										</button>
									</div>
								</div>
							</article>
						{/each}
					</div>
				{:else}
					<div class="text-center py-12">
						<div class="text-6xl mb-4">🔍</div>
						<h3 class="text-lg font-medium text-gray-900 mb-2">No products found</h3>
						<p class="text-gray-600">Try adjusting your filters or search query.</p>
					</div>
				{/if}
			</div>
		</div>
	</div>
</div>
