export const BASE_URL = "http://127.0.0.1:8787";

export interface AnalysisOptions {
  include_html?: boolean;
  include_accessibility_tree?: boolean;
  include_performance_timing?: boolean;
  include_computed_styles?: boolean;
  include_network_metadata?: boolean;
}

export interface StartAnalysisRequest {
  run_id?: string;
  url: string;
  name?: string;
  analysis_type?: "single" | "crawl" | "http_benchmark" | "favicon";
  options?: AnalysisOptions;
  crawl_options?: CrawlAnalysisOptions;
  benchmark_options?: BenchmarkOptions;
}

export interface CrawlAnalysisOptions {
  max_pages?: number;
  max_depth?: number;
  follow_external_links?: boolean;
  same_subdomain_only?: boolean;
  page_timeout_ms?: number;
  delay_ms?: number;
  max_concurrency?: number;
}

export interface BenchmarkOptions {
  duration_secs?: number;
  connections?: number;
  method?: string;
  qps?: number;
  headers?: Array<[string, string]>;
  cookies?: Array<[string, string]>;
  body?: string;
  follow_redirects?: boolean;
}

export interface StartAnalysisResponse {
  run_id: string;
}

export interface FaviconCandidate {
  url: string;
  rel: string;
  sizes: string | null;
  mime_type: string | null;
  media: string | null;
  source: "html_link" | "default_path" | string;
}

export interface FaviconAnalysisResult {
  schema_version: string;
  rule_version: string;
  input_url: string;
  resolved_page_url: string;
  default_favicon_url: string;
  candidates: FaviconCandidate[];
  warnings: string[];
  has_web_app_manifest: boolean;
  has_touch_icon: boolean;
  touch_web_app_title: string | null;
  has_svg_favicon: boolean;
  has_desktop_png_favicon: boolean;
  ico_declared: boolean;
  ico_found: boolean;
  ico_sizes: string[];
  ico_extra_sizes: string[];
  ico_missing_sizes: string[];
  audit_messages: string[];
  candidate_reports: Array<{
    url: string;
    rel: string;
    source: string;
    mime_type: string | null;
    media: string | null;
    preferred_theme: string | null;
    declared_sizes: string | null;
    format: string | null;
    file_size_bytes: number | null;
    detected_dimensions: string[];
    contrast_on_light: number | null;
    contrast_on_dark: number | null;
    issues: string[];
    recommendations: string[];
  }>;
  global_recommendations: string[];
  report_v2: {
    generated_at: string;
    inventory: Array<{
      asset_id: string;
      source: string;
      rel: string;
      href: string;
      final_url: string | null;
      media: string | null;
      theme: "light" | "dark" | "light_dark" | "any";
      declared_mime_type: string | null;
      declared_sizes: string | null;
      format: string | null;
      parsed_sizes: string[];
      file_size_bytes: number | null;
      fetch_status: string;
      contrast_on_light: number | null;
      contrast_on_dark: number | null;
    }>;
    checks: Array<{
      check_id: string;
      rule_id: string;
      category: string;
      title: string;
      subject_type: string;
      subject_id: string;
      status: "pass" | "warn" | "fail" | "info" | "not_applicable";
      severity: "critical" | "high" | "medium" | "low" | "info";
      confidence: number;
      user_impact: string;
      details: string;
      evidence: Array<{ kind: string; value: string }>;
      recommendation_ids: string[];
    }>;
    recommendations: Array<{
      recommendation_id: string;
      action_key: string;
      priority: "critical" | "high" | "medium" | "low" | "info";
      title: string;
      why: string[];
      fix_steps: string[];
      snippets: Array<{ language: string; content: string }>;
      related_check_ids: string[];
      applies_to_assets: string[];
      priority_score: number;
    }>;
    score: {
      score_0_100: number;
      grade: string;
      highlights: string[];
    };
    top_actions: string[];
  };
}

export interface PwaManifestSummary {
  manifest_url: string;
  name: string | null;
  short_name: string | null;
  start_url: string | null;
  display: string | null;
  theme_color: string | null;
  background_color: string | null;
  icon_count: number;
}

