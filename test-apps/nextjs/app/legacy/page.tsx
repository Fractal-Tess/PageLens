// TERRIBLE SEO PAGE - Minimal/no SEO practices
// Missing: title, description, charset handled by layout but still

export default function LegacyPage() {
  return (
    <div>
      {/* No semantic HTML - using div instead of main, section, etc */}
      
      {/* Multiple H1 tags - major SEO issue */}
      <h1>Legacy Systems Documentation</h1>
      <h1>Old Information Archive</h1>

      <div>
        <h1>Third H1 Tag</h1>
        <p>This page contains old documentation that is no longer maintained.</p>
      </div>

      {/* Skipped heading levels - h1 to h4 */}
      <h4>Available Documents</h4>
      <ul>
        <li>System Manual v1.0</li>
        <li>Deprecated API Reference</li>
        <li>Legacy Configuration Guide</li>
      </ul>

      {/* Images without src or alt - broken and inaccessible */}
      <img />
      <img src="old-image.jpg" />
      
      {/* More skipped heading levels */}
      <h6>Contact Information</h6>
      <p>For questions about legacy systems, email legacy@example.com</p>

      <h2>Recent Updates Section</h2>
      {/* Another case of skipped levels - h2 to h5 */}
      <h5>2023 Changes</h5>
      <p>Various updates were made.</p>

      <div>
        <h1>Another H1 Tag Here</h1>
        <p>Yes, we have four H1 tags on this page.</p>
      </div>
    </div>
  );
}
