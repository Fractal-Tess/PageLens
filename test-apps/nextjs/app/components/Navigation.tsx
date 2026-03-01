"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const navLinks = [
  { href: "/", label: "Home" },
  { href: "/about", label: "About" },
  { href: "/services", label: "Services" },
  { href: "/blog", label: "Blog" },
  { href: "/contact", label: "Contact" },
  { href: "/slow-page", label: "Slow" },
  { href: "/legacy", label: "Legacy" },
  { href: "/draft", label: "Draft" },
];

export default function Navigation() {
  const pathname = usePathname();

  return (
    <nav className="bg-slate-900 text-white">
      <div className="max-w-6xl mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          <Link href="/" className="font-bold text-xl text-emerald-400">
            EcoTech Solutions
          </Link>
          <ul className="flex space-x-6">
            {navLinks.map((link) => (
              <li key={link.href}>
                <Link
                  href={link.href}
                  className={`hover:text-emerald-400 transition-colors ${
                    pathname === link.href ? "text-emerald-400 font-medium" : ""
                  }`}
                >
                  {link.label}
                </Link>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </nav>
  );
}
