CREATE TABLE IF NOT EXISTS analysis_assets (
    id           TEXT PRIMARY KEY,
    run_id       TEXT NOT NULL REFERENCES analysis_runs(id) ON DELETE CASCADE,
    original_url TEXT NOT NULL,
    asset_type   TEXT NOT NULL,
    content_type TEXT,
    local_path   TEXT NOT NULL,
    file_size    INTEGER NOT NULL DEFAULT 0,
    download_error TEXT,
    created_at   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_analysis_assets_run_id ON analysis_assets(run_id);
