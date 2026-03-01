// POOR SEO PAGE - Missing metadata, bad heading structure, images without alt

import Image from "next/image";

// Intentionally not exporting metadata to test missing meta tags

export default function ServicesPage() {
  return (
    <main>
      {/* Using h3 instead of h1 - bad SEO */}
      <section className="bg-gray-700 text-white py-16">
        <div className="max-w-6xl mx-auto px-4">
          <h3 className="text-3xl font-bold mb-4">Our Services</h3>
          <p>We offer various sustainable technology services.</p>
        </div>
      </section>

      <section className="py-12 max-w-6xl mx-auto px-4">
        {/* Multiple h3 tags - no proper hierarchy */}
        <div className="grid md:grid-cols-3 gap-8">
          <div>
            <h3>Energy Audits</h3>
            <Image
              src="/images/service-audit.jpg"
              alt="" // Empty alt text - SEO issue
              width={300}
              height={200}
              className="rounded"
            />
            <p>Comprehensive energy analysis for your business.</p>
          </div>
          <div>
            <h3>Solar Installation</h3>
            <Image
              src="/images/service-solar.jpg"
              alt=""  // Empty alt - still an SEO issue for meaningful images
              width={300}
              height={200}
              className="rounded"
            />
            <p>Professional solar panel installation services.</p>
          </div>
          <div>
            <h3>Carbon Consulting</h3>
            <Image
              src="/images/service-carbon.jpg"
              alt="" // Empty alt text
              width={300}
              height={200}
              className="rounded"
            />
            <p>Expert guidance on carbon reduction strategies.</p>
          </div>
        </div>
      </section>

      {/* Another h3 where h2 should be */}
      <section className="py-12 bg-gray-100">
        <div className="max-w-4xl mx-auto px-4">
          <h3>Pricing Information</h3>
          <p>Contact us for custom quotes.</p>
        </div>
      </section>
    </main>
  );
}
