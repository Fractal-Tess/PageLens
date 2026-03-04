use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Db(#[from] pagelens_db::error::Error),
    #[error(transparent)]
    Core(#[from] pagelens_core::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Message(String),
}

impl Error {
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::Db(pagelens_db::error::Error::AnalysisRunNotFound { .. })
        )
    }
}
