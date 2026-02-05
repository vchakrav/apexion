//! SOQL date literal expansion to SQL expressions

use super::dialect::{DateUnit, SqlDialectImpl};
use super::error::{ConversionError, ConversionResult};

/// Expand a SOQL date literal to a SQL expression
///
/// Returns a tuple of (comparison_operator, sql_expression)
/// For range literals like LAST_N_DAYS, returns a compound expression
pub fn expand_date_literal(
    literal: &str,
    field_expr: &str,
    dialect: &dyn SqlDialectImpl,
) -> ConversionResult<String> {
    let lower = literal.to_lowercase();

    // Try to parse N-style literals first (e.g., LAST_N_DAYS:30)
    if let Some(result) = try_parse_n_literal(&lower, field_expr, dialect)? {
        return Ok(result);
    }

    // Handle simple date literals
    match lower.as_str() {
        "today" => Ok(format!(
            "DATE({}) = {}",
            field_expr,
            dialect.current_date()
        )),
        "yesterday" => Ok(format!(
            "DATE({}) = {}",
            field_expr,
            dialect.date_sub(dialect.current_date(), 1, DateUnit::Day)
        )),
        "tomorrow" => Ok(format!(
            "DATE({}) = {}",
            field_expr,
            dialect.date_add(dialect.current_date(), 1, DateUnit::Day)
        )),
        "this_week" => Ok(format!(
            "{} >= {} AND {} < {}",
            field_expr,
            dialect.date_trunc(DateUnit::Week, dialect.current_date()),
            field_expr,
            dialect.date_add(
                &dialect.date_trunc(DateUnit::Week, dialect.current_date()),
                7,
                DateUnit::Day
            )
        )),
        "last_week" => {
            let week_start = dialect.date_trunc(DateUnit::Week, dialect.current_date());
            Ok(format!(
                "{} >= {} AND {} < {}",
                field_expr,
                dialect.date_sub(&week_start, 7, DateUnit::Day),
                field_expr,
                week_start
            ))
        }
        "next_week" => {
            let week_start = dialect.date_trunc(DateUnit::Week, dialect.current_date());
            let next_week_start = dialect.date_add(&week_start, 7, DateUnit::Day);
            Ok(format!(
                "{} >= {} AND {} < {}",
                field_expr,
                next_week_start,
                field_expr,
                dialect.date_add(&next_week_start, 7, DateUnit::Day)
            ))
        }
        "this_month" => Ok(format!(
            "{} >= {} AND {} < {}",
            field_expr,
            dialect.date_trunc(DateUnit::Month, dialect.current_date()),
            field_expr,
            dialect.date_add(
                &dialect.date_trunc(DateUnit::Month, dialect.current_date()),
                1,
                DateUnit::Month
            )
        )),
        "last_month" => {
            let month_start = dialect.date_trunc(DateUnit::Month, dialect.current_date());
            Ok(format!(
                "{} >= {} AND {} < {}",
                field_expr,
                dialect.date_sub(&month_start, 1, DateUnit::Month),
                field_expr,
                month_start
            ))
        }
        "next_month" => {
            let month_start = dialect.date_trunc(DateUnit::Month, dialect.current_date());
            let next_month_start = dialect.date_add(&month_start, 1, DateUnit::Month);
            Ok(format!(
                "{} >= {} AND {} < {}",
                field_expr,
                next_month_start,
                field_expr,
                dialect.date_add(&next_month_start, 1, DateUnit::Month)
            ))
        }
        "this_quarter" => Ok(format!(
            "{} >= {} AND {} < {}",
            field_expr,
            dialect.date_trunc(DateUnit::Quarter, dialect.current_date()),
            field_expr,
            dialect.date_add(
                &dialect.date_trunc(DateUnit::Quarter, dialect.current_date()),
                3,
                DateUnit::Month
            )
        )),
        "last_quarter" => {
            let quarter_start = dialect.date_trunc(DateUnit::Quarter, dialect.current_date());
            Ok(format!(
                "{} >= {} AND {} < {}",
                field_expr,
                dialect.date_sub(&quarter_start, 3, DateUnit::Month),
                field_expr,
                quarter_start
            ))
        }
        "next_quarter" => {
            let quarter_start = dialect.date_trunc(DateUnit::Quarter, dialect.current_date());
            let next_quarter_start = dialect.date_add(&quarter_start, 3, DateUnit::Month);
            Ok(format!(
                "{} >= {} AND {} < {}",
                field_expr,
                next_quarter_start,
                field_expr,
                dialect.date_add(&next_quarter_start, 3, DateUnit::Month)
            ))
        }
        "this_year" => Ok(format!(
            "{} >= {} AND {} < {}",
            field_expr,
            dialect.date_trunc(DateUnit::Year, dialect.current_date()),
            field_expr,
            dialect.date_add(
                &dialect.date_trunc(DateUnit::Year, dialect.current_date()),
                1,
                DateUnit::Year
            )
        )),
        "last_year" => {
            let year_start = dialect.date_trunc(DateUnit::Year, dialect.current_date());
            Ok(format!(
                "{} >= {} AND {} < {}",
                field_expr,
                dialect.date_sub(&year_start, 1, DateUnit::Year),
                field_expr,
                year_start
            ))
        }
        "next_year" => {
            let year_start = dialect.date_trunc(DateUnit::Year, dialect.current_date());
            let next_year_start = dialect.date_add(&year_start, 1, DateUnit::Year);
            Ok(format!(
                "{} >= {} AND {} < {}",
                field_expr,
                next_year_start,
                field_expr,
                dialect.date_add(&next_year_start, 1, DateUnit::Year)
            ))
        }
        "this_fiscal_quarter" | "last_fiscal_quarter" | "next_fiscal_quarter" |
        "this_fiscal_year" | "last_fiscal_year" | "next_fiscal_year" => {
            // Fiscal periods depend on org configuration
            // For now, treat them as calendar periods
            let non_fiscal = lower.replace("_fiscal", "");
            expand_date_literal(&non_fiscal, field_expr, dialect)
        }
        _ => Err(ConversionError::UnknownDateLiteral(literal.to_string())),
    }
}

