//! SQL dialect abstraction for SQLite and PostgreSQL compatibility

/// Supported SQL dialects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SqlDialect {
    #[default]
    Postgres,
    Sqlite,
}

/// Trait for dialect-specific SQL generation
pub trait SqlDialectImpl {
    /// Get the dialect type
    fn dialect(&self) -> SqlDialect;

    /// Quote an identifier (table/column name)
    fn quote_identifier(&self, name: &str) -> String {
        format!("\"{}\"", name.replace('"', "\"\""))
    }

    /// Generate parameter placeholder for bind variable
    fn parameter_placeholder(&self, index: usize) -> String;

    /// Current timestamp function
    fn current_timestamp(&self) -> &str;

    /// Current date function
    fn current_date(&self) -> &str;

    /// Date arithmetic: add interval
    fn date_add(&self, date_expr: &str, amount: i32, unit: DateUnit) -> String;

    /// Date arithmetic: subtract interval
    fn date_sub(&self, date_expr: &str, amount: i32, unit: DateUnit) -> String;

    /// Truncate date to start of period
    fn date_trunc(&self, unit: DateUnit, date_expr: &str) -> String;

    /// Boolean literal
    fn boolean_literal(&self, value: bool) -> &str;

    /// NULLS FIRST clause
    fn nulls_first(&self) -> &str {
        "NULLS FIRST"
    }

    /// NULLS LAST clause
    fn nulls_last(&self) -> &str {
        "NULLS LAST"
    }

    /// FOR UPDATE clause (returns None if not supported)
    fn for_update(&self) -> Option<&str>;

    /// JSON array aggregation for subqueries
    fn json_array_agg(&self, inner_expr: &str) -> String;

    /// JSON object construction
    fn json_object(&self, pairs: &[(String, String)]) -> String;

    /// String concatenation
    fn concat(&self, exprs: &[String]) -> String;

    /// LIKE escape character (if needed)
    fn like_escape(&self) -> Option<&str> {
        None
    }

    /// LIMIT/OFFSET syntax
    fn limit_offset(&self, limit: Option<&str>, offset: Option<&str>) -> String {
        let mut result = String::new();
        if let Some(l) = limit {
            result.push_str(&format!("LIMIT {}", l));
        }
        if let Some(o) = offset {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&format!("OFFSET {}", o));
        }
        result
    }
}

/// Date/time units for interval arithmetic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateUnit {
    Day,
    Week,
    Month,
    Quarter,
    Year,
    Hour,
    Minute,
    Second,
}

impl DateUnit {
    pub fn as_str(&self) -> &'static str {
        match self {
            DateUnit::Day => "day",
            DateUnit::Week => "week",
            DateUnit::Month => "month",
            DateUnit::Quarter => "quarter",
            DateUnit::Year => "year",
            DateUnit::Hour => "hour",
            DateUnit::Minute => "minute",
            DateUnit::Second => "second",
        }
    }

    pub fn as_sqlite_modifier(&self) -> &'static str {
        match self {
            DateUnit::Day => "days",
            DateUnit::Week => "days", // Will multiply by 7
            DateUnit::Month => "months",
            DateUnit::Quarter => "months", // Will multiply by 3
            DateUnit::Year => "years",
            DateUnit::Hour => "hours",
            DateUnit::Minute => "minutes",
            DateUnit::Second => "seconds",
        }
    }
}

/// PostgreSQL dialect implementation
#[derive(Debug, Clone, Copy, Default)]
pub struct PostgresDialect;

impl SqlDialectImpl for PostgresDialect {
    fn dialect(&self) -> SqlDialect {
        SqlDialect::Postgres
    }

    fn parameter_placeholder(&self, index: usize) -> String {
        format!("${}", index)
    }

    fn current_timestamp(&self) -> &str {
        "CURRENT_TIMESTAMP"
    }

    fn current_date(&self) -> &str {
        "CURRENT_DATE"
    }

    fn date_add(&self, date_expr: &str, amount: i32, unit: DateUnit) -> String {
        format!(
            "({} + INTERVAL '{} {}')",
            date_expr,
            amount,
            unit.as_str()
        )
    }

    fn date_sub(&self, date_expr: &str, amount: i32, unit: DateUnit) -> String {
        format!(
            "({} - INTERVAL '{} {}')",
            date_expr,
            amount,
            unit.as_str()
        )
    }

    fn date_trunc(&self, unit: DateUnit, date_expr: &str) -> String {
        format!("date_trunc('{}', {})", unit.as_str(), date_expr)
    }

    fn boolean_literal(&self, value: bool) -> &str {
        if value {
            "TRUE"
        } else {
            "FALSE"
        }
    }

    fn for_update(&self) -> Option<&str> {
        Some("FOR UPDATE")
    }

    fn json_array_agg(&self, inner_expr: &str) -> String {
        format!("json_agg({})", inner_expr)
    }

    fn json_object(&self, pairs: &[(String, String)]) -> String {
        let args: Vec<String> = pairs
            .iter()
            .flat_map(|(k, v)| vec![format!("'{}'", k), v.clone()])
            .collect();
        format!("json_build_object({})", args.join(", "))
    }

    fn concat(&self, exprs: &[String]) -> String {
        exprs.join(" || ")
    }
}

