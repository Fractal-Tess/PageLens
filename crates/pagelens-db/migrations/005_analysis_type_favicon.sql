CREATE TABLE IF NOT EXISTS analysis_runs_new (
    id TEXT PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    created_at TEXT NOT NULL,
    name TEXT,
    analysis_type TEXT NOT NULL CHECK(analysis_type IN ('single', 'crawl', 'http_benchmark', 'favicon')),
    payload TEXT NOT NULL,
    seo_score REAL,
    page_count INTEGER NOT NULL DEFAULT 1,
    total_issues INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    warning_count INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'completed' CHECK(status IN ('pending', 'running', 'completed', 'failed')),
    current_stage TEXT,
    current_message TEXT,
    progress REAL
);

INSERT INTO analysis_runs_new (
    id, url, created_at, name, analysis_type, payload,
    seo_score, page_count, total_issues, error_count, warning_count, duration_ms,
    status, current_stage, current_message, progress
)
SELECT
    id, url, created_at, name, analysis_type, payload,
    seo_score, page_count, total_issues, error_count, warning_count, duration_ms,
    status, current_stage, current_message, progress
FROM analysis_runs;

DROP TABLE analysis_runs;
ALTER TABLE analysis_runs_new RENAME TO analysis_runs;

CREATE INDEX IF NOT EXISTS idx_analysis_runs_created_at ON analysis_runs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_analysis_runs_url ON analysis_runs(url);
