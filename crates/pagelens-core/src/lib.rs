pub mod browser;
pub mod crawl;
pub mod error;
pub mod http_benchmark;
mod prelude;
pub mod seo;
pub mod site_files;
pub mod snapshot;

pub use browser::{Browser, ChromiumLocator, Page};
pub use crawl::{CrawlOptions, CrawlResult, CrawledPage, Crawler};
pub use error::Error;
pub use http_benchmark::{
    HttpBenchmarkLatencyStats, HttpBenchmarkOptions, HttpBenchmarkRequestResult,
    HttpBenchmarkResult, HttpBenchmarker,
};
pub use seo::{Issue, ScoreConfig, SeoAnalyzer, SeoReport, Severity};
pub use site_files::{
    MiscFileReport, RobotsReport, SiteFileIssue, SiteFilesAnalyzer, SiteFilesReport,
    SitemapCrawlDiff, SitemapKind, SitemapReport,
};
pub use snapshot::{ReferencedAssets, Snapshot, SnapshotExt, SnapshotOptions};
