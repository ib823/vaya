//! Query building and execution

use crate::schema::{Record, Value};
use crate::{StoreError, StoreResult};

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Like,
    In,
    IsNull,
    IsNotNull,
}

impl CompareOp {
    /// Get the operator as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            CompareOp::Eq => "=",
            CompareOp::Ne => "!=",
            CompareOp::Lt => "<",
            CompareOp::Le => "<=",
            CompareOp::Gt => ">",
            CompareOp::Ge => ">=",
            CompareOp::Like => "LIKE",
            CompareOp::In => "IN",
            CompareOp::IsNull => "IS NULL",
            CompareOp::IsNotNull => "IS NOT NULL",
        }
    }
}

/// A filter condition
#[derive(Debug, Clone)]
pub struct Condition {
    /// Column name
    pub column: String,
    /// Operator
    pub op: CompareOp,
    /// Value(s) to compare against
    pub values: Vec<Value>,
}

impl Condition {
    /// Create an equality condition
    pub fn eq(column: impl Into<String>, value: Value) -> Self {
        Self {
            column: column.into(),
            op: CompareOp::Eq,
            values: vec![value],
        }
    }

    /// Create a not-equal condition
    pub fn ne(column: impl Into<String>, value: Value) -> Self {
        Self {
            column: column.into(),
            op: CompareOp::Ne,
            values: vec![value],
        }
    }

    /// Create a less-than condition
    pub fn lt(column: impl Into<String>, value: Value) -> Self {
        Self {
            column: column.into(),
            op: CompareOp::Lt,
            values: vec![value],
        }
    }

    /// Create a less-than-or-equal condition
    pub fn le(column: impl Into<String>, value: Value) -> Self {
        Self {
            column: column.into(),
            op: CompareOp::Le,
            values: vec![value],
        }
    }

    /// Create a greater-than condition
    pub fn gt(column: impl Into<String>, value: Value) -> Self {
        Self {
            column: column.into(),
            op: CompareOp::Gt,
            values: vec![value],
        }
    }

    /// Create a greater-than-or-equal condition
    pub fn ge(column: impl Into<String>, value: Value) -> Self {
        Self {
            column: column.into(),
            op: CompareOp::Ge,
            values: vec![value],
        }
    }

    /// Create an IN condition
    pub fn in_values(column: impl Into<String>, values: Vec<Value>) -> Self {
        Self {
            column: column.into(),
            op: CompareOp::In,
            values,
        }
    }

