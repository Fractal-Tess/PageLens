# Proposed Missing Features for PageLens

This document lists high-value features that are currently missing or only partially implemented in PageLens.

## 1) True Core Web Vitals Scoring Model

- Implement percentile/log-normal scoring for LCP, CLS, INP, TBT, and FCP instead of fixed heuristic thresholds.

## 2) Per-Request Waterfall and Critical Path View

- Add request timeline visualization with initiator chains, blocking resources, and long-tail requests.

## 3) Robots.txt and Sitemap.xml Fetch/Validation

- Fetch and parse `robots.txt` and sitemap files, validate syntax, and surface crawl/indexing conflicts.

## 4) Full Redirect-Chain Analysis

- Record complete redirect hops (URL, status, latency), detect long chains/loops, and estimate performance cost.

## 5) Header Policy Audits Beyond Presence

- Validate quality and strictness of security headers (CSP, HSTS, Referrer-Policy, COOP/COEP/CORP), not just existence.

## 6) JavaScript Error and Console Warning Audits

- Capture runtime exceptions and console warnings with stack traces, source URLs, and severity classification.

## 7) Render-Blocking Resource Detection

- Identify CSS/JS assets that delay first render and provide concrete optimization hints (defer/async/preload/splitting).

## 8) Unused CSS/JS and Payload Efficiency Audits

- Estimate unused bytes per asset and report optimization opportunities for bundle size and compression.

## 9) Accessibility Rule Engine Expansion

- Extend accessibility checks to include landmarks, focus order, ARIA misuse, and richer form/semantic validations.

## 10) Mobile and Device-Emulation Scoring Profiles

- Support separate desktop/mobile runs with CPU/network throttling presets and profile-specific scoring.

## 11) Historical Baselines and Regression Detection

- Persist audit history per URL/environment and detect statistically significant regressions over time.

## 12) Actionable Fix Suggestions with Confidence

- For each issue, provide impact explanation, exact element/resource references, and remediation snippets.

## 13) CI Budget Gates

- Add threshold budgets for performance/SEO/accessibility and fail CI when regressions exceed limits.

## 14) Multi-Page Site-Level Audits

- Aggregate findings across crawled pages to reveal template-level issues and repeated systemic failures.
