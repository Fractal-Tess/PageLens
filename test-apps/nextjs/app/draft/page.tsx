import type { Metadata } from "next";
import Image from "next/image";

// INCOMPLETE SEO PAGE - Issues with title length, description length

export const metadata: Metadata = {
  // Title too short (less than 10 chars) - SEO warning
  title: "Draft",
  // Description too short (less than 50 chars) - SEO info
  description: "Draft page content.",
  // Missing keywords, OG tags, Twitter cards
  robots: "noindex", // Draft pages shouldn't be indexed
};

export default function DraftPage() {
  return (
    <main>
      <section className="py-16 max-w-4xl mx-auto px-4">
        {/* Empty h1 - SEO error */}
        <h1></h1>
        
        <p>This is a draft page with incomplete content.</p>

        <h2>Work in Progress</h2>
        <p>
          We are currently working on this section. Please check back later for 
          more comprehensive information about our upcoming initiatives.
        </p>

        {/* Image with alt but will be overridden in actual HTML testing */}
        <div className="my-8">
          <Image
            src="/images/draft-placeholder.jpg"
            alt="Draft content placeholder showing work in progress"
            width={600}
            height={400}
            className="rounded"
          />
        </div>

        <h3>Planned Content</h3>
        <ul>
          <li>Feature overview</li>
          <li>Implementation timeline</li>
          <li>FAQ section</li>
        </ul>

        {/* Another image with empty alt */}
        <div className="my-8">
          <Image
            src="/images/timeline.jpg"
            alt=""
            width={600}
            height={300}
            className="rounded"
          />
        </div>
      </section>
    </main>
  );
}
