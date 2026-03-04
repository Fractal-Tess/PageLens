use crate::prelude::*;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpBenchmarkOptions {
    pub requests: usize,
    /// If set, run for this many seconds instead of a fixed request count.
    pub duration_secs: Option<f64>,
    pub connections: usize,
    pub method: String,
    pub timeout_ms: Option<u64>,
    pub qps: Option<f64>,
    pub headers: Vec<(String, String)>,
    pub cookies: Vec<(String, String)>,
    pub body: Option<String>,
    pub follow_redirects: bool,
}

impl Default for HttpBenchmarkOptions {
    fn default() -> Self {
        Self {
            requests: 200,
            duration_secs: None,
            connections: 50,
            method: "GET".to_string(),
            timeout_ms: None,
            qps: None,
            headers: Vec::new(),
            cookies: Vec::new(),
            body: None,
            follow_redirects: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpBenchmarkRequestResult {
    pub status_code: Option<u16>,
    pub latency_ms: f64,
    pub success: bool,
    pub error: Option<String>,
    #[serde(default)]
    pub response_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpBenchmarkLatencyStats {
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
    #[serde(default)]
    pub p10_ms: f64,
    #[serde(default)]
    pub p25_ms: f64,
    pub p50_ms: f64,
    #[serde(default)]
    pub p75_ms: f64,
    #[serde(default)]
    pub p90_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    #[serde(default)]
    pub p99_9_ms: f64,
}

impl Default for HttpBenchmarkLatencyStats {
    fn default() -> Self {
        Self {
            min_ms: 0.0,
            max_ms: 0.0,
            avg_ms: 0.0,
            p10_ms: 0.0,
            p25_ms: 0.0,
            p50_ms: 0.0,
            p75_ms: 0.0,
            p90_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
            p99_9_ms: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpBenchmarkResult {
    pub url: String,
    pub method: String,
    /// Actual number of requests made (may differ from target in duration mode).
    pub requests: usize,
    pub connections: usize,
    pub duration_ms: u64,
    pub requests_per_sec: f64,
    pub successful_requests: usize,
    pub failed_requests: usize,
    /// Fraction 0.0–1.0
    #[serde(default)]
    pub success_rate: f64,
    pub status_code_distribution: BTreeMap<String, usize>,
    #[serde(default)]
    pub error_distribution: BTreeMap<String, usize>,
    pub latency: HttpBenchmarkLatencyStats,
    pub latency_histogram: Vec<(f64, usize)>,
    pub samples: Vec<HttpBenchmarkRequestResult>,
    // Data transfer
    #[serde(default)]
    pub total_data_bytes: u64,
    #[serde(default)]
    pub avg_size_per_request_bytes: f64,
    #[serde(default)]
    pub data_per_sec_bytes: f64,
    // Mode metadata
    #[serde(default)]
    pub is_duration_mode: bool,
    #[serde(default)]
    pub target_duration_secs: Option<f64>,
}

pub struct HttpBenchmarker {
    client: reqwest::Client,
}

impl HttpBenchmarker {
    pub fn new(options: &HttpBenchmarkOptions) -> Result<Self> {
        let redirect_policy = if options.follow_redirects {
            reqwest::redirect::Policy::limited(10)
        } else {
            reqwest::redirect::Policy::none()
        };

        let mut builder = reqwest::Client::builder()
            .redirect(redirect_policy)
            .connect_timeout(Duration::from_secs(5))
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd();

        if !options.headers.is_empty() || !options.cookies.is_empty() {
            let mut header_map = reqwest::header::HeaderMap::new();
            for (k, v) in &options.headers {
                if let (Ok(name), Ok(value)) = (
                    reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                    reqwest::header::HeaderValue::from_str(v),
                ) {
                    header_map.insert(name, value);
                }
            }
            if !options.cookies.is_empty() {
                let cookie_line = options
                    .cookies
                    .iter()
                    .filter_map(|(k, v)| {
                        let key = k.trim();
                        if key.is_empty() {
                            None
                        } else {
                            Some(format!("{key}={}", v.trim()))
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                if !cookie_line.is_empty() {
                    if let Ok(cookie_header) = reqwest::header::HeaderValue::from_str(&cookie_line)
                    {
                        header_map.insert(reqwest::header::COOKIE, cookie_header);
                    }
                }
            }
            builder = builder.default_headers(header_map);
        }

        let client = builder
            .build()
            .map_err(|err| Error::HttpRequestFailed(err.to_string()))?;
        Ok(Self { client })
    }

    pub async fn benchmark_with_callback<F>(
        &self,
        url: &str,
        options: HttpBenchmarkOptions,
        cancel_requested: Arc<AtomicBool>,
        mut on_progress: F,
    ) -> Result<HttpBenchmarkResult>
    where
        F: FnMut(usize, usize, usize, &HttpBenchmarkRequestResult),
    {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(Error::InvalidUrl(url.to_string()));
        }

        let is_duration_mode = options.duration_secs.filter(|&d| d > 0.0).is_some();
        // total = 0 signals "duration mode" to the callback.
        let target_requests = if is_duration_mode { 0 } else { options.requests.max(1) };
        let connections = options.connections.max(1);
        let method = Method::from_bytes(options.method.as_bytes())
            .map_err(|err| Error::HttpRequestFailed(err.to_string()))?;
        let timeout = options.timeout_ms.filter(|&v| v > 0).map(Duration::from_millis);
        let body_opt = options.body.clone();
        let inject_accept_encoding = !options
            .headers
            .iter()
            .any(|(k, _)| k.trim().eq_ignore_ascii_case("accept-encoding"));
        let start = Instant::now();
        let benchmark_deadline: Option<Instant> = options
            .duration_secs
            .filter(|&d| d > 0.0)
            .map(|d| start + Duration::from_secs_f64(d));

        // Channel sizing: small buffer for duration mode (producer is live),
        // full capacity for request mode (producer pre-fills).
        let work_cap = if is_duration_mode { connections * 4 } else { target_requests };
        let result_cap = if is_duration_mode { connections * 16 } else { target_requests + connections };

        let (work_tx, work_rx) = mpsc::channel::<usize>(work_cap.max(8));
        let work_rx = Arc::new(Mutex::new(work_rx));
        let (result_tx, mut result_rx) =
            mpsc::channel::<HttpBenchmarkRequestResult>(result_cap.max(8));

        let mut producer = Some(tokio::spawn({
            let work_tx = work_tx;
            let qps = options.qps;
            let duration_secs = options.duration_secs;
            let cancel_requested = Arc::clone(&cancel_requested);
            async move {
                let schedule_start = Instant::now();
                let deadline = duration_secs
                    .filter(|&d| d > 0.0)
                    .map(|d| schedule_start + Duration::from_secs_f64(d));
                let mut i = 0usize;
                loop {
                    if cancel_requested.load(Ordering::Relaxed) {
                        break;
                    }
                    if let Some(dl) = deadline {
                        if Instant::now() >= dl {
                            break;
                        }
                    } else if i >= target_requests {
                        break;
                    }
                    if let Some(target_qps) = qps.filter(|q| *q > 0.0) {
                        let target = schedule_start
                            + Duration::from_secs_f64((i as f64 + 1.0) / target_qps);
                        tokio::time::sleep_until(target.into()).await;
                        if cancel_requested.load(Ordering::Relaxed) {
                            break;
                        }
                        // Re-check deadline after sleep.
                        if let Some(dl) = deadline {
                            if Instant::now() >= dl {
                                break;
                            }
                        }
                    }
                    if work_tx.send(i).await.is_err() {
                        break;
                    }
                    i += 1;
                }
            }
        }));

        let mut workers = Vec::with_capacity(connections);
        for _ in 0..connections {
            let client = self.client.clone();
            let work_rx = Arc::clone(&work_rx);
            let result_tx = result_tx.clone();
            let method = method.clone();
            let url = url.to_string();
            let body_opt = body_opt.clone();
            let inject_accept_encoding = inject_accept_encoding;
            let benchmark_deadline = benchmark_deadline;
            let cancel_requested = Arc::clone(&cancel_requested);

            workers.push(tokio::spawn(async move {
                loop {
                    if cancel_requested.load(Ordering::Relaxed) {
                        break;
                    }
                    let next = {
                        let mut rx = work_rx.lock().await;
                        rx.recv().await
                    };
                    if next.is_none() {
                        break;
                    }

                    if let Some(deadline) = benchmark_deadline {
                        if Instant::now() >= deadline {
                            break;
                        }
                    }
                    if cancel_requested.load(Ordering::Relaxed) {
                        break;
                    }

                    let request_start = Instant::now();
                    let send_result = async {
                        let mut req = client.request(method.clone(), &url);
                        if inject_accept_encoding {
                            req = req.header(reqwest::header::ACCEPT_ENCODING, "gzip, compress, deflate, br");
                        }
                        if let Some(ref body) = body_opt {
                            req = req.body(body.clone());
                        }
                        let mut response = req.send().await?;
                        let status = response.status();
                        let mut size = 0u64;
                        while let Some(chunk) = response.chunk().await? {
                            size = size.saturating_add(chunk.len() as u64);
                        }
                        Ok::<_, reqwest::Error>((status, size))
                    };

                    let remaining_deadline = benchmark_deadline
                        .and_then(|deadline| deadline.checked_duration_since(Instant::now()));

                    if benchmark_deadline.is_some() && remaining_deadline.is_none() {
                        break;
                    }

                    let effective_timeout = match (timeout, remaining_deadline) {
                        (Some(base), Some(rem)) => Some(base.min(rem)),
                        (Some(base), None) => Some(base),
                        (None, Some(rem)) => Some(rem),
                        (None, None) => None,
                    };
                    let timeout_due_to_deadline_cap = match (timeout, remaining_deadline) {
                        (None, Some(_)) => true,
                        (Some(base), Some(rem)) => rem <= base,
                        _ => false,
                    };

                    let request_result = if let Some(request_timeout) = effective_timeout {
                        match tokio::time::timeout(request_timeout, send_result).await {
                            Ok(Ok((status, size))) => HttpBenchmarkRequestResult {
                                status_code: Some(status.as_u16()),
                                latency_ms: request_start.elapsed().as_secs_f64() * 1000.0,
                                success: true,
                                error: None,
                                response_size_bytes: Some(size),
                            },
                            Ok(Err(err)) => HttpBenchmarkRequestResult {
                                status_code: None,
                                latency_ms: request_start.elapsed().as_secs_f64() * 1000.0,
                                success: false,
                                error: Some(err.to_string()),
                                response_size_bytes: None,
                            },
                            Err(_) => {
                                if timeout_due_to_deadline_cap {
                                    continue;
                                }
                                HttpBenchmarkRequestResult {
                                    status_code: None,
                                    latency_ms: request_start.elapsed().as_secs_f64() * 1000.0,
                                    success: false,
                                    error: Some(format!(
                                        "Request timed out after {}ms",
                                        request_timeout.as_millis()
                                    )),
                                    response_size_bytes: None,
                                }
                            }
                        }
                    } else {
                        match send_result.await {
                            Ok((status, size)) => HttpBenchmarkRequestResult {
                                status_code: Some(status.as_u16()),
                                latency_ms: request_start.elapsed().as_secs_f64() * 1000.0,
                                success: true,
                                error: None,
                                response_size_bytes: Some(size),
                            },
                            Err(err) => HttpBenchmarkRequestResult {
                                status_code: None,
                                latency_ms: request_start.elapsed().as_secs_f64() * 1000.0,
                                success: false,
                                error: Some(err.to_string()),
                                response_size_bytes: None,
                            },
                        }
                    };

                    if result_tx.send(request_result).await.is_err() {
                        break;
                    }
                }
            }));
        }
        drop(result_tx);

        // Drain results concurrently with running workers.
        let mut samples = Vec::new();
        let mut live_successful = 0usize;
        let mut live_failed = 0usize;
        while let Some(item) = result_rx.recv().await {
            if item.success {
                live_successful += 1;
            } else {
                live_failed += 1;
            }
            samples.push(item);
            if let Some(latest) = samples.last() {
                on_progress(live_successful, live_failed, target_requests, latest);
            }
        }

        if let Some(handle) = producer.take() {
            let _ = handle.await;
        }
        for worker in workers {
            let _ = worker.await;
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        let successful_requests = samples.iter().filter(|s| s.success).count();
        let failed_requests = samples.len().saturating_sub(successful_requests);
        let success_rate = if samples.is_empty() {
            0.0
        } else {
            successful_requests as f64 / samples.len() as f64
        };

        let mut status_code_distribution: BTreeMap<String, usize> = BTreeMap::new();
        let mut error_distribution: BTreeMap<String, usize> = BTreeMap::new();
        for sample in &samples {
            if let Some(code) = sample.status_code {
                *status_code_distribution
                    .entry(code.to_string())
                    .or_insert(0) += 1;
            }

            if let Some(ref err) = sample.error {
                let normalized = if err.contains("timed out") || err.contains("timeout") {
                    "timeout".to_string()
                } else if err.contains("connection refused") {
                    "connection refused".to_string()
                } else if err.contains("connection reset") {
                    "connection reset".to_string()
                } else if err.to_lowercase().contains("dns") {
                    "dns error".to_string()
                } else if err.contains("certificate") || err.contains("tls") || err.contains("ssl") {
                    "tls/certificate error".to_string()
                } else if err.to_lowercase().contains("redirect") {
                    "redirection limit reached".to_string()
                } else {
                    err.chars().take(60).collect()
                };
                *error_distribution.entry(normalized).or_insert(0) += 1;
            }
        }

        let mut latencies: Vec<f64> = samples.iter().map(|s| s.latency_ms).collect();
        latencies.sort_by(|a, b| a.total_cmp(b));

        let latency = latency_stats(&latencies);
        let latency_histogram = histogram(&latencies, 11);
        let requests_per_sec = if duration_ms == 0 {
            0.0
        } else {
            (samples.len() as f64) / (duration_ms as f64 / 1000.0)
        };

        // Data transfer stats.
        let total_data_bytes: u64 = samples
            .iter()
            .filter(|s| s.success)
            .filter_map(|s| s.response_size_bytes)
            .sum();
        let sized_response_count = samples
            .iter()
            .filter(|s| s.success && s.response_size_bytes.is_some())
            .count();
        let avg_size_per_request_bytes = if sized_response_count > 0 {
            total_data_bytes as f64 / sized_response_count as f64
        } else {
            0.0
        };
        let data_per_sec_bytes = if duration_ms == 0 {
            0.0
        } else {
            total_data_bytes as f64 / (duration_ms as f64 / 1000.0)
        };

        Ok(HttpBenchmarkResult {
            url: url.to_string(),
            method: options.method,
            requests: samples.len(),
            connections,
            duration_ms,
            requests_per_sec,
            successful_requests,
            failed_requests,
            success_rate,
            status_code_distribution,
            error_distribution,
            latency,
            latency_histogram,
            samples,
            total_data_bytes,
            avg_size_per_request_bytes,
            data_per_sec_bytes,
            is_duration_mode,
            target_duration_secs: options.duration_secs.filter(|&d| d > 0.0),
        })
    }
}

fn latency_stats(latencies: &[f64]) -> HttpBenchmarkLatencyStats {
    if latencies.is_empty() {
        return HttpBenchmarkLatencyStats::default();
    }

    let min_ms = *latencies.first().unwrap_or(&0.0);
    let max_ms = *latencies.last().unwrap_or(&0.0);
    let avg_ms = latencies.iter().sum::<f64>() / latencies.len() as f64;

    HttpBenchmarkLatencyStats {
        min_ms,
        max_ms,
        avg_ms,
        p10_ms: percentile(latencies, 0.10),
        p25_ms: percentile(latencies, 0.25),
        p50_ms: percentile(latencies, 0.50),
        p75_ms: percentile(latencies, 0.75),
        p90_ms: percentile(latencies, 0.90),
        p95_ms: percentile(latencies, 0.95),
        p99_ms: percentile(latencies, 0.99),
        p99_9_ms: percentile(latencies, 0.999),
    }
}

fn percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    let rank = ((sorted_values.len() as f64 - 1.0) * percentile).round() as usize;
    sorted_values.get(rank).copied().unwrap_or(0.0)
}

fn histogram(values: &[f64], bins: usize) -> Vec<(f64, usize)> {
    if values.is_empty() {
        return Vec::new();
    }

    let min = values[0];
    let max = values[values.len() - 1];
    if (max - min).abs() < f64::EPSILON {
        return vec![(max, values.len())];
    }

    let bins = bins.max(1);
    let step = (max - min) / bins as f64;
    let mut counts = vec![0usize; bins];

    for value in values {
        let mut idx = ((value - min) / step).floor() as usize;
        if idx >= bins {
            idx = bins - 1;
        }
        counts[idx] += 1;
    }

    counts
        .into_iter()
        .enumerate()
        .map(|(idx, count)| (min + step * (idx as f64 + 1.0), count))
        .collect()
}
