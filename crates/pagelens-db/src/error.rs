#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Analysis run not found: {id}")]
    AnalysisRunNotFound { id: String },

    #[error("Invalid data directory: {0}")]
    InvalidDataDir(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("Import failed: {0}")]
    ImportFailed(String),
}