/// Try to parse and expand N-style date literals like LAST_N_DAYS:30
fn try_parse_n_literal(
    literal: &str,
    field_expr: &str,
    dialect: &dyn SqlDialectImpl,
) -> ConversionResult<Option<String>> {
    // Parse patterns like LAST_N_DAYS:30, NEXT_N_MONTHS:6
    let patterns = [
        ("last_n_days:", DateUnit::Day, false),
        ("next_n_days:", DateUnit::Day, true),
        ("last_n_weeks:", DateUnit::Week, false),
        ("next_n_weeks:", DateUnit::Week, true),
        ("last_n_months:", DateUnit::Month, false),
        ("next_n_months:", DateUnit::Month, true),
        ("last_n_quarters:", DateUnit::Quarter, false),
        ("next_n_quarters:", DateUnit::Quarter, true),
        ("last_n_years:", DateUnit::Year, false),
        ("next_n_years:", DateUnit::Year, true),
        // Fiscal variants
        ("last_n_fiscal_quarters:", DateUnit::Quarter, false),
        ("next_n_fiscal_quarters:", DateUnit::Quarter, true),
        ("last_n_fiscal_years:", DateUnit::Year, false),
        ("next_n_fiscal_years:", DateUnit::Year, true),
    ];

    for (prefix, unit, is_future) in patterns {
        if let Some(n_str) = literal.strip_prefix(prefix) {
            let n: i32 = n_str
                .parse()
                .map_err(|_| ConversionError::UnknownDateLiteral(literal.to_string()))?;

            if is_future {
                // NEXT_N: from now to N units in the future
                return Ok(Some(format!(
                    "{} >= {} AND {} < {}",
                    field_expr,
                    dialect.current_date(),
                    field_expr,
                    dialect.date_add(dialect.current_date(), n, unit)
                )));
            } else {
                // LAST_N: from N units ago to now
                return Ok(Some(format!(
                    "{} >= {} AND {} < {}",
                    field_expr,
                    dialect.date_sub(dialect.current_date(), n, unit),
                    field_expr,
                    dialect.current_date()
                )));
            }
        }
    }

    // N_DAYS_AGO:n pattern
    if let Some(n_str) = literal.strip_prefix("n_days_ago:") {
        let n: i32 = n_str
            .parse()
            .map_err(|_| ConversionError::UnknownDateLiteral(literal.to_string()))?;
        return Ok(Some(format!(
            "DATE({}) = {}",
            field_expr,
            dialect.date_sub(dialect.current_date(), n, DateUnit::Day)
        )));
    }

    Ok(None)
}

