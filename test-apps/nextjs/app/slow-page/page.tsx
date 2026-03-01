import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Slow Loading Page - Response Time Test | EcoTech Solutions",
  description: "This page intentionally loads slowly for testing response time handling.",
};

// This function adds artificial delay during data fetching
async function getDelayedData() {
  // Get delay from environment or default to 2 seconds
  const delayMs = parseInt(process.env.SLOW_PAGE_DELAY || '2000', 10);
  
  // Artificial delay
  await new Promise(resolve => setTimeout(resolve, delayMs));
  
  return {
    loadedAt: new Date().toISOString(),
    delayMs,
    message: `This page was delayed by ${delayMs}ms for testing`,
  };
}

export default async function SlowPage() {
  const data = await getDelayedData();
  
  return (
    <main className="py-16 max-w-4xl mx-auto px-4">
      <h1 className="text-3xl font-bold mb-6">Slow Loading Page</h1>
      
      <div className="bg-amber-50 border border-amber-200 rounded-lg p-6 mb-8">
        <h2 className="text-xl font-semibold text-amber-800 mb-2">
          Response Time Test Page
        </h2>
        <p className="text-amber-700">
          This page is intentionally slowed down for testing the PageLens 
          analyzer&apos;s handling of slow-loading pages.
        </p>
      </div>
      
      <div className="space-y-4">
        <div className="bg-slate-50 p-4 rounded-lg">
          <h3 className="font-semibold text-slate-700">Delay Information</h3>
          <ul className="mt-2 space-y-2 text-slate-600">
            <li><strong>Configured Delay:</strong> {data.delayMs}ms</li>
            <li><strong>Loaded At:</strong> {data.loadedAt}</li>
            <li><strong>Message:</strong> {data.message}</li>
          </ul>
        </div>
        
        <div className="bg-blue-50 p-4 rounded-lg">
          <h3 className="font-semibold text-blue-700">How to Change Delay</h3>
          <p className="mt-2 text-blue-600 font-mono text-sm">
            SLOW_PAGE_DELAY=5000 npm run dev
          </p>
          <p className="mt-2 text-blue-600">
            Or set in .env.local file
          </p>
        </div>
      </div>
    </main>
  );
}
