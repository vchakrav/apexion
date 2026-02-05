//! Error types for SOQL to SQL conversion

use thiserror::Error;

use super::dialect::SqlDialect;

/// Errors that can occur during SOQL to SQL conversion
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ConversionError {
    #[error("Unknown SObject: {0}")]
    UnknownObject(String),

    #[error("Unknown field '{field}' on object '{object}'")]
    UnknownField { object: String, field: String },

    #[error("Field '{0}' is not a relationship field")]
    NotARelationship(String),

    #[error("Field '{0}' is not polymorphic")]
    NotPolymorphic(String),

    #[error("Unknown date literal: {0}")]
    UnknownDateLiteral(String),

    #[error("Relationship depth exceeded (max: {max}, actual: {actual})")]
    RelationshipDepthExceeded { max: u8, actual: u8 },

    #[error("Feature not supported in {dialect:?}: {feature}")]
    UnsupportedFeature { dialect: SqlDialect, feature: String },

    #[error("Schema required for this query (contains: {0})")]
    SchemaRequired(String),

    #[error("Child relationship '{0}' not found on object '{1}'")]
    UnknownChildRelationship(String, String),

    #[error("Invalid SOQL expression: {0}")]
    InvalidExpression(String),

    #[error("Unsupported SOQL feature: {0}")]
    UnsupportedSoqlFeature(String),
}

/// Warnings that may occur during conversion (non-fatal)
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionWarning {
    /// FOR UPDATE is not supported in SQLite
    ForUpdateNotSupported,
    /// Salesforce-only clause was removed (e.g., FOR VIEW, FOR REFERENCE)
    SalesforceOnlyClause(String),
    /// Polymorphic field accessed without TYPEOF
    PolymorphicFieldWithoutTypeof(String),
    /// Date literal translation may be approximate
    ApproximateDateLiteral(String),
    /// WITH clause (security) was removed
    SecurityClauseRemoved(String),
}

impl std::fmt::Display for ConversionWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionWarning::ForUpdateNotSupported => {
                write!(f, "FOR UPDATE is not supported in this SQL dialect")
            }
            ConversionWarning::SalesforceOnlyClause(clause) => {
                write!(f, "Salesforce-only clause removed: {}", clause)
            }
            ConversionWarning::PolymorphicFieldWithoutTypeof(field) => {
                write!(
                    f,
                    "Polymorphic field '{}' accessed without TYPEOF",
                    field
                )
            }
            ConversionWarning::ApproximateDateLiteral(literal) => {
                write!(f, "Date literal '{}' translation may be approximate", literal)
            }
            ConversionWarning::SecurityClauseRemoved(clause) => {
                write!(f, "Security clause removed: {}", clause)
            }
        }
    }
}

/// Result type for conversion operations
pub type ConversionResult<T> = Result<T, ConversionError>;
