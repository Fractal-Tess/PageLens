# Remaining Proposed Features for PageLens

This document lists high-value features that are still missing or only partially implemented in PageLens.

## 1) JavaScript Error and Console Warning Audits

- Capture runtime exceptions and console warnings with stack traces, source URLs, and severity classification.

## 2) Render-Blocking Resource Detection

- Identify CSS/JS assets that delay first render and provide concrete optimization hints (defer/async/preload/splitting).

## 3) Unused CSS/JS and Payload Efficiency Audits

- Estimate unused bytes per asset and report optimization opportunities for bundle size and compression.

## 4) Accessibility Rule Engine Expansion

- Extend accessibility checks to include landmarks, focus order, ARIA misuse, and richer form/semantic validations.

## 5) Mobile and Device-Emulation Scoring Profiles

- Support separate desktop/mobile runs with CPU/network throttling presets and profile-specific scoring.

## 6) Historical Baselines and Regression Detection

- Persist audit history per URL/environment and detect statistically significant regressions over time.

## 7) Actionable Fix Suggestions with Confidence

- For each issue, provide impact explanation, exact element/resource references, and remediation snippets.

## 8) CI Budget Gates

- Add threshold budgets for performance/SEO/accessibility and fail CI when regressions exceed limits.

## 9) Multi-Page Site-Level Audits

- Aggregate findings across crawled pages to reveal template-level issues and repeated systemic failures.
