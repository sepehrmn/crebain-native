//! Common error types for CREBAIN.

use std::error::Error;
use std::fmt;

/// Common errors for path operations.
#[derive(Debug)]
pub enum PathError {
    /// Path contains forbidden characters (null bytes, etc.)
    InvalidCharacters(String),
    /// Path traversal attempt detected
    TraversalAttempt(String),
    /// Path does not exist
    NotFound(String),
    /// Invalid file extension
    InvalidExtension { expected: Vec<String>, got: String },
    /// Canonicalization failed
    CanonicalizationFailed(String),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::InvalidCharacters(msg) => write!(f, "Invalid path characters: {}", msg),
            PathError::TraversalAttempt(path) => write!(f, "Path traversal detected: {}", path),
            PathError::NotFound(path) => write!(f, "Path not found: {}", path),
            PathError::InvalidExtension { expected, got } => {
                write!(f, "Invalid extension '{}', expected one of: {:?}", got, expected)
            }
            PathError::CanonicalizationFailed(msg) => write!(f, "Path resolution failed: {}", msg),
        }
    }
}

impl Error for PathError {}

/// Convenience type alias for path results
pub type PathResult<T> = Result<T, PathError>;