export interface PwaAnalysisResult {
  input_url: string;
  resolved_page_url: string;
  has_manifest: boolean;
  manifest: PwaManifestSummary | null;
  has_service_worker_registration: boolean;
  has_theme_color_meta: boolean;
  apple_touch_icon_count: number;
  mask_icon_count: number;
  installability_score: number;
  warnings: string[];
}

export interface AnalysisSummary {
  seo_score: number | null;
  page_count: number;
  total_issues: number;
  error_count: number;
  warning_count: number;
  duration_ms: number;
}

export interface AnalysisRun {
  id: string;
  url: string;
  analysis_type: "single" | "crawl" | "http_benchmark" | "favicon";
  name?: string;
  // DB serializes as "pending" | "running" | "completed" | "failed"
  status: "pending" | "running" | "completed" | "failed";
  progress?: number;
  summary?: AnalysisSummary;
  payload_json?: string;
  created_at?: string;
  updated_at?: string;
  current_stage?: string;
  current_message?: string;
}

export interface AnalysisPageResult {
  id: string;
  run_id: string;
  url: string;
  depth: number;
  success: boolean;
  seo_score: number | null;
  total_issues: number;
  error_count: number;
  warning_count: number;
  links_found_count: number;
  error_message: string | null;
  analyzed_at: string;
}

export interface HistoryListItem {
  id: string;
  url: string;
  name?: string;
  analysis_type: "single" | "crawl" | "http_benchmark" | "favicon";
  status: "pending" | "running" | "completed" | "failed";
  summary: AnalysisSummary;
  created_at: string;
}

export interface RunEvent {
  kind: "progress" | "page" | "complete" | "failed" | "cancelled";
  run_id: string;
  stage: string | null;
  message: string | null;
  progress: number | null;
  page_count: number | null;
  success_count: number | null;
  failed_count: number | null;
  page: AnalysisPageResult | null;
  links_found: string[] | null;
  discovered_count: number | null;
  queued_count: number | null;
  running_count: number | null;
  scanned_count: number | null;
  links_found_total: number | null;
  live_latency_min_ms?: number;
  live_latency_max_ms?: number;
  live_latency_avg_ms?: number;
  live_latency_p10_ms?: number;
  live_latency_p25_ms?: number;
  live_latency_p50_ms?: number;
  live_latency_p75_ms?: number;
  live_latency_p90_ms?: number;
  live_latency_p95_ms?: number;
  live_latency_p99_ms?: number;
  live_latency_p99_9_ms?: number;
  live_histogram?: Array<[number, number]>;
  live_total_data_bytes?: number;
  live_avg_size_per_request_bytes?: number;
  live_data_per_sec_bytes?: number;
  live_method?: string;
  live_connections?: number;
  live_target_duration_secs?: number;
}

// Snapshot / SEO payload types (parsed from AnalysisRun.payload_json)
export interface PerformanceTiming {
  navigation_start: number;
  dom_interactive: number | null;
  dom_content_loaded: number | null;
  load_complete: number | null;
  response_start: number | null;
  first_paint: number | null;
  first_contentful_paint: number | null;
  largest_contentful_paint: number | null;
  cumulative_layout_shift: number | null;
  interaction_to_next_paint: number | null;
}

export interface MainResourceNetwork {
  final_url: string | null;
  status_code: number | null;
  headers: Record<string, string>;
  fetch_error: string | null;
}

export interface ReferencedAssets {
  javascript: string[];
  stylesheets: string[];
  media: string[];
  fonts: string[];
}

export interface Snapshot {
  url: string;
  title: string;
  html: string;
  favicon_url: string | null;
  performance_timing: PerformanceTiming;
  main_resource_network: MainResourceNetwork;
  referenced_assets: ReferencedAssets;
  network_requests: Array<{
    url: string;
    resource_type?: string;
    status_code?: number;
    duration_ms?: number;
    from_cache?: boolean;
    failed?: boolean;
  }>;
  capture_issues: Array<{ stage: string; message: string }>;
}