    /// Create an IS NULL condition
    pub fn is_null(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            op: CompareOp::IsNull,
            values: vec![],
        }
    }

    /// Create an IS NOT NULL condition
    pub fn is_not_null(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            op: CompareOp::IsNotNull,
            values: vec![],
        }
    }

    /// Evaluate this condition against a record
    pub fn matches(&self, record: &Record) -> bool {
        let field_value = record.get(&self.column);

        match self.op {
            CompareOp::IsNull => field_value.map(|v| v.is_null()).unwrap_or(true),
            CompareOp::IsNotNull => field_value.map(|v| !v.is_null()).unwrap_or(false),
            CompareOp::Eq => {
                field_value
                    .and_then(|v| self.values.first().map(|target| v == target))
                    .unwrap_or(false)
            }
            CompareOp::Ne => {
                field_value
                    .and_then(|v| self.values.first().map(|target| v != target))
                    .unwrap_or(true)
            }
            CompareOp::Lt => self.compare_numeric(field_value, |a, b| a < b),
            CompareOp::Le => self.compare_numeric(field_value, |a, b| a <= b),
            CompareOp::Gt => self.compare_numeric(field_value, |a, b| a > b),
            CompareOp::Ge => self.compare_numeric(field_value, |a, b| a >= b),
            CompareOp::In => {
                field_value
                    .map(|v| self.values.contains(v))
                    .unwrap_or(false)
            }
            CompareOp::Like => self.matches_like(field_value),
        }
    }

    fn compare_numeric<F>(&self, field: Option<&Value>, cmp: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        let Some(field) = field else { return false };
        let Some(target) = self.values.first() else { return false };

        match (field, target) {
            (Value::Int64(a), Value::Int64(b)) => cmp(*a as f64, *b as f64),
            (Value::Float64(a), Value::Float64(b)) => cmp(*a, *b),
            (Value::Float32(a), Value::Float32(b)) => cmp(*a as f64, *b as f64),
            (Value::Int64(a), Value::Float64(b)) => cmp(*a as f64, *b),
            (Value::Float64(a), Value::Int64(b)) => cmp(*a, *b as f64),
            (Value::String(a), Value::String(b)) => cmp(a.len() as f64, b.len() as f64), // String comparison by length for numeric ops
            _ => false,
        }
    }

    fn matches_like(&self, field: Option<&Value>) -> bool {
        let Some(Value::String(s)) = field else { return false };
        let Some(Value::String(pattern)) = self.values.first() else { return false };

        // Simple LIKE implementation: % matches any sequence, _ matches single char
        self.like_match(s, pattern)
    }

    fn like_match(&self, s: &str, pattern: &str) -> bool {
        let mut s_chars = s.chars().peekable();
        let mut p_chars = pattern.chars().peekable();

        while let Some(p) = p_chars.next() {
            match p {
                '%' => {
                    // Match any sequence
                    if p_chars.peek().is_none() {
                        return true; // % at end matches everything
                    }
                    // Try matching the rest of the pattern at each position
                    let remaining: String = p_chars.collect();
                    while s_chars.peek().is_some() {
                        let remaining_s: String = s_chars.clone().collect();
                        if self.like_match(&remaining_s, &remaining) {
                            return true;
                        }
                        s_chars.next();
                    }
                    return self.like_match("", &remaining);
                }
                '_' => {
                    // Match exactly one character
                    if s_chars.next().is_none() {
                        return false;
                    }
                }
                c => {
                    // Match exact character
                    match s_chars.next() {
                        Some(sc) if sc == c => {}
                        _ => return false,
                    }
                }
            }
        }

        s_chars.peek().is_none()
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// A sort specification
#[derive(Debug, Clone)]
pub struct Sort {
    /// Column to sort by
    pub column: String,
    /// Sort direction
    pub order: SortOrder,
}

impl Sort {
    /// Create ascending sort
    pub fn asc(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            order: SortOrder::Asc,
        }
    }

    /// Create descending sort
    pub fn desc(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            order: SortOrder::Desc,
        }
    }
}

/// A query against a table
#[derive(Debug, Clone)]
pub struct Query {
    /// Table name
    pub table: String,
    /// Conditions (ANDed together)
    pub conditions: Vec<Condition>,
    /// Sort specifications
    pub sorts: Vec<Sort>,
    /// Maximum number of records to return
    pub limit: Option<usize>,
    /// Number of records to skip
    pub offset: Option<usize>,
    /// Columns to select (empty = all)
    pub select_columns: Vec<String>,
}

impl Query {
    /// Create a new query for a table
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            conditions: Vec::new(),
            sorts: Vec::new(),
            limit: None,
            offset: None,
            select_columns: Vec::new(),
        }
    }

    /// Add a filter condition
    pub fn filter(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Add an equality filter
    pub fn eq(self, column: impl Into<String>, value: Value) -> Self {
        self.filter(Condition::eq(column, value))
    }

    /// Add a sort specification
    pub fn order_by(mut self, sort: Sort) -> Self {
        self.sorts.push(sort);
        self
    }

    /// Add ascending sort
    pub fn order_asc(self, column: impl Into<String>) -> Self {
        self.order_by(Sort::asc(column))
    }

    /// Add descending sort
    pub fn order_desc(self, column: impl Into<String>) -> Self {
        self.order_by(Sort::desc(column))
    }

    /// Set the limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Select specific columns
    pub fn select(mut self, columns: Vec<String>) -> Self {
        self.select_columns = columns;
        self
    }

    /// Check if a record matches all conditions
    pub fn matches(&self, record: &Record) -> bool {
        self.conditions.iter().all(|c| c.matches(record))
    }
}

