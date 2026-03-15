# PageLens Knowledge Base

**Generated:** 2026-03-14
**Project:** PageLens - website analysis toolkit built with Svelte and Rust

## Overview

Monorepo containing:

- **Frontend**: SvelteKit app in `apps/app`
- **App Services**: Rust orchestration layer in `crates/pagelens-app`
- **API**: Axum service in `crates/pagelens-api`
- **Core Engine**: Rust library (`pagelens-core`) for website analysis
- **CLI**: Rust CLI tool (`pagelens-cli`)
- **Test Apps**: Next.js and SvelteKit apps for integration testing

## Structure

```
pagelens/
├── apps/
│   └── app/                # SvelteKit frontend
├── crates/
│   ├── pagelens-app/       # Shared application services
│   ├── pagelens-api/       # HTTP API
│   ├── pagelens-core/      # Rust analysis engine
│   └── pagelens-cli/       # Rust CLI
├── packages/
│   └── ui/                 # Shared Svelte UI package
├── test-apps/              # Next.js & SvelteKit test sites
├── Cargo.toml              # Workspace root
└── package.json            # Bun workspace root
```

## Where to Look

| Task                  | Location                         | Notes                               |
| --------------------- | -------------------------------- | ----------------------------------- |
| Add web route         | `apps/app/src/routes/`           | SvelteKit file-based routing        |
| Add frontend UI       | `packages/ui/src/lib/`           | Shared Svelte components and stores |
| Add app service logic | `crates/pagelens-app/src/lib.rs` | Shared run orchestration            |
| Add API endpoint      | `crates/pagelens-api/src/lib.rs` | Axum router over app services       |
| Add core analysis     | `crates/pagelens-core/src/`      | See crate-level docs and tests      |
| Run integration tests | `crates/pagelens-core/tests/`    | Requires test apps running          |

## Conventions

### Rust

- Crates commonly expose `prelude.rs` and `error.rs`
- Errors use `thiserror`
- Integration tests live in `tests/` with shared helpers in `common/mod.rs`

### TypeScript / Svelte

- Svelte 5 with runes (`$state`, `$derived`, etc.)
- Tailwind CSS v4 with `@tailwindcss/vite`
- SvelteKit app in `apps/app`
- Shared UI lives in `packages/ui`

### Build System

- Cargo workspace covers `crates/*`
- Bun workspace covers `apps/*`, `packages/*`, and `test-apps/*`
- Turbo orchestrates `build` and `start`

## Commands

```bash
# Install dependencies
bun install

# Start the web app
bun run dev

# Typecheck the web app
bun run check

# Build test apps
bun turbo run build --filter="./test-apps/*"

# Start apps for integration tests
bun run start-apps

# Run Rust tests (requires test apps running)
cargo test -p pagelens-core -- --test-threads=1

# Build everything
bun turbo run build

# Format code
bun format
```

## Key Dependencies

| Layer        | Key Crates/Packages                             |
| ------------ | ----------------------------------------------- |
| Rust Core    | `chromey`, `tokio`, `thiserror`, `serde`        |
| Rust App/API | `axum`, `reqwest`, `tokio`, `serde`             |
| Rust CLI     | `clap`                                          |
| Frontend     | Svelte 5, Tailwind CSS 4, bits-ui, mode-watcher |

## Testing

- Unit tests are inline in Rust source files
- Integration tests live in `crates/pagelens-core/tests/`
- Test servers run on `:44791` and `:44792`
- Local runs skip gracefully when fixtures are unavailable; CI fails loudly

## Notes

- Browser automation uses `chromey` (Chromium DevTools Protocol)
- Test apps must be built before running integration tests
- Integration tests run sequentially to avoid Chromium port conflicts
- Each browser test gets an isolated `Browser::launch()` instance
- `pagelens-cli` still has a large monolithic `main.rs`
