use std::fmt::{Display, Write};

/// Struct representing a where clause in a query
#[derive(Debug, PartialEq, Clone)]
pub struct WhereClause {
    pub field: &'static str,
    pub operator: Operator,
    pub values: Vec<String>,
}

impl WhereClause {
    /// Create a new where clause
    #[must_use]
    pub fn new(field: &'static str, operator: Operator) -> Self {
        Self {
            field,
            operator,
            values: Vec::new(),
        }
    }

    /// Add a value to the where clause
    #[must_use]
    pub fn add_value<T: Display>(mut self, value: T) -> Self {
        self.values.push(value.to_string());
        self
    }

    /// Add multiple values to the where clause from an iterator
    #[must_use]
    pub fn add_values<I, T>(mut self, values: I) -> Self
    where
        I: Iterator<Item = T>,
        T: Display,
    {
        self.values.extend(values.map(|v| v.to_string()));
        self
    }
}

impl WhereClause {
    pub fn extend_query(&self, query: &mut String) {
        let op_str = match self.operator {
            Operator::In => "IN",
            Operator::Like => "LIKE",
            Operator::Equal => "=",
            Operator::Less => "<",
            Operator::Greater => ">",
            Operator::LessEqual => "<=",
            Operator::GreaterEqual => ">=",
        };

        if self.operator == Operator::In {
            write!(query, " {} IN (", self.field).unwrap();
            for (i, value) in self.values.iter().enumerate() {
                if i > 0 {
                    query.push_str(", ");
                }
                write!(query, "'{value}'").unwrap();
            }
            query.push(')');
        } else {
            write!(query, " {} {} '{}'", self.field, op_str, self.values[0]).unwrap();
        }
    }
}

/// Enum representing the operators used in where clauses
#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    In,
    Like,
    Equal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}
