-- Migration 001: Initial schema for analysis history
-- Creates the analysis_runs table to store complete analysis results

CREATE TABLE IF NOT EXISTS analysis_runs (
    id TEXT PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    created_at TEXT NOT NULL, -- ISO 8601 format
    name TEXT,
    analysis_type TEXT NOT NULL CHECK(analysis_type IN ('single', 'crawl')),
    payload TEXT NOT NULL, -- JSON payload
    seo_score REAL,
    page_count INTEGER NOT NULL DEFAULT 1,
    total_issues INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    warning_count INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER NOT NULL DEFAULT 0
);

-- Index for efficient listing by creation date (newest first)
CREATE INDEX IF NOT EXISTS idx_analysis_runs_created_at 
ON analysis_runs(created_at DESC);

-- Index for URL-based lookups
CREATE INDEX IF NOT EXISTS idx_analysis_runs_url 
ON analysis_runs(url);