/// SQLite dialect implementation
#[derive(Debug, Clone, Copy, Default)]
pub struct SqliteDialect;

impl SqlDialectImpl for SqliteDialect {
    fn dialect(&self) -> SqlDialect {
        SqlDialect::Sqlite
    }

    fn parameter_placeholder(&self, index: usize) -> String {
        format!("?{}", index)
    }

    fn current_timestamp(&self) -> &str {
        "datetime('now')"
    }

    fn current_date(&self) -> &str {
        "date('now')"
    }

    fn date_add(&self, date_expr: &str, amount: i32, unit: DateUnit) -> String {
        let (actual_amount, modifier) = match unit {
            DateUnit::Week => (amount * 7, "days"),
            DateUnit::Quarter => (amount * 3, "months"),
            _ => (amount, unit.as_sqlite_modifier()),
        };
        format!(
            "date({}, '+{} {}')",
            date_expr, actual_amount, modifier
        )
    }

    fn date_sub(&self, date_expr: &str, amount: i32, unit: DateUnit) -> String {
        let (actual_amount, modifier) = match unit {
            DateUnit::Week => (amount * 7, "days"),
            DateUnit::Quarter => (amount * 3, "months"),
            _ => (amount, unit.as_sqlite_modifier()),
        };
        format!(
            "date({}, '-{} {}')",
            date_expr, actual_amount, modifier
        )
    }

    fn date_trunc(&self, unit: DateUnit, date_expr: &str) -> String {
        // SQLite doesn't have date_trunc, we need to construct it
        match unit {
            DateUnit::Day => format!("date({})", date_expr),
            DateUnit::Week => {
                // Start of week (assuming Sunday)
                format!(
                    "date({}, '-' || strftime('%w', {}) || ' days')",
                    date_expr, date_expr
                )
            }
            DateUnit::Month => format!("date({}, 'start of month')", date_expr),
            DateUnit::Quarter => {
                // Quarter start: Jan 1, Apr 1, Jul 1, Oct 1
                format!(
                    "date({}, 'start of month', '-' || ((cast(strftime('%m', {}) as integer) - 1) % 3) || ' months')",
                    date_expr, date_expr
                )
            }
            DateUnit::Year => format!("date({}, 'start of year')", date_expr),
            DateUnit::Hour | DateUnit::Minute | DateUnit::Second => {
                // For time truncation, use datetime
                format!("datetime({}, 'start of day')", date_expr)
            }
        }
    }

    fn boolean_literal(&self, value: bool) -> &str {
        if value {
            "1"
        } else {
            "0"
        }
    }

    fn for_update(&self) -> Option<&str> {
        // SQLite doesn't support FOR UPDATE (uses file-level locking)
        None
    }

    fn json_array_agg(&self, inner_expr: &str) -> String {
        format!("json_group_array({})", inner_expr)
    }

    fn json_object(&self, pairs: &[(String, String)]) -> String {
        let args: Vec<String> = pairs
            .iter()
            .flat_map(|(k, v)| vec![format!("'{}'", k), v.clone()])
            .collect();
        format!("json_object({})", args.join(", "))
    }

    fn concat(&self, exprs: &[String]) -> String {
        exprs.join(" || ")
    }
}

/// Get dialect implementation for a given dialect type
pub fn get_dialect(dialect: SqlDialect) -> Box<dyn SqlDialectImpl> {
    match dialect {
        SqlDialect::Postgres => Box::new(PostgresDialect),
        SqlDialect::Sqlite => Box::new(SqliteDialect),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_placeholders() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.parameter_placeholder(1), "$1");
        assert_eq!(dialect.parameter_placeholder(10), "$10");
    }

    #[test]
    fn test_sqlite_placeholders() {
        let dialect = SqliteDialect;
        assert_eq!(dialect.parameter_placeholder(1), "?1");
        assert_eq!(dialect.parameter_placeholder(10), "?10");
    }

    #[test]
    fn test_postgres_date_arithmetic() {
        let dialect = PostgresDialect;
        assert_eq!(
            dialect.date_add("CURRENT_DATE", 30, DateUnit::Day),
            "(CURRENT_DATE + INTERVAL '30 day')"
        );
        assert_eq!(
            dialect.date_sub("CURRENT_DATE", 7, DateUnit::Day),
            "(CURRENT_DATE - INTERVAL '7 day')"
        );
    }

    #[test]
    fn test_sqlite_date_arithmetic() {
        let dialect = SqliteDialect;
        assert_eq!(
            dialect.date_add("date('now')", 30, DateUnit::Day),
            "date(date('now'), '+30 days')"
        );
        assert_eq!(
            dialect.date_sub("date('now')", 7, DateUnit::Day),
            "date(date('now'), '-7 days')"
        );
    }

    #[test]
    fn test_identifier_quoting() {
        let dialect = PostgresDialect;
        assert_eq!(dialect.quote_identifier("account"), "\"account\"");
        assert_eq!(
            dialect.quote_identifier("weird\"name"),
            "\"weird\"\"name\""
        );
    }

    #[test]
    fn test_json_aggregation() {
        let postgres = PostgresDialect;
        assert_eq!(postgres.json_array_agg("row"), "json_agg(row)");

        let sqlite = SqliteDialect;
        assert_eq!(sqlite.json_array_agg("row"), "json_group_array(row)");
    }
}
