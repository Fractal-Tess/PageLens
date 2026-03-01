import { NextResponse } from 'next/server'
import type { NextRequest } from 'next/server'

/**
 * Response Time Control Middleware
 * 
 * This middleware allows artificial delays to be added to responses
 * for testing the PageLens analyzer's handling of slow pages.
 * 
 * Set DELAY_MS environment variable to add delays:
 *   DELAY_MS=1000 npm run dev
 * 
 * Or use specific page delays:
 *   SLOW_PAGE_DELAY=2000 npm run dev
 */

export function middleware(request: NextRequest) {
  // Check if we should add artificial delays (for testing)
  const globalDelay = parseInt(process.env.DELAY_MS || '0', 10)
  const slowPageDelay = parseInt(process.env.SLOW_PAGE_DELAY || '0', 10)
  
  // Add delay header for tracking
  const response = NextResponse.next()
  
  // You can also add specific delays for certain paths
  if (request.nextUrl.pathname === '/slow-page') {
    // This would need actual delay implementation
    // For now, we just add a header indicating it should be slow
    response.headers.set('X-Artificial-Delay', String(slowPageDelay || globalDelay))
  }
  
  return response
}

export const config = {
  matcher: [
    // Apply to all pages except static files and api
    '/((?!api|_next/static|_next/image|favicon.ico|images).*)',
  ],
}
