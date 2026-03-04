use crate::browser::Browser;
use crate::prelude::*;
use futures::future::join_all;
use std::collections::{HashSet, VecDeque};
use std::time::Instant;

use super::aggregate;
use super::page;
use super::types::{CrawlOptions, CrawlResult, CrawlStats, CrawledPage};

pub struct Crawler<'a> {
    browser: &'a Browser,
    options: CrawlOptions,
}

impl<'a> Crawler<'a> {
    pub fn new(browser: &'a Browser, options: CrawlOptions) -> Self {
        Self { browser, options }
    }

    pub async fn crawl(&self, seed_url: &str) -> Result<CrawlResult> {
        self.crawl_with_callback(seed_url, |_page, _stats| {}).await
    }

    pub async fn crawl_with_callback<F>(
        &self,
        seed_url: &str,
        mut on_page: F,
    ) -> Result<CrawlResult>
    where
        F: FnMut(&CrawledPage, &CrawlStats),
    {
        let start_time = Instant::now();

        if !seed_url.starts_with("http://")
            && !seed_url.starts_with("https://")
            && !seed_url.starts_with("data:")
        {
            return Err(Error::InvalidUrl(seed_url.to_string()));
        }

        let mut result = CrawlResult {
            seed_url: seed_url.to_string(),
            pages: Vec::new(),
            skipped_urls: Vec::new(),
            stats: CrawlStats::default(),
            aggregate: Default::default(),
            summary: String::new(),
        };

        let mut visited: HashSet<String> = HashSet::new();
        let mut queued: HashSet<String> = HashSet::new();
        let mut to_visit: VecDeque<(String, usize)> = VecDeque::new();
        let mut total_page_load_ms: u64 = 0;

        to_visit.push_back((seed_url.to_string(), 0));
        queued.insert(seed_url.to_string());
        result.stats.total_pages = 1;
        result.stats.queued_pages = 1;

        let max_concurrency = self.options.max_concurrency.max(1);

        while !to_visit.is_empty() {
            let mut batch = Vec::new();

            while batch.len() < max_concurrency {
                let Some((url, depth)) = to_visit.pop_front() else {
                    break;
                };
                queued.remove(&url);

                if result.pages.len() + batch.len() >= self.options.max_pages {
                    result.skipped_urls.push(url);
                    continue;
                }

                if self.options.max_depth > 0 && depth > self.options.max_depth {
                    result.skipped_urls.push(url);
                    continue;
                }

                if visited.contains(&url) {
                    continue;
                }

                visited.insert(url.clone());
                batch.push((url, depth));
            }

            if batch.is_empty() {
                break;
            }

            result.stats.running_pages = batch.len();
            result.stats.queued_pages = to_visit.len();
            result.stats.total_pages = visited.len() + queued.len() + result.skipped_urls.len();

            let batch_len = batch.len();
            let batch_results = join_all(batch.into_iter().map(|(url, depth)| async move {
                let page_start = Instant::now();
                let crawled_page =
                    page::crawl_single_page(self.browser, &self.options, &url, depth).await;
                let page_load_ms = page_start.elapsed().as_millis() as u64;
                (url, depth, crawled_page, page_load_ms)
            }))
            .await;

            for (index, (url, depth, crawled_page, page_load_ms)) in
                batch_results.into_iter().enumerate()
            {
                total_page_load_ms += page_load_ms;

                if crawled_page.success {
                    result.stats.crawled_pages += 1;
                } else {
                    result.stats.failed_pages += 1;
                }

                result.stats.links_found_total += crawled_page.links_found.len();

                for link in &crawled_page.links_found {
                    if visited.contains(link) || queued.contains(link) {
                        continue;
                    }

                    if self.should_follow_link(&url, link) {
                        to_visit.push_back((link.clone(), depth + 1));
                        queued.insert(link.clone());
                    } else {
                        result.stats.external_links += 1;
                    }
                }

                result.pages.push(crawled_page);
                result.stats.running_pages = batch_len.saturating_sub(index + 1);
                result.stats.queued_pages = to_visit.len();
                result.stats.total_pages = visited.len() + queued.len() + result.skipped_urls.len();
                if let Some(last_page) = result.pages.last() {
                    on_page(last_page, &result.stats);
                }
            }

            if self.options.delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(self.options.delay_ms)).await;
            }
        }

        result.stats.crawl_time_ms = start_time.elapsed().as_millis() as u64;
        if !result.pages.is_empty() {
            result.stats.avg_page_load_ms = total_page_load_ms / result.pages.len() as u64;
        }

        result.aggregate = aggregate::calculate_aggregates(&result.pages);
        result.summary = aggregate::generate_summary(&result);

        Ok(result)
    }

    fn should_follow_link(&self, current_url: &str, target_url: &str) -> bool {
        if current_url.starts_with("data:") || target_url.starts_with("data:") {
            return true;
        }

        let current_parsed = url::Url::parse(current_url).ok();
        let target_parsed = url::Url::parse(target_url).ok();

        let (current_host, target_host) = match (&current_parsed, &target_parsed) {
            (Some(c), Some(t)) => (
                c.host_str().unwrap_or("").to_string(),
                t.host_str().unwrap_or("").to_string(),
            ),
            _ => return false,
        };

        let is_external = current_host != target_host;

        if is_external && !self.options.follow_external_links {
            return false;
        }

        if self.options.same_subdomain_only && current_host != target_host {
            return false;
        }

        true
    }
}
