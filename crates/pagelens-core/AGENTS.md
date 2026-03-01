# pagelens-core Knowledge Base

**Crate:** Core analysis engine for PageLens  
**Purpose:** Browser automation, SEO analysis, crawling, page snapshots

## Structure

```
crates/pagelens-core/
├── src/
│   ├── lib.rs           # Public exports
│   ├── browser.rs       # Browser/Page management (chromey wrapper)
│   ├── crawl.rs         # Web crawling logic
│   ├── error.rs         # Error types (thiserror)
│   ├── prelude.rs       # Result<T> type alias
│   ├── seo.rs           # SEO analysis (largest file ~900 lines)
│   └── snapshot.rs      # Page snapshot capture
└── tests/               # Integration tests
    ├── common/mod.rs    # Test helpers
    └── *_tests.rs       # Test modules
```

## Module Patterns

### Error Handling

All errors centralized in `error.rs`:

```rust
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Chromium binary not found...")]
    ChromiumNotFound,
    // ...
}
```

### Prelude Pattern

Every module uses crate prelude:

```rust
// In source files
use crate::prelude::*;  // Imports Error, Result<T>

// External consumers
use pagelens_core::prelude::*;
```

### Browser Usage

Always launch isolated browser per test/operation:

```rust
let browser = Browser::launch().await?;
let page = browser.navigate("https://...").await?;
let snapshot = page.snapshot(SnapshotOptions::default()).await?;
```

## Public API

| Type          | Export Path          | Purpose                      |
| ------------- | -------------------- | ---------------------------- |
| `Browser`     | `browser::Browser`   | Launch/control Chromium      |
| `Page`        | `browser::Page`      | Navigate, interact with page |
| `Crawler`     | `crawl::Crawler`     | Multi-page crawling          |
| `SeoAnalyzer` | `seo::SeoAnalyzer`   | Analyze SEO from snapshot    |
| `Snapshot`    | `snapshot::Snapshot` | Page content + metadata      |

## Testing

Integration tests require running test-apps:

```bash
# Terminal 1: Start test servers (ask the user if they have already started them for you)
bun run start-test-apps

# Terminal 2: Run tests (sequential required)
cargo test -p pagelens-core -- --test-threads=1
```

### Test Helpers (`tests/common/mod.rs`)

| Helper                                     | Purpose                         |
| ------------------------------------------ | ------------------------------- |
| `create_browser()`                         | Spawn isolated browser instance |
| `snapshot_from_url(url)`                   | Navigate + snapshot             |
| `seo_report_from_url(url)`                 | Full SEO analysis               |
| `nextjs_url()`, `svelte_url()`             | Get test app URLs               |
| `nextjs_available()`, `svelte_available()` | Check server health             |

### Test Pattern

```rust
#[tokio::test]
async fn test_something() {
    if !common::nextjs_available().await {
        common::skip_or_fail("Next.js");
        return;
    }

    let browser = common::create_browser().await;
    // ... test code
}
```

## Notes

- Uses `chromey` crate for Chrome DevTools Protocol
- All async operations use `tokio`
- Tests **must** run single-threaded to avoid Chromium port conflicts
- `common/mod.rs` has `#[allow(dead_code)]` for test-only helpers
