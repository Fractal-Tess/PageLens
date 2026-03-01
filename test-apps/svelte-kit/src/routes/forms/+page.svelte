<svelte:head>
	<title>Forms - PageLens Test App</title>
	<meta name="description" content="Test various form elements, validation states, and input types. Complete form testing suite." />
	<meta name="keywords" content="forms, inputs, validation, form elements" />
	<link rel="canonical" href="https://example.com/forms" />
</svelte:head>

<script lang="ts">
	let formData = {
		username: '',
		email: '',
		password: '',
		confirmPassword: '',
		bio: '',
		country: '',
		interests: [] as string[],
		newsletter: false,
		terms: false,
		rating: 3,
		birthdate: '',
		website: '',
		phone: '',
		color: '#4f46e5',
	};

	let errors: Record<string, string> = {};
	let submitted = false;

	function validateForm(): boolean {
		errors = {};
		
		if (!formData.username || formData.username.length < 3) {
			errors.username = 'Username must be at least 3 characters';
		}
		
		if (!formData.email || !formData.email.includes('@')) {
			errors.email = 'Please enter a valid email address';
		}
		
		if (!formData.password || formData.password.length < 8) {
			errors.password = 'Password must be at least 8 characters';
		}
		
		if (formData.password !== formData.confirmPassword) {
			errors.confirmPassword = 'Passwords do not match';
		}
		
		if (!formData.country) {
			errors.country = 'Please select a country';
		}
		
		if (!formData.terms) {
			errors.terms = 'You must accept the terms and conditions';
		}
		
		return Object.keys(errors).length === 0;
	}

	function handleSubmit(e: Event) {
		e.preventDefault();
		if (validateForm()) {
			submitted = true;
			setTimeout(() => submitted = false, 3000);
		}
	}

	const countries = [
		{ value: '', label: 'Select a country' },
		{ value: 'us', label: 'United States' },
		{ value: 'uk', label: 'United Kingdom' },
		{ value: 'ca', label: 'Canada' },
		{ value: 'au', label: 'Australia' },
		{ value: 'de', label: 'Germany' },
		{ value: 'fr', label: 'France' },
		{ value: 'jp', label: 'Japan' },
	];

	const interests = [
		{ value: 'tech', label: 'Technology' },
		{ value: 'design', label: 'Design' },
		{ value: 'business', label: 'Business' },
		{ value: 'marketing', label: 'Marketing' },
		{ value: 'science', label: 'Science' },
	];
</script>

