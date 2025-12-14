//! # Mamba Error Handling
//!
//! This crate provides error types, formatting, and reporting for the Mamba compiler.

use colored::Colorize;
use thiserror::Error;

/// Main error type for Mamba compilation
#[derive(Debug, Error)]
pub enum MambaError {
    #[error("Syntax error: {0}")]
    SyntaxError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Semantic error: {0}")]
    SemanticError(String),

    #[error("Transpilation error: {0}")]
    TranspileError(String),

    #[error("Compilation error: {0}")]
    CompileError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Format an error with color and position information
pub fn format_error(error: &MambaError, file: &str, line: usize, column: usize) -> String {
    format!(
        "{} {}:{}:{}\n  {}",
        "Error:".red().bold(),
        file,
        line,
        column,
        error
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = MambaError::SyntaxError("unexpected token".to_string());
        assert!(error.to_string().contains("Syntax error"));
    }
}
