import Image from "next/image";
import Link from "next/link";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "EcoTech Solutions - Leading Sustainable Technology Provider",
  description: "Transform your business with our cutting-edge sustainable technology solutions. Reduce costs, minimize environmental impact, and build a greener future.",
  keywords: ["sustainability", "green technology", "carbon footprint reduction", "renewable energy systems"],
  openGraph: {
    title: "EcoTech Solutions - Leading Sustainable Technology Provider",
    description: "Transform your business with our cutting-edge sustainable technology solutions.",
    url: "https://ecotech-solutions.example.com",
    type: "website",
    images: [
      {
        url: "/images/home-hero.jpg",
        width: 1200,
        height: 630,
        alt: "EcoTech Solutions Headquarters with Solar Panels",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "EcoTech Solutions - Leading Sustainable Technology Provider",
    description: "Transform your business with our cutting-edge sustainable technology solutions.",
    images: ["/images/home-hero.jpg"],
  },
  alternates: {
    canonical: "https://ecotech-solutions.example.com",
  },
};

export default function Home() {
  return (
    <main>
      {/* Hero Section */}
      <section className="bg-gradient-to-br from-emerald-900 via-slate-900 to-slate-900 text-white py-24">
        <div className="max-w-6xl mx-auto px-4">
          <div className="grid md:grid-cols-2 gap-12 items-center">
            <div>
              <h1 className="text-5xl font-bold mb-6 leading-tight">
                Building a Sustainable Future with Innovative Technology
              </h1>
              <p className="text-xl text-slate-300 mb-8">
                EcoTech Solutions empowers businesses to achieve carbon neutrality 
                through cutting-edge sustainable technology and expert consultation.
              </p>
              <div className="flex gap-4">
                <Link
                  href="/services"
                  className="bg-emerald-500 hover:bg-emerald-600 text-white px-8 py-3 rounded-lg font-medium transition-colors"
                >
                  Explore Services
                </Link>
                <Link
                  href="/contact"
                  className="border border-white hover:bg-white hover:text-slate-900 text-white px-8 py-3 rounded-lg font-medium transition-colors"
                >
                  Get in Touch
                </Link>
              </div>
            </div>
            <div className="relative h-96 rounded-xl overflow-hidden shadow-2xl">
              <Image
                src="/images/home-hero.jpg"
                alt="Modern sustainable technology facility with solar panels and green architecture"
                fill
                className="object-cover"
                priority
                sizes="(max-width: 768px) 100vw, 50vw"
              />
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20 bg-slate-50">
        <div className="max-w-6xl mx-auto px-4">
          <h2 className="text-3xl font-bold text-center mb-4 text-slate-900">
            Why Choose EcoTech Solutions?
          </h2>
          <p className="text-center text-slate-600 mb-12 max-w-2xl mx-auto">
            We combine expertise in sustainable technology with a passion for environmental stewardship.
          </p>
          <div className="grid md:grid-cols-3 gap-8">
            <div className="bg-white p-8 rounded-xl shadow-md">
              <div className="w-14 h-14 bg-emerald-100 rounded-lg flex items-center justify-center mb-4">
                <svg className="w-8 h-8 text-emerald-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-3 text-slate-900">Energy Efficiency</h3>
              <p className="text-slate-600">
                Reduce your energy consumption by up to 60% with our smart building 
                solutions and IoT-powered monitoring systems.
              </p>
            </div>
            <div className="bg-white p-8 rounded-xl shadow-md">
              <div className="w-14 h-14 bg-emerald-100 rounded-lg flex items-center justify-center mb-4">
                <svg className="w-8 h-8 text-emerald-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-3 text-slate-900">Carbon Neutral</h3>
              <p className="text-slate-600">
                Achieve true carbon neutrality with our comprehensive offset programs 
                and renewable energy integration strategies.
              </p>
            </div>
            <div className="bg-white p-8 rounded-xl shadow-md">
              <div className="w-14 h-14 bg-emerald-100 rounded-lg flex items-center justify-center mb-4">
                <svg className="w-8 h-8 text-emerald-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-3 text-slate-900">Certified Expertise</h3>
              <p className="text-slate-600">
                Our team holds certifications from leading sustainability organizations 
                and has implemented solutions for Fortune 500 companies.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="py-20 bg-emerald-900 text-white">
        <div className="max-w-6xl mx-auto px-4">
          <div className="grid md:grid-cols-4 gap-8 text-center">
            <div>
              <div className="text-4xl font-bold text-emerald-400 mb-2">500+</div>
              <div className="text-emerald-100">Projects Completed</div>
            </div>
            <div>
              <div className="text-4xl font-bold text-emerald-400 mb-2">2M</div>
              <div className="text-emerald-100">Tons CO₂ Offset</div>
            </div>
            <div>
              <div className="text-4xl font-bold text-emerald-400 mb-2">98%</div>
              <div className="text-emerald-100">Client Satisfaction</div>
            </div>
            <div>
              <div className="text-4xl font-bold text-emerald-400 mb-2">15</div>
              <div className="text-emerald-100">Countries Served</div>
            </div>
          </div>
        </div>
      </section>

      {/* Latest Blog Posts */}
      <section className="py-20">
        <div className="max-w-6xl mx-auto px-4">
          <h2 className="text-3xl font-bold text-center mb-12 text-slate-900">
            Latest Insights
          </h2>
          <div className="grid md:grid-cols-2 gap-8">
            <article className="border border-slate-200 rounded-xl overflow-hidden hover:shadow-lg transition-shadow">
              <div className="relative h-48 bg-slate-200">
                <Image
                  src="/images/blog-solar.jpg"
                  alt="Solar panels installed on a modern commercial building rooftop"
                  fill
                  className="object-cover"
                  sizes="(max-width: 768px) 100vw, 50vw"
                />
              </div>
              <div className="p-6">
                <h3 className="text-xl font-semibold mb-2 text-slate-900">
                  <Link href="/blog/solar-energy-2024" className="hover:text-emerald-600">
                    The Future of Solar Energy in 2024
                  </Link>
                </h3>
                <p className="text-slate-600 mb-4">
                  Discover how advancements in photovoltaic technology are making 
                  solar energy more accessible and efficient than ever before.
                </p>
                <Link 
                  href="/blog/solar-energy-2024" 
                  className="text-emerald-600 font-medium hover:underline"
                >
                  Read More →
                </Link>
              </div>
            </article>
            <article className="border border-slate-200 rounded-xl overflow-hidden hover:shadow-lg transition-shadow">
              <div className="relative h-48 bg-slate-200">
                <Image
                  src="/images/blog-carbon.jpg"
                  alt="Digital dashboard showing carbon footprint analytics and metrics"
                  fill
                  className="object-cover"
                  sizes="(max-width: 768px) 100vw, 50vw"
                />
              </div>
              <div className="p-6">
                <h3 className="text-xl font-semibold mb-2 text-slate-900">
                  <Link href="/blog/carbon-tracking" className="hover:text-emerald-600">
                    Understanding Carbon Tracking for Business
                  </Link>
                </h3>
                <p className="text-slate-600 mb-4">
                  Learn how effective carbon tracking can help your business reduce 
                  emissions and meet sustainability goals.
                </p>
                <Link 
                  href="/blog/carbon-tracking" 
                  className="text-emerald-600 font-medium hover:underline"
                >
                  Read More →
                </Link>
              </div>
            </article>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 bg-slate-100">
        <div className="max-w-4xl mx-auto px-4 text-center">
          <h2 className="text-3xl font-bold mb-4 text-slate-900">
            Ready to Make a Difference?
          </h2>
          <p className="text-lg text-slate-600 mb-8">
            Join hundreds of companies that have already transformed their environmental 
            impact with EcoTech Solutions.
          </p>
          <Link
            href="/contact"
            className="bg-emerald-600 hover:bg-emerald-700 text-white px-10 py-4 rounded-lg font-medium text-lg transition-colors inline-block"
          >
            Schedule a Free Consultation
          </Link>
        </div>
      </section>

      {/* Structured Data for Organization */}
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{
          __html: JSON.stringify({
            "@context": "https://schema.org",
            "@type": "Organization",
            "name": "EcoTech Solutions",
            "url": "https://ecotech-solutions.example.com",
            "logo": "https://ecotech-solutions.example.com/logo.png",
            "description": "Leading provider of sustainable technology solutions",
            "address": {
              "@type": "PostalAddress",
              "streetAddress": "123 Green Innovation Drive",
              "addressLocality": "San Francisco",
              "addressRegion": "CA",
              "postalCode": "94105",
              "addressCountry": "US",
            },
            "contactPoint": {
              "@type": "ContactPoint",
              "telephone": "+1-555-ECO-TECH",
              "contactType": "customer service",
            },
            "sameAs": [
              "https://twitter.com/EcoTechSolutions",
              "https://linkedin.com/company/ecotech-solutions",
            ],
          }),
        }}
      />
    </main>
  );
}
