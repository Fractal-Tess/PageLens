# PageLens

A website analysis tool built with Tauri, Svelte, and Rust.

<div align="center">
  <img src="https://raw.githubusercontent.com/Fractal-Tess/Svelte-Tauri/dev/.github/app.jpeg" width="580" style="border-radius:2rem"/>
</div>

## Overview

PageLens is a monorepo containing:

- **Web App** (`@pagelens/web`): Tauri + Svelte desktop application
- **Core Engine** (`pagelens-core`): Rust library for website analysis
- **CLI** (`pagelens-cli`): Command-line interface
- **Test Apps** (`@pagelens/test-apps-*`): Next.js and SvelteKit test sites

## Tech Stack

- **Frontend**: TypeScript, Svelte, Tailwind CSS, shadcn-svelte
- **Backend**: Rust, Tauri
- **Build**: Vite, Bun, Turbo
- **Testing**: Rust test suite with real application integration tests

## Quick Start

### Prerequisites

- [Bun](https://bun.sh/) >= 1.3.0
- [Node.js](https://nodejs.org/) >= 20.0.0
- [Rust](https://rustup.rs/)

### Installation

```bash
# Install dependencies
bun install

# Build test applications
bun turbo run build --filter="./test-apps/*"
```

### Development

```bash
# Run the main Tauri application
bun tauri dev
```

## Monorepo Structure

This is a monorepo using **Bun** and **Turbo**.

```
pagelens/
├── apps/                       # Future applications
├── crates/
│   ├── pagelens-core/          # Rust analysis engine
│   └── pagelens-cli/           # Rust CLI
├── test-apps/
│   ├── nextjs/                 # @pagelens/test-apps-nextjs
│   └── svelte-kit/             # @pagelens/test-apps-svelte
├── src/                        # Main Svelte app source
├── src-tauri/                  # Tauri Rust source
├── package.json                # Root workspace
├── bun.lockb                   # Lockfile
└── turbo.json                  # Turbo config
```

See [MONOREPO.md](./MONOREPO.md) for detailed documentation.

## Testing

### Integration Tests

Run tests against real production applications:

```bash
# Full automated test (builds, serves, tests)
bun run test:integration

# Or manually:
bun run prod                    # Terminal 1: Start servers
bun run test:integration        # Terminal 2: Run tests
```

### Unit Tests

```bash
# Rust core tests
cargo test -p pagelens-core -- --test-threads=1
```

## Available Scripts

```bash
bun install                      # Install dependencies
bun turbo run build             # Build all packages
bun run prod                    # Start test servers
bun run test:integration        # Run integration tests
```

## Requirements

- (optional) On Linux you need to have installed the [mold](https://github.com/rui314/mold) linker. If you prefer not to do that, go ahead and remove the rust flags in the `/src-tauri/.cargo/config.toml` file.

## License

MIT
