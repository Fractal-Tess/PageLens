import Image from "next/image";
import Link from "next/link";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Blog - Sustainability Insights & Green Tech News | EcoTech Solutions",
  description: "Stay informed with the latest trends in sustainable technology, carbon reduction strategies, and eco-friendly business practices from our expert team.",
  keywords: ["sustainability blog", "green tech news", "carbon reduction tips", "renewable energy insights", "eco-friendly business"],
  openGraph: {
    title: "Blog - Sustainability Insights & Green Tech News | EcoTech Solutions",
    description: "Stay informed with the latest trends in sustainable technology and eco-friendly business practices.",
    url: "https://ecotech-solutions.example.com/blog",
    type: "website",
    images: [
      {
        url: "/images/blog-og.jpg",
        width: 1200,
        height: 630,
        alt: "EcoTech Solutions Blog - Sustainability and Green Technology Insights",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "Blog - Sustainability Insights & Green Tech News | EcoTech Solutions",
    description: "Stay informed with the latest trends in sustainable technology and eco-friendly business practices.",
    images: ["/images/blog-og.jpg"],
  },
  alternates: {
    canonical: "https://ecotech-solutions.example.com/blog",
  },
};

const blogPosts = [
  {
    slug: "solar-energy-2024",
    title: "The Future of Solar Energy in 2024: Efficiency Breakthroughs",
    excerpt: "Discover how perovskite tandem cells and AI-optimized tracking systems are revolutionizing solar energy efficiency, achieving record-breaking conversion rates of over 30%.",
    image: "/images/blog-solar.jpg",
    alt: "Advanced solar panel installation with sun tracking technology on a commercial building",
    date: "2024-01-15",
    category: "Renewable Energy",
    readTime: "8 min read",
  },
  {
    slug: "carbon-tracking",
    title: "Understanding Carbon Tracking for Modern Business",
    excerpt: "Learn how comprehensive carbon tracking systems can help your business identify emission hotspots, set reduction targets, and achieve carbon neutrality goals.",
    image: "/images/blog-carbon.jpg",
    alt: "Corporate carbon footprint dashboard displaying real-time emissions data",
    date: "2024-01-10",
    category: "Carbon Management",
    readTime: "6 min read",
  },
  {
    slug: "smart-buildings",
    title: "Smart Buildings: The Intersection of IoT and Sustainability",
    excerpt: "Explore how Internet of Things technology is transforming building management, reducing energy consumption by up to 40% through intelligent automation.",
    image: "/images/blog-smart-building.jpg",
    alt: "Modern smart building with integrated IoT sensors and automated lighting systems",
    date: "2024-01-05",
    category: "Smart Technology",
    readTime: "7 min read",
  },
  {
    slug: "circular-economy",
    title: "Implementing Circular Economy Principles in Manufacturing",
    excerpt: "A comprehensive guide to redesigning manufacturing processes for circularity, reducing waste while creating new revenue streams from byproducts.",
    image: "/images/blog-circular.jpg",
    alt: "Sustainable manufacturing facility with recycling stations and material recovery systems",
    date: "2023-12-28",
    category: "Sustainability Strategy",
    readTime: "10 min read",
  },
  {
    slug: "green-hydrogen",
    title: "Green Hydrogen: The Fuel of the Future?",
    excerpt: "Examining the potential of green hydrogen in decarbonizing heavy industry and long-haul transportation, plus the infrastructure challenges ahead.",
    image: "/images/blog-hydrogen.jpg",
    alt: "Green hydrogen production facility using renewable energy electrolysis",
    date: "2023-12-20",
    category: "Clean Energy",
    readTime: "9 min read",
  },
  {
    slug: "esg-reporting",
    title: "ESG Reporting Standards: Navigating the New Requirements",
    excerpt: "Understanding the evolving landscape of Environmental, Social, and Governance reporting and how to prepare your organization for compliance.",
    image: "/images/blog-esg.jpg",
    alt: "Corporate ESG report showing environmental metrics and sustainability achievements",
    date: "2023-12-15",
    category: "Compliance",
    readTime: "5 min read",
  },
];

export default function BlogIndexPage() {
  return (
    <main>
      {/* Hero Section */}
      <section className="bg-gradient-to-br from-emerald-900 to-slate-900 text-white py-20">
        <div className="max-w-6xl mx-auto px-4">
          <h1 className="text-4xl md:text-5xl font-bold mb-6">
            Sustainability Insights & Green Tech News
          </h1>
          <p className="text-xl text-emerald-100 max-w-3xl">
            Expert perspectives on sustainable technology, carbon reduction strategies, 
            and the latest innovations driving the green revolution.
          </p>
        </div>
      </section>

      {/* Featured Post */}
      <section className="py-16">
        <div className="max-w-6xl mx-auto px-4">
          <article className="grid md:grid-cols-2 gap-8 items-center bg-white rounded-2xl overflow-hidden shadow-lg">
            <div className="relative h-64 md:h-full min-h-[300px]">
              <Image
                src={blogPosts[0].image}
                alt={blogPosts[0].alt}
                fill
                className="object-cover"
                sizes="(max-width: 768px) 100vw, 50vw"
                priority
              />
            </div>
            <div className="p-8">
              <div className="flex items-center gap-4 text-sm text-slate-500 mb-4">
                <span className="bg-emerald-100 text-emerald-700 px-3 py-1 rounded-full font-medium">
                  {blogPosts[0].category}
                </span>
                <span>{blogPosts[0].date}</span>
                <span>{blogPosts[0].readTime}</span>
              </div>
              <h2 className="text-2xl font-bold mb-4 text-slate-900">
                <Link href={`/blog/${blogPosts[0].slug}`} className="hover:text-emerald-600">
                  {blogPosts[0].title}
                </Link>
              </h2>
              <p className="text-slate-600 mb-6">{blogPosts[0].excerpt}</p>
              <Link
                href={`/blog/${blogPosts[0].slug}`}
                className="inline-flex items-center text-emerald-600 font-medium hover:underline"
              >
                Read Article
                <svg className="w-4 h-4 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 8l4 4m0 0l-4 4m4-4H3" />
                </svg>
              </Link>
            </div>
          </article>
        </div>
      </section>

      {/* Blog Grid */}
      <section className="py-16 bg-slate-50">
        <div className="max-w-6xl mx-auto px-4">
          <h2 className="text-2xl font-bold mb-8 text-slate-900">Latest Articles</h2>
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            {blogPosts.slice(1).map((post) => (
              <article
                key={post.slug}
                className="bg-white rounded-xl overflow-hidden shadow-md hover:shadow-lg transition-shadow"
              >
                <div className="relative h-48">
                  <Image
                    src={post.image}
                    alt={post.alt}
                    fill
                    className="object-cover"
                    sizes="(max-width: 768px) 100vw, (max-width: 1024px) 50vw, 33vw"
                  />
                </div>
                <div className="p-6">
                  <div className="flex items-center gap-3 text-xs text-slate-500 mb-3">
                    <span className="bg-emerald-50 text-emerald-600 px-2 py-1 rounded">
                      {post.category}
                    </span>
                    <span>{post.readTime}</span>
                  </div>
                  <h3 className="text-lg font-semibold mb-2 text-slate-900">
                    <Link href={`/blog/${post.slug}`} className="hover:text-emerald-600">
                      {post.title}
                    </Link>
                  </h3>
                  <p className="text-slate-600 text-sm line-clamp-3">{post.excerpt}</p>
                </div>
              </article>
            ))}
          </div>
        </div>
      </section>

      {/* Newsletter CTA */}
      <section className="py-16">
        <div className="max-w-4xl mx-auto px-4 text-center">
          <h2 className="text-3xl font-bold mb-4 text-slate-900">
            Stay Updated on Green Tech
          </h2>
          <p className="text-slate-600 mb-8">
            Subscribe to our newsletter for weekly insights on sustainable technology 
            and carbon reduction strategies.
          </p>
          <form className="flex flex-col sm:flex-row gap-4 max-w-md mx-auto">
            <input
              type="email"
              placeholder="Enter your email"
              className="flex-1 px-4 py-3 border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-emerald-500"
              aria-label="Email address for newsletter subscription"
            />
            <button
              type="submit"
              className="bg-emerald-600 hover:bg-emerald-700 text-white px-8 py-3 rounded-lg font-medium transition-colors"
            >
              Subscribe
            </button>
          </form>
        </div>
      </section>

      {/* Structured Data */}
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{
          __html: JSON.stringify({
            "@context": "https://schema.org",
            "@type": "Blog",
            "name": "EcoTech Solutions Blog",
            "description": "Sustainability insights and green technology news",
            "url": "https://ecotech-solutions.example.com/blog",
            "publisher": {
              "@type": "Organization",
              "name": "EcoTech Solutions",
              "logo": {
                "@type": "ImageObject",
                "url": "https://ecotech-solutions.example.com/logo.png",
              },
            },
            "blogPosts": blogPosts.map((post) => ({
              "@type": "BlogPosting",
              "headline": post.title,
              "description": post.excerpt,
              "image": `https://ecotech-solutions.example.com${post.image}`,
              "datePublished": post.date,
              "author": {
                "@type": "Organization",
                "name": "EcoTech Solutions",
              },
            })),
          }),
        }}
      />
    </main>
  );
}
