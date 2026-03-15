# PageLens

PageLens is a website analysis toolkit built with Rust and Svelte. It provides a web app, API, and CLI for SEO checks, crawl analysis, snapshots, performance signals, site-file inspection, favicon audits, and HTTP benchmarking.

## Monorepo Packages

- `apps/app`: SvelteKit web application
- `crates/pagelens-app`: shared application services and run orchestration
- `crates/pagelens-api`: HTTP API over the app service
- `crates/pagelens-core`: core Rust analysis engine
- `crates/pagelens-cli`: command-line interface
- `test-apps/nextjs` and `test-apps/svelte-kit`: integration test targets

## Requirements

- Bun `>= 1.3.0`
- Node.js `>= 20`
- Rust stable toolchain

Optional: use the pinned development environment from `flake.nix` with `nix develop`.

## Getting Started

Install dependencies:

```bash
bun install
```

Or enter the Nix development shell first:

```bash
nix develop
```

Start the web app in development:

```bash
bun run dev
```

Build test applications:

```bash
bun turbo run build --filter="./test-apps/*"
```

## Common Commands

```bash
# Build workspace packages
bun turbo run build

# Typecheck the web app
bun run check

# Start test applications used by Rust integration tests
bun run start-apps

# Run pagelens-core tests (sequential to avoid browser/CDP conflicts)
cargo test -p pagelens-core -- --test-threads=1

# Run CLI tests
cargo test -p pagelens-cli --tests
```

## CLI Releases

CLI releases are built from Git tags by `.github/workflows/release-cli.yml`.

`Cargo.toml` is the source of truth for CLI release versions. Release tags must match `workspace.package.version` there and use the format `cli-vX.Y.Z`.

The helper scripts in `scripts/` require Python 3.11+ because they read TOML with `tomllib`.

On Linux CI, the CLI release validation and Linux release build run inside the pinned Nix dev shell from `flake.nix` so GitHub Actions uses the same Rust/Bun environment as local development.

Release a new CLI version with:

```bash
# 1. Update workspace.package.version in Cargo.toml and commit it.

# 2. Verify the tag you are about to create matches Cargo.toml.
bun run release:cli:check-tag -- cli-v0.1.0

# 3. Create and push the release tag for the current Cargo.toml version.
bun run release:cli:create-tag
```

When the tag is pushed, GitHub Actions will:

- validate that the tag matches `workspace.package.version`
- verify the tagged commit is reachable from the default branch
- run `cargo test -p pagelens-cli --tests`
- build `pagelens` release archives for Linux, macOS, and Windows on GitHub
- attach the packaged binaries and `SHA256SUMS` to the GitHub Release

Release assets follow the pattern `pagelens-vX.Y.Z-<target>.<archive>`.

## Integration Testing

The integration suites in `crates/pagelens-core/tests` depend on running test apps:

- Next.js app: `http://localhost:44791`
- SvelteKit app: `http://localhost:44792`

If those services are not running, server-dependent tests fail in CI mode by design.

## Architecture Notes

- `pagelens-core` exposes browser automation, crawling, snapshot capture, SEO analysis, HTTP benchmarking, and site-file analysis.
- `pagelens-app` provides shared application services for the API layer.
- `pagelens-cli` builds on `pagelens-core` and adds terminal and JSON output modes.
- `apps/app` is the SvelteKit frontend workspace.

## Project Layout

```text
pagelens/
├── apps/
│   └── app/
├── crates/
│   ├── pagelens-app/
│   ├── pagelens-api/
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
