//! Apex to TypeScript/JavaScript transpiler
//!
//! Converts parsed Apex AST to TypeScript code that can run in:
//! - Browser Service Workers
//! - Node.js
//! - Deno
//! - Edge functions (Cloudflare Workers, etc.)
//!
//! SOQL queries are converted to async calls against an injected database context.
//! DML statements (insert/update/delete) become database mutations.

mod codegen;
pub mod context;
mod error;

pub use codegen::Transpiler;
pub use context::{RuntimeContext, RUNTIME_INTERFACE};
pub use error::TranspileError;

use crate::ast::CompilationUnit;

/// Transpile a parsed Apex compilation unit to TypeScript
pub fn transpile(unit: &CompilationUnit) -> Result<String, TranspileError> {
    let mut transpiler = Transpiler::new();
    transpiler.transpile(unit)
}

/// Transpile with custom options
pub fn transpile_with_options(
    unit: &CompilationUnit,
    options: TranspileOptions,
) -> Result<String, TranspileError> {
    let mut transpiler = Transpiler::with_options(options);
    transpiler.transpile(unit)
}

/// Options for transpilation
#[derive(Debug, Clone)]
pub struct TranspileOptions {
    /// Generate TypeScript (true) or plain JavaScript (false)
    pub typescript: bool,
    /// Include runtime imports at top of file
    pub include_imports: bool,
    /// Indent string (default: 2 spaces)
    pub indent: String,
    /// Generate async methods for SOQL/DML
    pub async_database: bool,
}

impl Default for TranspileOptions {
    fn default() -> Self {
        Self {
            typescript: true,
            include_imports: true,
            indent: "  ".to_string(),
            async_database: true,
        }
    }
}