<div class="bg-gray-50 min-h-screen">
	<!-- Header -->
	<div class="bg-white shadow-sm">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
			<h1 class="text-3xl font-bold text-gray-900">Form Elements</h1>
			<p class="mt-2 text-gray-600">Comprehensive testing of various form inputs, validation states, and user interactions.</p>
		</div>
	</div>

	<div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
		{#if submitted}
			<div class="mb-6 bg-green-50 border border-green-200 rounded-lg p-4 flex items-center gap-3">
				<svg class="w-5 h-5 text-green-500" fill="currentColor" viewBox="0 0 20 20">
					<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
				</svg>
				<p class="text-green-800 font-medium">Form submitted successfully!</p>
			</div>
		{/if}

		<form on:submit={handleSubmit} class="bg-white rounded-lg shadow-md p-8">
			<!-- Text Inputs -->
			<section class="mb-8" aria-labelledby="text-inputs-heading">
				<h2 id="text-inputs-heading" class="text-xl font-semibold text-gray-900 mb-4 pb-2 border-b border-gray-200">Text Inputs</h2>
				<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
					<div>
						<label for="username" class="block text-sm font-medium text-gray-700 mb-1">
							Username <span class="text-red-500">*</span>
						</label>
						<input
							type="text"
							id="username"
							bind:value={formData.username}
							class="w-full px-4 py-2 border {errors.username ? 'border-red-500' : 'border-gray-300'} rounded-md focus:ring-indigo-500 focus:border-indigo-500"
							placeholder="Enter username"
							aria-invalid={errors.username ? 'true' : 'false'}
							aria-describedby={errors.username ? 'username-error' : undefined}
						/>
						{#if errors.username}
							<p id="username-error" class="mt-1 text-sm text-red-600">{errors.username}</p>
						{/if}
					</div>

					<div>
						<label for="email" class="block text-sm font-medium text-gray-700 mb-1">
							Email <span class="text-red-500">*</span>
						</label>
						<input
							type="email"
							id="email"
							bind:value={formData.email}
							class="w-full px-4 py-2 border {errors.email ? 'border-red-500' : 'border-gray-300'} rounded-md focus:ring-indigo-500 focus:border-indigo-500"
							placeholder="you@example.com"
							aria-invalid={errors.email ? 'true' : 'false'}
						/>
						{#if errors.email}
							<p class="mt-1 text-sm text-red-600">{errors.email}</p>
						{/if}
					</div>
				</div>
			</section>

			<!-- Password Inputs -->
			<section class="mb-8" aria-labelledby="password-heading">
				<h2 id="password-heading" class="text-xl font-semibold text-gray-900 mb-4 pb-2 border-b border-gray-200">Password</h2>
				<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
					<div>
						<label for="password" class="block text-sm font-medium text-gray-700 mb-1">
							Password <span class="text-red-500">*</span>
						</label>
						<input
							type="password"
							id="password"
							bind:value={formData.password}
							class="w-full px-4 py-2 border {errors.password ? 'border-red-500' : 'border-gray-300'} rounded-md focus:ring-indigo-500 focus:border-indigo-500"
							placeholder="Min 8 characters"
							aria-invalid={errors.password ? 'true' : 'false'}
						/>
						{#if errors.password}
							<p class="mt-1 text-sm text-red-600">{errors.password}</p>
						{/if}
					</div>

					<div>
						<label for="confirm-password" class="block text-sm font-medium text-gray-700 mb-1">
							Confirm Password <span class="text-red-500">*</span>
						</label>
						<input
							type="password"
							id="confirm-password"
							bind:value={formData.confirmPassword}
							class="w-full px-4 py-2 border {errors.confirmPassword ? 'border-red-500' : 'border-gray-300'} rounded-md focus:ring-indigo-500 focus:border-indigo-500"
							placeholder="Confirm your password"
							aria-invalid={errors.confirmPassword ? 'true' : 'false'}
						/>
						{#if errors.confirmPassword}
							<p class="mt-1 text-sm text-red-600">{errors.confirmPassword}</p>
						{/if}
					</div>
				</div>
			</section>

			<!-- Select and Textarea -->
			<section class="mb-8" aria-labelledby="select-textarea-heading">
				<h2 id="select-textarea-heading" class="text-xl font-semibold text-gray-900 mb-4 pb-2 border-b border-gray-200">Select & Textarea</h2>
				<div class="space-y-6">
					<div>
						<label for="country" class="block text-sm font-medium text-gray-700 mb-1">
							Country <span class="text-red-500">*</span>
						</label>
						<select
							id="country"
							bind:value={formData.country}
							class="w-full px-4 py-2 border {errors.country ? 'border-red-500' : 'border-gray-300'} rounded-md focus:ring-indigo-500 focus:border-indigo-500"
							aria-invalid={errors.country ? 'true' : 'false'}
						>
							{#each countries as country}
								<option value={country.value}>{country.label}</option>
							{/each}
						</select>
						{#if errors.country}
							<p class="mt-1 text-sm text-red-600">{errors.country}</p>
						{/if}
					</div>

					<div>
						<label for="bio" class="block text-sm font-medium text-gray-700 mb-1">Bio</label>
						<textarea
							id="bio"
							bind:value={formData.bio}
							rows="4"
							class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500"
							placeholder="Tell us about yourself..."
						></textarea>
						<p class="mt-1 text-xs text-gray-500">Maximum 500 characters</p>
					</div>
				</div>
			</section>

			<!-- Checkboxes -->
			<section class="mb-8" aria-labelledby="checkboxes-heading">
				<h2 id="checkboxes-heading" class="text-xl font-semibold text-gray-900 mb-4 pb-2 border-b border-gray-200">Interests (Checkboxes)</h2>
				<div class="grid grid-cols-2 md:grid-cols-3 gap-4">
					{#each interests as interest}
						<label class="flex items-center gap-3 p-3 border border-gray-200 rounded-md hover:bg-gray-50 cursor-pointer">
							<input
								type="checkbox"
								value={interest.value}
								bind:group={formData.interests}
								class="w-5 h-5 text-indigo-600 rounded focus:ring-indigo-500"
							/>
							<span class="text-gray-700">{interest.label}</span>
						</label>
					{/each}
				</div>
				{#if formData.interests.length > 0}
					<p class="mt-3 text-sm text-gray-600">Selected: {formData.interests.join(', ')}</p>
				{/if}
			</section>

			<!-- Special Inputs -->
			<section class="mb-8" aria-labelledby="special-inputs-heading">
				<h2 id="special-inputs-heading" class="text-xl font-semibold text-gray-900 mb-4 pb-2 border-b border-gray-200">Special Input Types</h2>
				<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
					<div>
						<label for="birthdate" class="block text-sm font-medium text-gray-700 mb-1">Birth Date</label>
						<input
							type="date"
							id="birthdate"
							bind:value={formData.birthdate}
							class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500"
						/>
					</div>

					<div>
						<label for="website" class="block text-sm font-medium text-gray-700 mb-1">Website</label>
						<input
							type="url"
							id="website"
							bind:value={formData.website}
							class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500"
							placeholder="https://example.com"
						/>
					</div>

					<div>
						<label for="phone" class="block text-sm font-medium text-gray-700 mb-1">Phone Number</label>
						<input
							type="tel"
							id="phone"
							bind:value={formData.phone}
							class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500"
							placeholder="+1 (555) 123-4567"
						/>
					</div>

					<div>
						<label for="color" class="block text-sm font-medium text-gray-700 mb-1">Favorite Color</label>
						<div class="flex items-center gap-3">
							<input
								type="color"
								id="color"
								bind:value={formData.color}
								class="w-12 h-10 border border-gray-300 rounded-md cursor-pointer"
							/>
							<span class="text-sm text-gray-600 font-mono">{formData.color}</span>
						</div>
					</div>
				</div>
			</section>

			<!-- Range Input -->
			<section class="mb-8" aria-labelledby="range-heading">
				<h2 id="range-heading" class="text-xl font-semibold text-gray-900 mb-4 pb-2 border-b border-gray-200">Rating (Range)</h2>
				<div>
					<label for="rating" class="block text-sm font-medium text-gray-700 mb-2">
						Rate your experience: <span class="text-indigo-600 font-bold">{formData.rating}/5</span>
					</label>
					<input
						type="range"
						id="rating"
						min="1"
						max="5"
						step="1"
						bind:value={formData.rating}
						class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-indigo-600"
					/>
					<div class="flex justify-between text-xs text-gray-500 mt-1">
						<span>Poor</span>
						<span>Excellent</span>
					</div>
				</div>
			</section>

			<!-- Terms & Submit -->
			<section class="mb-8" aria-labelledby="terms-heading">
				<h2 id="terms-heading" class="text-xl font-semibold text-gray-900 mb-4 pb-2 border-b border-gray-200">Terms & Submission</h2>
				<div class="space-y-4">
					<label class="flex items-start gap-3">
						<input
							type="checkbox"
							bind:checked={formData.newsletter}
							class="w-5 h-5 mt-0.5 text-indigo-600 rounded focus:ring-indigo-500"
						/>
						<span class="text-gray-700">Subscribe to our newsletter for updates and promotions</span>
					</label>

					<label class="flex items-start gap-3">
						<input
							type="checkbox"
							bind:checked={formData.terms}
							class="w-5 h-5 mt-0.5 text-indigo-600 rounded focus:ring-indigo-500 {errors.terms ? 'border-red-500' : ''}"
							aria-invalid={errors.terms ? 'true' : 'false'}
						/>
						<span class="text-gray-700">
							I agree to the <a href="/" class="text-indigo-600 hover:underline">Terms and Conditions</a> 
							and <a href="/" class="text-indigo-600 hover:underline">Privacy Policy</a> 
							<span class="text-red-500">*</span>
						</span>
					</label>
					{#if errors.terms}
						<p class="text-sm text-red-600 ml-8">{errors.terms}</p>
					{/if}
				</div>
			</section>

			<!-- Submit Buttons -->
			<div class="flex flex-col sm:flex-row gap-4 pt-6 border-t border-gray-200">
				<button
					type="submit"
					class="flex-1 sm:flex-none px-8 py-3 bg-indigo-600 text-white font-medium rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors"
				>
					Submit Form
				</button>
				<button
					type="reset"
					on:click={() => { formData = { ...formData, username: '', email: '', password: '', confirmPassword: '', bio: '', country: '', interests: [], newsletter: false, terms: false, rating: 3, birthdate: '', website: '', phone: '', color: '#4f46e5' }; errors = {}; }}
					class="flex-1 sm:flex-none px-8 py-3 border border-gray-300 text-gray-700 font-medium rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 transition-colors"
				>
					Reset
				</button>
			</div>
		</form>
	</div>
</div>
