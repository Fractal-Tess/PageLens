use crate::models::{
    AnalysisAsset, AnalysisPageResult, AnalysisRun, AnalysisRunStatus, AnalysisSummary,
    AnalysisType, CreateAnalysisAsset, CreateAnalysisPageResult, CreateAnalysisRun,
    HistoryListItem, UpdateAnalysisRun,
};
use crate::prelude::*;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};

/// Repository for analysis run operations.
pub struct AnalysisRepository<'a> {
    conn: &'a Connection,
}

impl<'a> AnalysisRepository<'a> {
    /// Create a new repository instance.
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Create a new analysis run.
    pub fn create(&self, input: CreateAnalysisRun) -> Result<AnalysisRun> {
        let id = input
            .id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let created_at = Utc::now();
        let analysis_type_str = match input.analysis_type {
            AnalysisType::Single => "single",
            AnalysisType::Crawl => "crawl",
            AnalysisType::HttpBenchmark => "http_benchmark",
        };
        let status_str = status_to_str(input.status);

        self.conn.execute(
            "INSERT INTO analysis_runs (
                id, url, created_at, name, analysis_type, payload,
                seo_score, page_count, total_issues, error_count, warning_count, duration_ms,
                status, current_stage, current_message, progress
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                &id,
                &input.url,
                created_at.to_rfc3339(),
                input.name,
                analysis_type_str,
                &input.payload_json,
                input.summary.seo_score,
                input.summary.page_count as i64,
                input.summary.total_issues as i64,
                input.summary.error_count as i64,
                input.summary.warning_count as i64,
                input.summary.duration_ms as i64,
                status_str,
                input.current_stage,
                input.current_message,
                input.progress,
            ],
        )?;

