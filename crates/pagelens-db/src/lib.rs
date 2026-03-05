pub mod error;
pub mod models;
pub mod prelude;
pub mod repository;

use crate::prelude::*;
use pagelens_logging::{debug, info};
use rusqlite::{params, Connection, OpenFlags};
use std::path::Path;

pub use models::*;
pub use repository::AnalysisRepository;

/// The database manager handles connection pooling, migrations, and configuration.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create the database at the specified path.
    ///
    /// This will:
    /// 1. Create parent directories if they don't exist
    /// 2. Open the database with WAL mode enabled
    /// 3. Run any pending migrations
    /// 4. Configure busy timeout for concurrent access
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        debug!(db_path = %path.display(), "Opening database");

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open database with appropriate flags
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;

        let db = Self { conn };
        db.configure()?;
        db.run_migrations()?;
        debug!(db_path = %path.display(), "Database ready");

        Ok(db)
    }

    /// Configure database settings for optimal performance and reliability.
    fn configure(&self) -> Result<()> {
        // Enable WAL mode for better concurrency
        self.conn.execute_batch("PRAGMA journal_mode = WAL;")?;

        // Set busy timeout to 5 seconds to handle concurrent access gracefully
        self.conn.execute_batch("PRAGMA busy_timeout = 5000;")?;

        // Enable foreign key support
        self.conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Synchronous mode NORMAL for good balance of safety and performance
        self.conn.execute_batch("PRAGMA synchronous = NORMAL;")?;

        Ok(())
    }

    /// Run database migrations.
    fn run_migrations(&self) -> Result<()> {
        // Create migrations tracking table if it doesn't exist
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            )",
            [],
        )?;

        // Get currently applied migrations
        let mut stmt = self
            .conn
            .prepare("SELECT version FROM _migrations ORDER BY version")?;
        let applied_versions: std::collections::HashSet<i64> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        // Define migrations
        let migrations: Vec<(i64, &str)> = vec![
            (1, include_str!("../migrations/001_initial_schema.sql")),
            (2, include_str!("../migrations/002_live_runs_and_pages.sql")),
            (3, include_str!("../migrations/003_assets.sql")),
            (
                4,
                include_str!("../migrations/004_analysis_type_http_benchmark.sql"),
            ),
            (
                5,
                include_str!("../migrations/005_analysis_type_favicon.sql"),
            ),
        ];

        // Apply pending migrations in a transaction
        let tx = self.conn.unchecked_transaction()?;

        for (version, sql) in migrations {
            if !applied_versions.contains(&version) {
                info!(migration_version = version, "Applying database migration");
                tx.execute_batch(sql).map_err(|e| {
                    Error::Migration(format!("Migration {} failed: {}", version, e))
                })?;

                tx.execute(
                    "INSERT INTO _migrations (version, applied_at) VALUES (?1, ?2)",
                    params![version, chrono::Utc::now().to_rfc3339()],
                )?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Get a reference to the underlying connection.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Get a repository for analysis run operations.
    pub fn analysis_repository(&self) -> AnalysisRepository<'_> {
        AnalysisRepository::new(&self.conn)
    }

    /// Close the database connection.
    pub fn close(self) -> Result<()> {
        self.conn.close().map_err(|(_, e)| e.into())
    }
}

/// Export the entire database to a JSON file.
pub fn export_to_file(db: &Database, path: &Path) -> Result<()> {
    use std::io::Write;

    let runs = db.analysis_repository().get_all_for_export()?;
    let export = HistoryExport::new(runs);
    let json = serde_json::to_string_pretty(&export)?;

    let mut file = std::fs::File::create(path)?;
    file.write_all(json.as_bytes())?;

    Ok(())
}

/// Import analysis runs from a JSON file.
pub fn import_from_file(db: &Database, path: &Path) -> Result<usize> {
    let json = std::fs::read_to_string(path)?;
    let export: HistoryExport = serde_json::from_str(&json)
        .map_err(|e| Error::ImportFailed(format!("Invalid export format: {}", e)))?;

    let repo = db.analysis_repository();
    let mut imported = 0;

    for run in export.runs {
        // Create a new run with the same data but new ID
        let input = CreateAnalysisRun {
            id: None,
            url: run.url,
            name: run.name,
            analysis_type: run.analysis_type,
            payload_json: run.payload_json,
            summary: run.summary,
            status: run.status,
            current_stage: run.current_stage,
            current_message: run.current_message,
            progress: run.progress,
        };

        repo.create(input)?;
        imported += 1;
    }

    Ok(imported)
}
