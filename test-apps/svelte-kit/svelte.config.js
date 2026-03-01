import adapter from '@sveltejs/adapter-node'

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    adapter: adapter(),
    prerender: {
      handleMissingId: 'ignore',
      handleHttpError: ({ path, message }) => {
        // Ignore 404s for these paths during build
        if (path.includes('/blog/') || path.includes('/products/')) {
          return
        }
        throw new Error(message)
      }
    }
  }
}

export default config
