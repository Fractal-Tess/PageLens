pub mod browser;
pub mod error;
pub mod seo;
pub mod snapshot;
mod prelude;

pub use browser::{Browser, ChromiumLocator, Page};
pub use error::Error;
pub use seo::{SeoAnalyzer, SeoReport, Severity, Issue};
pub use snapshot::{Snapshot, SnapshotOptions, SnapshotExt};
