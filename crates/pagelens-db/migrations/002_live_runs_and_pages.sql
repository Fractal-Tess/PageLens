ALTER TABLE analysis_runs ADD COLUMN status TEXT NOT NULL DEFAULT 'completed' CHECK(status IN ('pending', 'running', 'completed', 'failed'));
ALTER TABLE analysis_runs ADD COLUMN current_stage TEXT;
ALTER TABLE analysis_runs ADD COLUMN current_message TEXT;
ALTER TABLE analysis_runs ADD COLUMN progress REAL;

CREATE TABLE IF NOT EXISTS analysis_page_results (
    id TEXT PRIMARY KEY NOT NULL,
    run_id TEXT NOT NULL,
    url TEXT NOT NULL,
    depth INTEGER NOT NULL DEFAULT 0,
    success INTEGER NOT NULL DEFAULT 1,
    seo_score REAL,
    total_issues INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    warning_count INTEGER NOT NULL DEFAULT 0,
    links_found_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    analyzed_at TEXT NOT NULL,
    FOREIGN KEY(run_id) REFERENCES analysis_runs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_analysis_page_results_run_id
ON analysis_page_results(run_id);

CREATE INDEX IF NOT EXISTS idx_analysis_page_results_analyzed_at
ON analysis_page_results(analyzed_at);
