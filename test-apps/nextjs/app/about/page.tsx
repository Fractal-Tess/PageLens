import Image from "next/image";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "About Us - Our Mission & Team | EcoTech Solutions",
  description: "Learn about EcoTech Solutions' mission to transform businesses through sustainable technology. Meet our team of environmental experts and innovators.",
  keywords: ["about ecotech", "sustainability mission", "green tech team", "environmental experts", "company history"],
  openGraph: {
    title: "About Us - Our Mission & Team | EcoTech Solutions",
    description: "Learn about EcoTech Solutions' mission to transform businesses through sustainable technology.",
    url: "https://ecotech-solutions.example.com/about",
    type: "website",
    images: [
      {
        url: "/images/about-team.jpg",
        width: 1200,
        height: 630,
        alt: "EcoTech Solutions leadership team in modern sustainable office",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "About Us - Our Mission & Team | EcoTech Solutions",
    description: "Learn about EcoTech Solutions' mission to transform businesses through sustainable technology.",
    images: ["/images/about-team.jpg"],
  },
  alternates: {
    canonical: "https://ecotech-solutions.example.com/about",
  },
};

export default function AboutPage() {
  return (
    <main>
      {/* Hero Section */}
      <section className="bg-gradient-to-r from-emerald-800 to-teal-900 text-white py-20">
        <div className="max-w-6xl mx-auto px-4">
          <h1 className="text-4xl md:text-5xl font-bold mb-6">
            Our Mission: Technology for a Greener Tomorrow
          </h1>
          <p className="text-xl text-emerald-100 max-w-3xl">
            Founded in 2015, EcoTech Solutions has been at the forefront of sustainable 
            technology innovation, helping businesses worldwide reduce their environmental 
            impact while improving their bottom line.
          </p>
        </div>
      </section>

      {/* Our Story */}
      <section className="py-16">
        <div className="max-w-6xl mx-auto px-4">
          <div className="grid md:grid-cols-2 gap-12 items-center">
            <div>
              <h2 className="text-3xl font-bold mb-6 text-slate-900">Our Story</h2>
              <p className="text-slate-600 mb-4">
                EcoTech Solutions began with a simple but powerful idea: that technology 
                and environmental sustainability should go hand in hand. Our founders, 
                Dr. Sarah Chen and Marcus Rodriguez, met at a climate technology conference 
                in 2014 and discovered their shared vision.
              </p>
              <p className="text-slate-600 mb-4">
                What started as a small consultancy in San Francisco has grown into a 
                global company with offices in 15 countries. We have helped over 500 
                businesses implement sustainable technology solutions, reducing their 
                combined carbon footprint by over 2 million tons.
              </p>
              <p className="text-slate-600">
                Today, our team of 200+ experts continues to push the boundaries of 
                what is possible in sustainable technology, from AI-powered energy 
                optimization to breakthrough renewable energy systems.
              </p>
            </div>
            <div className="relative h-96 rounded-xl overflow-hidden shadow-xl">
              <Image
                src="/images/about-office.jpg"
                alt="EcoTech Solutions modern sustainable office interior with living walls and natural lighting"
                fill
                className="object-cover"
                sizes="(max-width: 768px) 100vw, 50vw"
              />
            </div>
          </div>
        </div>
      </section>

      {/* Core Values */}
      <section className="py-16 bg-slate-50">
        <div className="max-w-6xl mx-auto px-4">
          <h2 className="text-3xl font-bold text-center mb-12 text-slate-900">
            Our Core Values
          </h2>
          <div className="grid md:grid-cols-4 gap-6">
            <div className="bg-white p-6 rounded-xl shadow-sm text-center">
              <div className="w-16 h-16 bg-emerald-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg className="w-8 h-8 text-emerald-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064" />
                </svg>
              </div>
              <h3 className="text-lg font-semibold mb-2 text-slate-900">Sustainability First</h3>
              <p className="text-slate-600 text-sm">
                Every solution we design prioritizes environmental impact.
              </p>
            </div>
            <div className="bg-white p-6 rounded-xl shadow-sm text-center">
              <div className="w-16 h-16 bg-emerald-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg className="w-8 h-8 text-emerald-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                </svg>
              </div>
              <h3 className="text-lg font-semibold mb-2 text-slate-900">Innovation</h3>
              <p className="text-slate-600 text-sm">
                We constantly push boundaries to find better solutions.
              </p>
            </div>
            <div className="bg-white p-6 rounded-xl shadow-sm text-center">
              <div className="w-16 h-16 bg-emerald-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg className="w-8 h-8 text-emerald-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                </svg>
              </div>
              <h3 className="text-lg font-semibold mb-2 text-slate-900">Partnership</h3>
              <p className="text-slate-600 text-sm">
                We work alongside our clients as true partners.
              </p>
            </div>
            <div className="bg-white p-6 rounded-xl shadow-sm text-center">
              <div className="w-16 h-16 bg-emerald-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg className="w-8 h-8 text-emerald-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <h3 className="text-lg font-semibold mb-2 text-slate-900">Integrity</h3>
              <p className="text-slate-600 text-sm">
                We are transparent and honest in everything we do.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Leadership Team */}
      <section className="py-16">
        <div className="max-w-6xl mx-auto px-4">
          <h2 className="text-3xl font-bold text-center mb-12 text-slate-900">
            Meet Our Leadership Team
          </h2>
          <div className="grid md:grid-cols-3 gap-8">
            <div className="text-center">
              <div className="relative w-48 h-48 mx-auto mb-4 rounded-full overflow-hidden">
                <Image
                  src="/images/team-sarah.jpg"
                  alt="Dr. Sarah Chen, CEO and Co-founder of EcoTech Solutions"
                  fill
                  className="object-cover"
                  sizes="192px"
                />
              </div>
              <h3 className="text-xl font-semibold text-slate-900">Dr. Sarah Chen</h3>
              <p className="text-emerald-600 font-medium mb-2">CEO & Co-Founder</p>
              <p className="text-slate-600 text-sm">
                Former MIT researcher with 15 years in renewable energy systems.
              </p>
            </div>
            <div className="text-center">
              <div className="relative w-48 h-48 mx-auto mb-4 rounded-full overflow-hidden">
                <Image
                  src="/images/team-marcus.jpg"
                  alt="Marcus Rodriguez, CTO and Co-founder of EcoTech Solutions"
                  fill
                  className="object-cover"
                  sizes="192px"
                />
              </div>
              <h3 className="text-xl font-semibold text-slate-900">Marcus Rodriguez</h3>
              <p className="text-emerald-600 font-medium mb-2">CTO & Co-Founder</p>
              <p className="text-slate-600 text-sm">
                Tech innovator with experience at Tesla and SolarCity.
              </p>
            </div>
            <div className="text-center">
              <div className="relative w-48 h-48 mx-auto mb-4 rounded-full overflow-hidden">
                <Image
                  src="/images/team-elena.jpg"
                  alt="Elena Volkov, Chief Sustainability Officer at EcoTech Solutions"
                  fill
                  className="object-cover"
                  sizes="192px"
                />
              </div>
              <h3 className="text-xl font-semibold text-slate-900">Elena Volkov</h3>
              <p className="text-emerald-600 font-medium mb-2">Chief Sustainability Officer</p>
              <p className="text-slate-600 text-sm">
                Environmental scientist and former UN climate advisor.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Structured Data */}
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{
          __html: JSON.stringify({
            "@context": "https://schema.org",
            "@type": "AboutPage",
            "mainEntity": {
              "@type": "Organization",
              "name": "EcoTech Solutions",
              "founders": [
                {
                  "@type": "Person",
                  "name": "Dr. Sarah Chen",
                  "jobTitle": "CEO & Co-Founder",
                },
                {
                  "@type": "Person",
                  "name": "Marcus Rodriguez",
                  "jobTitle": "CTO & Co-Founder",
                },
              ],
              "foundingDate": "2015",
              "description": "Leading provider of sustainable technology solutions for businesses worldwide.",
            },
          }),
        }}
      />
    </main>
  );
}
