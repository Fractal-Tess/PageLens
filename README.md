# PageLens

PageLens is a website analysis toolkit built with Rust, Tauri, and Svelte. It provides a desktop app and CLI for SEO checks, crawl analysis, snapshots, performance signals, and site-file inspection.

## Monorepo Packages

- `apps/tauri`: Desktop application (Tauri backend + Svelte frontend)
- `crates/pagelens-core`: Core Rust analysis engine
- `crates/pagelens-cli`: Command-line interface
- `test-apps/nextjs` and `test-apps/svelte-kit`: Integration test targets

## Requirements

- Bun `>= 1.3.0`
- Node.js `>= 20`
- Rust stable toolchain

Optional (Linux): install `mold` linker, or remove related rust flags in `apps/tauri/src-tauri/.cargo/config.toml`.

## Getting Started

Install dependencies:

```bash
bun install
```

Build test applications:

```bash
bun turbo run build --filter="./test-apps/*"
```

Start desktop app in development:

```bash
bun run dev
```

## Common Commands

```bash
# Build workspace packages
bun turbo run build

# Start test applications used by Rust integration tests
bun run start-apps

# Run pagelens-core tests (sequential to avoid browser/CDP conflicts)
cargo test -p pagelens-core -- --test-threads=1

# Run CLI tests
cargo test -p pagelens-cli --tests
```

## Integration Testing

The integration suites in `crates/pagelens-core/tests` depend on running test apps:

- Next.js app: `http://localhost:44791`
- SvelteKit app: `http://localhost:44792`

If those services are not running, server-dependent tests fail in CI mode by design.

## Architecture Notes

- `pagelens-core` exposes browser automation, crawling, snapshot capture, SEO analysis, HTTP benchmarking, and site-file analysis.
- `pagelens-cli` builds on `pagelens-core` and adds terminal and JSON output modes.
- Tauri IPC types are generated to `apps/tauri/src/lib/ipc.ts` and should not be edited manually.

## Project Layout

```text
pagelens/
├── apps/
│   └── tauri/
├── crates/
│   ├── pagelens-core/
│   └── pagelens-cli/
├── test-apps/
│   ├── nextjs/
│   └── svelte-kit/
├── package.json
└── Cargo.toml
```

For additional repository details, see `MONOREPO.md`.

## License

MIT
