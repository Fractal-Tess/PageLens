# PageLens — Implementation Spec

## Overview

PageLens is a desktop + CLI web auditing tool that analyzes web pages for contrast, loading time, SEO, ARIA/accessibility, and more. It uses a bundled Chromium instance (via `chromiumoxide`) to fetch and render pages, then runs a suite of checks and presents results in a visual dashboard (desktop) or structured output (CLI).

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│                  Cargo Workspace                 │
│                                                  │
│  crates/                                         │
│  ├── pagelens-core/    Core analysis engine (lib)│
│  ├── pagelens-cli/     CLI interface (bin)       │
│  │                                               │
│  src-tauri/            Tauri desktop app (bin)    │
│  src/                  Svelte frontend            │
└──────────────────────────────────────────────────┘
```

### Crate Responsibilities

#### `pagelens-core` (library)
The engine. No UI, no framework dependencies. Everything analysis-related lives here.

- **Browser management** — Download, locate, and launch a Chromium instance. Manage browser lifecycle (launch, connect, shutdown). Provide a clean async API over `chromiumoxide`.
- **Page fetching** — Navigate to URLs, wait for load, capture the rendered DOM.
- **Analysis modules** — Each check category is a module:
  - `contrast` — WCAG color contrast ratio checks on text elements.
  - `seo` — Meta tags, headings hierarchy, open graph, structured data.
  - `accessibility` — ARIA attributes, landmark roles, alt text, form labels.
  - `performance` — Page load timing, resource sizes, request counts.
- **Result types** — Shared structs for check results, severity levels, and page reports. Serializable with serde for both CLI output and IPC to the frontend.

#### `pagelens-cli` (binary)
A full-featured CLI that wraps `pagelens-core`. Same analysis capabilities as desktop, different presentation.

- **Commands** — `pagelens <url>` to audit a single page. Subcommands for specific check categories, batch URLs, configuration.
- **Output** — Human-readable terminal output by default. `--json` flag for machine-readable output. Colored, scored summaries.
- **Dependencies** — `pagelens-core`, `clap` for argument parsing, `tokio` for async runtime.

#### `src-tauri` / `pagelens-desktop` (binary)
The Tauri shell. Thin layer that bridges `pagelens-core` to the Svelte frontend via type-safe IPC.

- **IPC** — Tauri commands that invoke `pagelens-core` functions. TypeScript bindings generated via `specta`/`tauri-specta`.
- **State** — Managed Tauri state for browser instance, analysis results, user preferences.
- **Frontend** — Svelte + shadcn-svelte dashboard. Visual presentation of audit results with per-check detail views.

#### Future crates (not yet created)
- **`pagelens-rules`** — Pluggable rule/check definition system. Users or plugins could define custom checks with severity, selectors, and expected values.
- **`pagelens-report`** — Shared reporting and output formatting. Generate JSON, HTML, or PDF reports from analysis results.

---

## Code Patterns

All crates follow the same conventions established in `src-tauri`:

### Error handling
Each crate has an `error.rs`:
```rust
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Other(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}
```
Downstream crates (cli, desktop) add `From` conversions for upstream errors.

### Prelude
Each crate has a `prelude.rs`:
```rust
pub use crate::error::Error;
pub type Result<T> = core::result::Result<T, Error>;
pub struct W<T>(pub T);
```
`W<T>` is the newtype wrapper for implementing external traits on external types.

### IPC (desktop only)
Type-safe RPC via `specta` + `tauri-specta`. All command return types derive `Serialize`, `Type`. TypeScript bindings auto-generated to `src/lib/ipc.ts` on dev builds.

---

## Tech Stack

| Layer      | Technology                                    |
|------------|-----------------------------------------------|
| Browser    | Chromium (auto-downloaded) via `chromiumoxide` |
| Async      | `tokio`                                       |
| CLI        | `clap` (derive)                               |
| Desktop    | Tauri v2                                      |
| Frontend   | Svelte 5, Tailwind CSS v4, shadcn-svelte      |
| IPC        | `specta` + `tauri-specta`                     |
| Build      | Cargo workspace, Vite, Bun                    |
| Dev env    | Nix flake, direnv, lefthook                   |

---

## Chromium Management

`pagelens-core` handles Chromium lifecycle:

1. **Discovery** — Check for an existing Chromium/Chrome binary on the system.
2. **Download** — If not found, download a known-good Chromium revision to a local cache directory (`~/.cache/pagelens/chromium/` or platform equivalent).
3. **Launch** — Spawn headless Chromium with appropriate flags. Expose a managed `Browser` handle.
4. **Pooling** — Support concurrent page analysis by managing browser tabs/contexts.
5. **Cleanup** — Graceful shutdown on drop or signal.

---

## Analysis Pipeline

```
URL
 │
 ▼
┌─────────────┐
│ Browser.get  │  Navigate, wait for load
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Snapshot    │  Capture DOM, computed styles, accessibility tree, performance timing
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────────┐
│         Analysis Modules            │
│  ┌───────────┐ ┌─────┐ ┌────────┐  │
│  │ Contrast  │ │ SEO │ │  A11y  │  │
│  └───────────┘ └─────┘ └────────┘  │
│  ┌─────────────┐                   │
│  │ Performance │                   │
│  └─────────────┘                   │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────┐
│  PageReport     │  Aggregated results, scores, issue list
└─────────────────┘
               │
       ┌───────┴───────┐
       ▼               ▼
   CLI output     IPC → Svelte dashboard
```

---

## Implementation Phases

### Phase 1 — Foundation
- [x] Cargo workspace structure
- [x] Crate scaffolding with error/prelude patterns
- [x] Tauri renamed to pagelens-desktop
- [ ] `pagelens-core`: Chromium download & launch
- [ ] `pagelens-core`: Basic page navigation and DOM capture
- [ ] `pagelens-cli`: Wire up basic `pagelens <url>` command

### Phase 2 — Core Checks
- [ ] `contrast` module — Extract text elements, compute WCAG contrast ratios
- [ ] `seo` module — Check meta tags, headings, OG tags, canonical URLs
- [ ] `accessibility` module — ARIA roles, alt text, form labels, landmark regions
- [ ] `performance` module — Navigation timing, resource count/size, LCP/FCP
- [ ] `PageReport` result type — Aggregate all checks into a scored report

### Phase 3 — CLI Polish
- [ ] Colored terminal output with severity indicators
- [ ] `--json` output flag
- [ ] Per-category subcommands (`pagelens contrast <url>`, etc.)
- [ ] Batch URL support (file input or multiple args)
- [ ] Exit codes based on severity thresholds

### Phase 4 — Desktop Dashboard
- [ ] IPC commands exposing `pagelens-core` analysis via specta
- [ ] Dashboard landing page — URL input, recent audits
- [ ] Results view — Overall score, category breakdown
- [ ] Detail view — Per-check issue list with element highlighting
- [ ] Settings — Severity thresholds, check toggles

### Phase 5 — Future
- [ ] `pagelens-rules` crate — Custom check definitions
- [ ] `pagelens-report` crate — HTML/PDF report generation
- [ ] Batch/crawl mode — Follow internal links and audit entire sites
- [ ] CI integration — `pagelens --ci` with configurable pass/fail thresholds
- [ ] Browser tab pooling for concurrent multi-page analysis
