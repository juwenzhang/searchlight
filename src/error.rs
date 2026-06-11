use thiserror::Error;

/// Unified error type for searchlight operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SearchlightError {
    #[error("query exceeds maximum length of {max} characters (got {actual})")]
    QueryTooLong { max: usize, actual: usize },

    #[error("query contains too many terms (max {max})")]
    TooManyTerms { max: usize },

    #[error("document {doc_id} not found in index")]
    DocumentNotFound { doc_id: usize },

    #[error("index is empty")]
    IndexEmpty,

    #[error("invalid search options: {0}")]
    InvalidOptions(String),
}

pub type Result<T> = std::result::Result<T, SearchlightError>;
