//! Transpilation errors

use std::fmt;

/// Error during transpilation
#[derive(Debug, Clone)]
pub enum TranspileError {
    /// Unsupported Apex feature
    UnsupportedFeature(String),
    /// Invalid AST structure
    InvalidAst(String),
    /// Type conversion error
    TypeError(String),
}

impl fmt::Display for TranspileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranspileError::UnsupportedFeature(msg) => {
                write!(f, "Unsupported feature: {}", msg)
            }
            TranspileError::InvalidAst(msg) => {
                write!(f, "Invalid AST: {}", msg)
            }
            TranspileError::TypeError(msg) => {
                write!(f, "Type error: {}", msg)
            }
        }
    }
}

impl std::error::Error for TranspileError {}