export interface SeoIssue {
  severity: "error" | "warning" | "info";
  message: string;
  category: string;
}

export interface SeoReport {
  score: number;
  meta: {
    charset: boolean;
    viewport: boolean;
    title: string | null;
    description: string | null;
    language: string | null;
    robots: string | null;
  };
  open_graph: {
    title: string | null;
    description: string | null;
    og_type: string | null;
    url: string | null;
    image: string | null;
  };
  twitter_card: {
    card: string | null;
    title: string | null;
    description: string | null;
    image: string | null;
  };
  canonical_url: string | null;
  favicon_url: string | null;
  headings: {
    h1_count: number;
    h2_count: number;
    h3_count: number;
    h4_count: number;
    h5_count: number;
    h6_count: number;
    structure: string[];
  };
  images: Array<{ src: string; alt: string | null; has_alt: boolean }>;
  structured_data: Array<{ schema_type: string; raw: string }>;
  issues: SeoIssue[];
}

export interface SingleRunPayload {
  snapshot: Snapshot;
  seo_report: SeoReport;
}

export interface CrawlRunPayload {
  seed_url: string;
  pages: Array<{
    url: string;
    snapshot: Snapshot;
    seo_report: SeoReport;
    depth: number;
    links_found: string[];
    success: boolean;
    error: string | null;
  }>;
  skipped_urls: string[];
  stats: {
    total_pages: number;
    crawled_pages: number;
    failed_pages: number;
    external_links: number;
    crawl_time_ms: number;
    avg_page_load_ms: number;
  };
  aggregate: {
    avg_seo_score: number;
    min_seo_score: number;
    max_seo_score: number;
    total_issues: number;
    total_errors: number;
    total_warnings: number;
    score_distribution: number[];
  };
}

export interface HttpBenchmarkRequestSample {
  status_code: number | null;
  latency_ms: number;
  success: boolean;
  error: string | null;
  response_size_bytes?: number | null;
}

export interface HttpBenchmarkRunPayload {
  url: string;
  method: string;
  requests: number;
  connections: number;
  duration_ms: number;
  requests_per_sec: number;
  successful_requests: number;
  failed_requests: number;
  /** Fraction 0.0–1.0; absent on older records */
  success_rate?: number;
  status_code_distribution: Record<string, number>;
  /** Error type → count; absent on older records */
  error_distribution?: Record<string, number>;
  latency: {
    min_ms: number;
    max_ms: number;
    avg_ms: number;
    p10_ms?: number;
    p25_ms?: number;
    p50_ms: number;
    p75_ms?: number;
    p90_ms?: number;
    p95_ms: number;
    p99_ms: number;
    p99_9_ms?: number;
  };
  latency_histogram: Array<[number, number]>;
  samples: HttpBenchmarkRequestSample[];
  total_data_bytes?: number;
  avg_size_per_request_bytes?: number;
  data_per_sec_bytes?: number;
  is_duration_mode?: boolean;
  target_duration_secs?: number | null;
}

export type RunPayload =
  | SingleRunPayload
  | CrawlRunPayload
  | HttpBenchmarkRunPayload
  | FaviconAnalysisResult;

export function isCrawlRunPayload(
  payload: RunPayload,
): payload is CrawlRunPayload {
  return "pages" in payload;
}

export function isHttpBenchmarkRunPayload(
  payload: RunPayload,
): payload is HttpBenchmarkRunPayload {
  return "requests_per_sec" in payload && "latency" in payload;
}

export function isFaviconRunPayload(
  payload: RunPayload,
): payload is FaviconAnalysisResult {
  return "schema_version" in payload && "candidate_reports" in payload;
}

export function runResultPath(
  runId: string,
  analysisType:
    | AnalysisRun["analysis_type"]
    | StartAnalysisRequest["analysis_type"],
): string {
  if (analysisType === "favicon") return `/favicon/analyse/${runId}`;
  return `/run/${runId}`;
}

export function parseRunPayload(payloadJson: string): RunPayload | null {
  try {
    return JSON.parse(payloadJson) as RunPayload;
  } catch {
    return null;
  }
}

