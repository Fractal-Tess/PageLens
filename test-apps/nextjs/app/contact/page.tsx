import type { Metadata } from "next";

// Medium SEO - some issues like no OG image specified
export const metadata: Metadata = {
  title: "Contact EcoTech Solutions",
  description: "Get in touch with our team for sustainable technology consultations.",
  // Missing keywords
  openGraph: {
    title: "Contact EcoTech Solutions",
    description: "Get in touch with our team for sustainable technology consultations.",
    // Missing image
  },
  // Missing twitter card
  alternates: {
    canonical: "https://ecotech-solutions.example.com/contact",
  },
};

export default function ContactPage() {
  return (
    <main>
      <section className="bg-slate-800 text-white py-16">
        <div className="max-w-4xl mx-auto px-4">
          <h1 className="text-3xl font-bold mb-4">Contact Us</h1>
          <p className="text-slate-300">
            Ready to start your sustainability journey? We are here to help.
          </p>
        </div>
      </section>

      <section className="py-12 max-w-4xl mx-auto px-4">
        <div className="grid md:grid-cols-2 gap-12">
          <div>
            <h2 className="text-xl font-semibold mb-6">Send us a Message</h2>
            <form className="space-y-4">
              <div>
                <label htmlFor="name" className="block text-sm font-medium text-slate-700 mb-1">
                  Name
                </label>
                <input
                  type="text"
                  id="name"
                  name="name"
                  className="w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                />
              </div>
              <div>
                <label htmlFor="email" className="block text-sm font-medium text-slate-700 mb-1">
                  Email
                </label>
                <input
                  type="email"
                  id="email"
                  name="email"
                  className="w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                />
              </div>
              <div>
                <label htmlFor="message" className="block text-sm font-medium text-slate-700 mb-1">
                  Message
                </label>
                <textarea
                  id="message"
                  name="message"
                  rows={4}
                  className="w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                />
              </div>
              <button
                type="submit"
                className="w-full bg-emerald-600 hover:bg-emerald-700 text-white py-3 rounded-lg font-medium transition-colors"
              >
                Send Message
              </button>
            </form>
          </div>

          <div>
            <h2 className="text-xl font-semibold mb-6">Contact Information</h2>
            <div className="space-y-4">
              <div>
                <h3 className="font-medium text-slate-900">Address</h3>
                <p className="text-slate-600">
                  123 Green Innovation Drive<br />
                  San Francisco, CA 94105
                </p>
              </div>
              <div>
                <h3 className="font-medium text-slate-900">Phone</h3>
                <p className="text-slate-600">+1 (555) ECO-TECH</p>
              </div>
              <div>
                <h3 className="font-medium text-slate-900">Email</h3>
                <p className="text-slate-600">info@ecotech-solutions.example.com</p>
              </div>
              <div>
                <h3 className="font-medium text-slate-900">Hours</h3>
                <p className="text-slate-600">
                  Monday - Friday: 9:00 AM - 6:00 PM PST
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>
    </main>
  );
}