/// Check if a string looks like a SOQL date literal
pub fn is_date_literal(s: &str) -> bool {
    let lower = s.to_lowercase();

    // Simple literals
    let simple = [
        "today",
        "yesterday",
        "tomorrow",
        "this_week",
        "last_week",
        "next_week",
        "this_month",
        "last_month",
        "next_month",
        "this_quarter",
        "last_quarter",
        "next_quarter",
        "this_year",
        "last_year",
        "next_year",
        "this_fiscal_quarter",
        "last_fiscal_quarter",
        "next_fiscal_quarter",
        "this_fiscal_year",
        "last_fiscal_year",
        "next_fiscal_year",
    ];

    if simple.contains(&lower.as_str()) {
        return true;
    }

    // N-style literals
    let n_prefixes = [
        "last_n_days:",
        "next_n_days:",
        "last_n_weeks:",
        "next_n_weeks:",
        "last_n_months:",
        "next_n_months:",
        "last_n_quarters:",
        "next_n_quarters:",
        "last_n_years:",
        "next_n_years:",
        "last_n_fiscal_quarters:",
        "next_n_fiscal_quarters:",
        "last_n_fiscal_years:",
        "next_n_fiscal_years:",
        "n_days_ago:",
    ];

    n_prefixes.iter().any(|p| lower.starts_with(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::dialect::{PostgresDialect, SqliteDialect};

    #[test]
    fn test_is_date_literal() {
        assert!(is_date_literal("TODAY"));
        assert!(is_date_literal("today"));
        assert!(is_date_literal("LAST_N_DAYS:30"));
        assert!(is_date_literal("last_n_days:30"));
        assert!(is_date_literal("NEXT_N_MONTHS:6"));
        assert!(is_date_literal("THIS_QUARTER"));
        assert!(!is_date_literal("2024-01-01"));
        assert!(!is_date_literal("some_field"));
    }

    #[test]
    fn test_postgres_today() {
        let dialect = PostgresDialect;
        let result = expand_date_literal("TODAY", "created_date", &dialect).unwrap();
        assert!(result.contains("CURRENT_DATE"));
    }

    #[test]
    fn test_sqlite_today() {
        let dialect = SqliteDialect;
        let result = expand_date_literal("TODAY", "created_date", &dialect).unwrap();
        assert!(result.contains("date('now')"));
    }

    #[test]
    fn test_last_n_days() {
        let dialect = PostgresDialect;
        let result = expand_date_literal("LAST_N_DAYS:30", "created_date", &dialect).unwrap();
        assert!(result.contains(">="));
        assert!(result.contains("<"));
        assert!(result.contains("30"));
    }

    #[test]
    fn test_this_month() {
        let dialect = PostgresDialect;
        let result = expand_date_literal("THIS_MONTH", "created_date", &dialect).unwrap();
        assert!(result.contains("date_trunc('month'"));
    }

    #[test]
    fn test_unknown_literal() {
        let dialect = PostgresDialect;
        let result = expand_date_literal("UNKNOWN_LITERAL", "field", &dialect);
        assert!(result.is_err());
    }
}