export async function startAnalysis(
  req: StartAnalysisRequest,
): Promise<StartAnalysisResponse> {
  const res = await fetch(`${BASE_URL}/api/runs/analyse`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    throw new Error(await apiErrorMessage(res, "Failed to start analysis"));
  }
  return res.json();
}

export async function analyzeFavicon(
  url: string,
): Promise<FaviconAnalysisResult> {
  const res = await fetch(`${BASE_URL}/api/tools/favicon`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ url }),
  });
  if (!res.ok) {
    throw new Error(await apiErrorMessage(res, "Failed to analyze favicon"));
  }
  return res.json();
}

export async function analyzePwa(url: string): Promise<PwaAnalysisResult> {
  const res = await fetch(`${BASE_URL}/api/tools/pwa`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ url }),
  });
  if (!res.ok) throw new Error(`Failed to analyze PWA: ${res.status}`);
  return res.json();
}

export async function getRun(runId: string): Promise<AnalysisRun> {
  const res = await fetch(`${BASE_URL}/api/runs/${runId}`);
  if (!res.ok) throw new Error(await apiErrorMessage(res, "Failed to get run"));
  return res.json();
}

export async function getRunPages(
  runId: string,
): Promise<AnalysisPageResult[]> {
  const res = await fetch(`${BASE_URL}/api/runs/${runId}/pages`);
  if (!res.ok) {
    throw new Error(await apiErrorMessage(res, "Failed to get run pages"));
  }
  return res.json();
}

async function apiErrorMessage(
  response: Response,
  fallbackPrefix: string,
): Promise<string> {
  try {
    const body = (await response.json()) as { error?: string };
    const message = body.error?.trim();
    if (message) return message;
  } catch {}
  return `${fallbackPrefix}: ${response.status}`;
}

export async function cancelRun(runId: string): Promise<void> {
  const res = await fetch(`${BASE_URL}/api/runs/${runId}/cancel`, {
    method: "POST",
  });
  if (!res.ok) throw new Error(`Failed to cancel run: ${res.status}`);
}

export async function updateRun(
  runId: string,
  name: string | null,
): Promise<AnalysisRun> {
  const res = await fetch(`${BASE_URL}/api/runs/${runId}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name }),
  });
  if (!res.ok) throw new Error(`Failed to update run: ${res.status}`);
  return res.json();
}

export async function getHistory(
  limit = 100,
  offset = 0,
): Promise<HistoryListItem[]> {
  const res = await fetch(
    `${BASE_URL}/api/history?limit=${limit}&offset=${offset}`,
  );
  if (!res.ok) throw new Error(`Failed to get history: ${res.status}`);
  return res.json();
}

export interface AnalysisAsset {
  id: string;
  run_id: string;
  original_url: string;
  /** 'html' | 'favicon' | 'og_image' | 'javascript' | 'stylesheet' | 'media' | 'font' */
  asset_type: string;
  content_type: string | null;
  local_path: string;
  file_size: number;
  download_error: string | null;
  created_at: string;
}

export function assetUrl(runId: string, localPath: string): string {
  return `${BASE_URL}/api/assets/${runId}/${localPath}`;
}

export async function getRunAssets(runId: string): Promise<AnalysisAsset[]> {
  const res = await fetch(`${BASE_URL}/api/runs/${runId}/assets`);
  if (!res.ok) throw new Error(`Failed to get run assets: ${res.status}`);
  return res.json();
}

const SSE_EVENT_TYPES = [
  "progress",
  "page",
  "complete",
  "failed",
  "cancelled",
] as const;

export function subscribeToEvents(
  runId: string,
  onEvent: (event: RunEvent) => void,
  onError?: (err: Event) => void,
): EventSource {
  const es = new EventSource(`${BASE_URL}/api/runs/${runId}/events`);

  for (const type of SSE_EVENT_TYPES) {
    es.addEventListener(type, (e: MessageEvent) => {
      try {
        onEvent(JSON.parse(e.data) as RunEvent);
      } catch {}
    });
  }

  es.onerror = (e) => onError?.(e);
  return es;
}