/// Query builder for fluent API
pub struct QueryBuilder {
    query: Query,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn from(table: impl Into<String>) -> Self {
        Self {
            query: Query::new(table),
        }
    }

    /// Add an equality filter
    pub fn where_eq(mut self, column: impl Into<String>, value: Value) -> Self {
        self.query = self.query.eq(column, value);
        self
    }

    /// Add a filter condition
    pub fn where_cond(mut self, condition: Condition) -> Self {
        self.query = self.query.filter(condition);
        self
    }

    /// Add ascending order
    pub fn order_by_asc(mut self, column: impl Into<String>) -> Self {
        self.query = self.query.order_asc(column);
        self
    }

    /// Add descending order
    pub fn order_by_desc(mut self, column: impl Into<String>) -> Self {
        self.query = self.query.order_desc(column);
        self
    }

    /// Set limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.query = self.query.limit(limit);
        self
    }

    /// Set offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.query = self.query.offset(offset);
        self
    }

    /// Build the query
    pub fn build(self) -> Query {
        self.query
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::RecordBuilder;

    #[test]
    fn test_condition_eq() {
        let record = RecordBuilder::new()
            .int64("id", 1)
            .string("name", "Alice")
            .build();

        assert!(Condition::eq("id", Value::Int64(1)).matches(&record));
        assert!(!Condition::eq("id", Value::Int64(2)).matches(&record));
        assert!(Condition::eq("name", Value::String("Alice".into())).matches(&record));
    }

    #[test]
    fn test_condition_comparison() {
        let record = RecordBuilder::new().int64("age", 25).build();

        assert!(Condition::lt("age", Value::Int64(30)).matches(&record));
        assert!(Condition::le("age", Value::Int64(25)).matches(&record));
        assert!(Condition::gt("age", Value::Int64(20)).matches(&record));
        assert!(Condition::ge("age", Value::Int64(25)).matches(&record));
        assert!(!Condition::gt("age", Value::Int64(25)).matches(&record));
    }

    #[test]
    fn test_condition_like() {
        let record = RecordBuilder::new()
            .string("name", "Alice Smith")
            .build();

        assert!(Condition {
            column: "name".into(),
            op: CompareOp::Like,
            values: vec![Value::String("Alice%".into())]
        }
        .matches(&record));

        assert!(Condition {
            column: "name".into(),
            op: CompareOp::Like,
            values: vec![Value::String("%Smith".into())]
        }
        .matches(&record));

        assert!(Condition {
            column: "name".into(),
            op: CompareOp::Like,
            values: vec![Value::String("Alice_Smith".into())]
        }
        .matches(&record));
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::from("users")
            .where_eq("active", Value::Bool(true))
            .where_cond(Condition::gt("age", Value::Int64(18)))
            .order_by_desc("created_at")
            .limit(10)
            .build();

        assert_eq!(query.table, "users");
        assert_eq!(query.conditions.len(), 2);
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_query_matches() {
        let query = QueryBuilder::from("users")
            .where_eq("active", Value::Bool(true))
            .where_cond(Condition::gt("age", Value::Int64(18)))
            .build();

        let record1 = RecordBuilder::new()
            .bool("active", true)
            .int64("age", 25)
            .build();

        let record2 = RecordBuilder::new()
            .bool("active", false)
            .int64("age", 25)
            .build();

        let record3 = RecordBuilder::new()
            .bool("active", true)
            .int64("age", 15)
            .build();

        assert!(query.matches(&record1));
        assert!(!query.matches(&record2));
        assert!(!query.matches(&record3));
    }
}
