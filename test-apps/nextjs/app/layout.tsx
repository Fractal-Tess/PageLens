import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import Navigation from "./components/Navigation";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "EcoTech Solutions - Sustainable Technology for a Better Future",
  description: "EcoTech Solutions provides innovative sustainable technology solutions for businesses looking to reduce their environmental footprint while increasing efficiency.",
  keywords: ["sustainable technology", "green tech", "eco-friendly solutions", "carbon neutral", "renewable energy"],
  authors: [{ name: "EcoTech Solutions" }],
  creator: "EcoTech Solutions",
  publisher: "EcoTech Solutions",
  robots: "index, follow",
  openGraph: {
    type: "website",
    locale: "en_US",
    url: "https://ecotech-solutions.example.com",
    siteName: "EcoTech Solutions",
    title: "EcoTech Solutions - Sustainable Technology for a Better Future",
    description: "Innovative sustainable technology solutions for businesses looking to reduce their environmental footprint.",
    images: [
      {
        url: "/images/og-default.jpg",
        width: 1200,
        height: 630,
        alt: "EcoTech Solutions - Sustainable Technology",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "EcoTech Solutions - Sustainable Technology for a Better Future",
    description: "Innovative sustainable technology solutions for businesses looking to reduce their environmental footprint.",
    images: ["/images/og-default.jpg"],
    creator: "@EcoTechSolutions",
  },
  alternates: {
    canonical: "https://ecotech-solutions.example.com",
  },
  verification: {
    google: "google-verification-code-placeholder",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <Navigation />
        {children}
        <footer className="bg-slate-900 text-white py-8 mt-16">
          <div className="max-w-6xl mx-auto px-4 text-center">
            <p>&copy; 2024 EcoTech Solutions. All rights reserved.</p>
            <p className="text-slate-400 text-sm mt-2">
              Leading the way in sustainable technology innovation.
            </p>
          </div>
        </footer>
      </body>
    </html>
  );
}
