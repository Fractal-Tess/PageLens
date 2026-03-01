import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async () => {
	// Simulate a slow server response (3 seconds)
	await new Promise(resolve => setTimeout(resolve, 3000));
	
	return {
		loadedAt: new Date().toISOString(),
		delayMs: 3000
	};
};
