use crate::models::{
    AnalysisRun, AnalysisType, CreateAnalysisRun, HistoryListItem, UpdateAnalysisRun,
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
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = Utc::now();
        let analysis_type_str = match input.analysis_type {
            AnalysisType::Single => "single",
            AnalysisType::Crawl => "crawl",
        };

        self.conn.execute(
            "INSERT INTO analysis_runs (
                id, url, created_at, name, analysis_type, payload,
                seo_score, page_count, total_issues, error_count, warning_count, duration_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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
        })
    }

    /// Get a single analysis run by ID.
    pub fn get(&self, id: &str) -> Result<AnalysisRun> {
        let mut stmt = self.conn.prepare(
            "SELECT 
                id, url, created_at, name, analysis_type, payload,
                seo_score, page_count, total_issues, error_count, warning_count, duration_ms
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
                seo_score, page_count, total_issues, error_count, warning_count, duration_ms
            FROM analysis_runs 
            ORDER BY created_at DESC
            LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map([limit, offset], |row| {
            let analysis_type_str: String = row.get(4)?;
            let analysis_type = match analysis_type_str.as_str() {
                "crawl" => AnalysisType::Crawl,
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
                seo_score, page_count, total_issues, error_count, warning_count, duration_ms
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
        })
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