        Ok(AnalysisRun {
            id,
            url: input.url,
            created_at,
            name: input.name,
            analysis_type: input.analysis_type,
            payload_json: input.payload_json,
            summary: input.summary,
            status: input.status,
            current_stage: input.current_stage,
            current_message: input.current_message,
            progress: input.progress,
        })
    }

    /// Get a single analysis run by ID.
    pub fn get(&self, id: &str) -> Result<AnalysisRun> {
        let mut stmt = self.conn.prepare(
            "SELECT 
                id, url, created_at, name, analysis_type, payload,
                seo_score, page_count, total_issues, error_count, warning_count, duration_ms,
                status, current_stage, current_message, progress
            FROM analysis_runs 
            WHERE id = ?1",
        )?;

        stmt.query_row([id], |row| self.map_row_to_analysis_run(row))
            .map_err(|_| Error::AnalysisRunNotFound { id: id.to_string() })
    }

    /// List all analysis runs (newest first) with pagination.
    pub fn list(&self, limit: i64, offset: i64) -> Result<Vec<HistoryListItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT 
                id, url, created_at, name, analysis_type,
                seo_score, page_count, total_issues, error_count, warning_count, duration_ms,
                status
            FROM analysis_runs 
            ORDER BY created_at DESC
            LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map([limit, offset], |row| {
            let analysis_type_str: String = row.get(4)?;
            let analysis_type = match analysis_type_str.as_str() {
                "crawl" => AnalysisType::Crawl,
                "http_benchmark" => AnalysisType::HttpBenchmark,
                _ => AnalysisType::Single,
            };

            Ok(HistoryListItem {
                id: row.get(0)?,
                url: row.get(1)?,
                created_at: parse_datetime(row.get(2)?)?,
                name: row.get(3)?,
                analysis_type,
                summary: crate::models::AnalysisSummary {
                    seo_score: row.get(5)?,
                    page_count: row.get::<_, i64>(6)? as u32,
                    total_issues: row.get::<_, i64>(7)? as u32,
                    error_count: row.get::<_, i64>(8)? as u32,
                    warning_count: row.get::<_, i64>(9)? as u32,
                    duration_ms: row.get::<_, i64>(10)? as u32,
                },
                status: status_from_str(row.get(11)?),
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }

        Ok(items)
    }

    /// Get total count of analysis runs.
    pub fn count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM analysis_runs", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Update an analysis run.
    pub fn update(&self, id: &str, input: UpdateAnalysisRun) -> Result<AnalysisRun> {
        self.conn.execute(
            "UPDATE analysis_runs SET name = ?1 WHERE id = ?2",
            params![input.name, id],
        )?;

        self.get(id)
    }

    pub fn update_run_state(
        &self,
        id: &str,
        status: AnalysisRunStatus,
        current_stage: Option<&str>,
        current_message: Option<&str>,
        progress: Option<f64>,
    ) -> Result<()> {
        let updated = self.conn.execute(
            "UPDATE analysis_runs
             SET status = ?1, current_stage = ?2, current_message = ?3, progress = ?4
             WHERE id = ?5",
            params![
                status_to_str(status),
                current_stage,
                current_message,
                progress,
                id,
            ],
        )?;

        if updated == 0 {
            return Err(Error::AnalysisRunNotFound { id: id.to_string() });
        }

        Ok(())
    }

    pub fn update_run_state_if_active(
        &self,
        id: &str,
        status: AnalysisRunStatus,
        current_stage: Option<&str>,
        current_message: Option<&str>,
        progress: Option<f64>,
    ) -> Result<bool> {
        let updated = self.conn.execute(
            "UPDATE analysis_runs
             SET status = ?1, current_stage = ?2, current_message = ?3, progress = ?4
             WHERE id = ?5 AND status IN ('pending', 'running')",
            params![
                status_to_str(status),
                current_stage,
                current_message,
                progress,
                id,
            ],
        )?;

        if updated > 0 {
            return Ok(true);
        }

        if !self.run_exists(id)? {
            return Err(Error::AnalysisRunNotFound { id: id.to_string() });
        }

        Ok(false)
    }

    pub fn complete_run(
        &self,
        id: &str,
        payload_json: &str,
        summary: &AnalysisSummary,
    ) -> Result<()> {
        let updated = self.conn.execute(
            "UPDATE analysis_runs
             SET payload = ?1,
                 seo_score = ?2,
                 page_count = ?3,
                 total_issues = ?4,
                 error_count = ?5,
                 warning_count = ?6,
                 duration_ms = ?7,
                 status = 'completed',
                 current_stage = 'complete',
                 current_message = 'Analysis complete!',
                 progress = 1.0
             WHERE id = ?8",
            params![
                payload_json,
                summary.seo_score,
                summary.page_count as i64,
                summary.total_issues as i64,
                summary.error_count as i64,
                summary.warning_count as i64,
                summary.duration_ms as i64,
                id,
            ],
        )?;

        if updated == 0 {
            return Err(Error::AnalysisRunNotFound { id: id.to_string() });
        }

        Ok(())
    }

    pub fn complete_run_if_active(
        &self,
        id: &str,
        payload_json: &str,
        summary: &AnalysisSummary,
    ) -> Result<bool> {
        let updated = self.conn.execute(
            "UPDATE analysis_runs
             SET payload = ?1,
                 seo_score = ?2,
                 page_count = ?3,
                 total_issues = ?4,
                 error_count = ?5,
                 warning_count = ?6,
                 duration_ms = ?7,
                 status = 'completed',
                 current_stage = 'complete',
                 current_message = 'Analysis complete!',
                 progress = 1.0
             WHERE id = ?8 AND status IN ('pending', 'running')",
            params![
                payload_json,
                summary.seo_score,
                summary.page_count as i64,
                summary.total_issues as i64,
                summary.error_count as i64,
                summary.warning_count as i64,
                summary.duration_ms as i64,
                id,
            ],
        )?;

        if updated > 0 {
            return Ok(true);
        }

        if !self.run_exists(id)? {
            return Err(Error::AnalysisRunNotFound { id: id.to_string() });
        }

        Ok(false)
    }

    pub fn complete_run_as_cancelled_if_active(
        &self,
        id: &str,
        payload_json: &str,
        summary: &AnalysisSummary,
        message: &str,
    ) -> Result<bool> {
        let updated = self.conn.execute(
            "UPDATE analysis_runs
             SET payload = ?1,
                 seo_score = ?2,
                 page_count = ?3,
                 total_issues = ?4,
                 error_count = ?5,
                 warning_count = ?6,
                 duration_ms = ?7,
                 status = 'failed',
                 current_stage = 'cancelled',
                 current_message = ?8,
                 progress = 1.0
             WHERE id = ?9 AND status IN ('pending', 'running')",
            params![
                payload_json,
                summary.seo_score,
                summary.page_count as i64,
                summary.total_issues as i64,
                summary.error_count as i64,
                summary.warning_count as i64,
                summary.duration_ms as i64,
                message,
                id,
            ],
        )?;

        if updated > 0 {
            return Ok(true);
        }

        if !self.run_exists(id)? {
            return Err(Error::AnalysisRunNotFound { id: id.to_string() });
        }

        Ok(false)
    }

    pub fn insert_page_result(
        &self,
        input: CreateAnalysisPageResult,
    ) -> Result<AnalysisPageResult> {
        let id = uuid::Uuid::new_v4().to_string();
        let analyzed_at = Utc::now();

        self.conn.execute(
            "INSERT INTO analysis_page_results (
                id, run_id, url, depth, success, seo_score, total_issues,
                error_count, warning_count, links_found_count, error_message, analyzed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                &id,
                &input.run_id,
                &input.url,
                input.depth as i64,
                input.success,
                input.seo_score,
                input.total_issues as i64,
                input.error_count as i64,
                input.warning_count as i64,
                input.links_found_count as i64,
                input.error_message,
                analyzed_at.to_rfc3339(),
            ],
        )?;

        Ok(AnalysisPageResult {
            id,
            run_id: input.run_id,
            url: input.url,
            depth: input.depth,
            success: input.success,
            seo_score: input.seo_score,
            total_issues: input.total_issues,
            error_count: input.error_count,
            warning_count: input.warning_count,
            links_found_count: input.links_found_count,
            error_message: input.error_message,
            analyzed_at,
        })
    }

    pub fn list_page_results(&self, run_id: &str) -> Result<Vec<AnalysisPageResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                id, run_id, url, depth, success, seo_score, total_issues,
                error_count, warning_count, links_found_count, error_message, analyzed_at
             FROM analysis_page_results
             WHERE run_id = ?1
             ORDER BY analyzed_at ASC",
        )?;

        let rows = stmt.query_map([run_id], |row| {
            Ok(AnalysisPageResult {
                id: row.get(0)?,
                run_id: row.get(1)?,
                url: row.get(2)?,
                depth: row.get::<_, i64>(3)? as u32,
                success: row.get(4)?,
                seo_score: row.get(5)?,
                total_issues: row.get::<_, i64>(6)? as u32,
                error_count: row.get::<_, i64>(7)? as u32,
                warning_count: row.get::<_, i64>(8)? as u32,
                links_found_count: row.get::<_, i64>(9)? as u32,
                error_message: row.get(10)?,
                analyzed_at: parse_datetime(row.get(11)?)?,
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// Insert a downloaded asset record.
    pub fn insert_asset(&self, input: CreateAnalysisAsset) -> Result<AnalysisAsset> {
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = Utc::now();

        self.conn.execute(
            "INSERT INTO analysis_assets (
                id, run_id, original_url, asset_type, content_type,
                local_path, file_size, download_error, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &id,
                &input.run_id,
                &input.original_url,
                &input.asset_type,
                input.content_type.as_deref(),
                &input.local_path,
                input.file_size,
                input.download_error.as_deref(),
                created_at.to_rfc3339(),
            ],
        )?;

        Ok(AnalysisAsset {
            id,
            run_id: input.run_id,
            original_url: input.original_url,
            asset_type: input.asset_type,
            content_type: input.content_type,
            local_path: input.local_path,
            file_size: input.file_size,
            download_error: input.download_error,
            created_at,
        })
    }

    /// List all cached assets for a run.
    pub fn list_assets(&self, run_id: &str) -> Result<Vec<AnalysisAsset>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, run_id, original_url, asset_type, content_type,
                    local_path, file_size, download_error, created_at
             FROM analysis_assets
             WHERE run_id = ?1
             ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map([run_id], |row| {
            Ok(AnalysisAsset {
                id: row.get(0)?,
                run_id: row.get(1)?,
                original_url: row.get(2)?,
                asset_type: row.get(3)?,
                content_type: row.get(4)?,
                local_path: row.get(5)?,
                file_size: row.get(6)?,
                download_error: row.get(7)?,
                created_at: parse_datetime(row.get(8)?)?,
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// Delete a single analysis run by ID.
    pub fn delete(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM analysis_runs WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Delete all analysis runs.
    pub fn delete_all(&self) -> Result<usize> {
        let count = self.conn.execute("DELETE FROM analysis_runs", [])?;
        Ok(count)
    }

    /// Get all analysis runs for export.
    pub fn get_all_for_export(&self) -> Result<Vec<AnalysisRun>> {
        let mut stmt = self.conn.prepare(
            "SELECT 
                id, url, created_at, name, analysis_type, payload,
                seo_score, page_count, total_issues, error_count, warning_count, duration_ms,
                status, current_stage, current_message, progress
            FROM analysis_runs 
            ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| self.map_row_to_analysis_run(row))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }

        Ok(items)
    }

    /// Helper to map a database row to AnalysisRun.
    fn map_row_to_analysis_run(
        &self,
        row: &Row,
    ) -> std::result::Result<AnalysisRun, rusqlite::Error> {
        let analysis_type_str: String = row.get(4)?;
        let analysis_type = match analysis_type_str.as_str() {
            "crawl" => AnalysisType::Crawl,
            "http_benchmark" => AnalysisType::HttpBenchmark,
            _ => AnalysisType::Single,
        };

        let payload_json: String = row.get(5)?;

        Ok(AnalysisRun {
            id: row.get(0)?,
            url: row.get(1)?,
            created_at: parse_datetime(row.get(2)?)?,
            name: row.get(3)?,
            analysis_type,
            payload_json,
            summary: crate::models::AnalysisSummary {
                seo_score: row.get(6)?,
                page_count: row.get::<_, i64>(7)? as u32,
                total_issues: row.get::<_, i64>(8)? as u32,
                error_count: row.get::<_, i64>(9)? as u32,
                warning_count: row.get::<_, i64>(10)? as u32,
                duration_ms: row.get::<_, i64>(11)? as u32,
            },
            status: status_from_str(row.get(12)?),
            current_stage: row.get(13)?,
            current_message: row.get(14)?,
            progress: row.get(15)?,
        })
    }

    fn run_exists(&self, id: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM analysis_runs WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}

fn status_to_str(status: AnalysisRunStatus) -> &'static str {
    match status {
        AnalysisRunStatus::Pending => "pending",
        AnalysisRunStatus::Running => "running",
        AnalysisRunStatus::Completed => "completed",
        AnalysisRunStatus::Failed => "failed",
    }
}

fn status_from_str(status: String) -> AnalysisRunStatus {
    match status.as_str() {
        "pending" => AnalysisRunStatus::Pending,
        "running" => AnalysisRunStatus::Running,
        "failed" => AnalysisRunStatus::Failed,
        _ => AnalysisRunStatus::Completed,
    }
}

/// Helper function to parse datetime from string.
fn parse_datetime(s: String) -> std::result::Result<DateTime<Utc>, rusqlite::Error> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
}
