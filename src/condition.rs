use std::fmt::{Display, Write};

/// Macro to create a typed where clause for a given `QuickBooks` item type
///
/// # Example
/// ```rust
/// use quick_oxibooks_sql::{qb_where, Operator, TypedWhereClause};
/// use quickbooks_types::Customer;
///
/// let clause: TypedWhereClause<Customer> = qb_where!(Customer, display_name, Operator::Like);
/// ```
#[macro_export]
#[cfg(feature = "macros")]
macro_rules! qb_where {
    ($item:ty, $first:ident$(.$nested:ident)*, $op:expr) => {
      {
        const _: () = {
          fn _type_check(v: $item) {
            let _ = v.$first$(.unwrap().$nested)*;
          }
        };
        unsafe {
          $crate::paste! {
            TypedWhereClause::<$item>::new(
                stringify!([<$first:camel>]$(.[<$nested:camel>])*),
                $op,
            )
          }
        }
      }
    };
}

// Struct representing a typed where clause in a query, for a more safe API
#[derive(Debug, PartialEq, Clone)]
pub struct TypedWhereClause<QB> {
    pub field: &'static str,
    pub operator: Operator,
    pub values: Vec<String>,
    _phantom: std::marker::PhantomData<QB>,
}

impl<QB> TypedWhereClause<QB> {
    /// Create a new typed where clause
    ///
    /// # Safety
    /// This function is unsafe because it accepts a raw string slice as the field name.
    /// The caller must ensure that the field name is valid and corresponds to a field in the
    /// `QuickBooks` entity.
    #[must_use]
    pub unsafe fn new(field: &'static str, operator: Operator) -> Self {
        Self {
            field,
            operator,
            values: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add a value to the typed where clause
    #[must_use]
    pub fn add_value<T: Display>(mut self, value: T) -> Self {
        self.values.push(value.to_string());
        self
    }

    /// Add multiple values to the typed where clause from an iterator
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

impl<T> From<TypedWhereClause<T>> for WhereClause {
    fn from(val: TypedWhereClause<T>) -> Self {
        WhereClause {
            field: val.field,
            operator: val.operator,
            values: val.values,
        }
    }
}

#[cfg(test)]
mod typed_where_tests {
    use super::*;
    use quickbooks_types::Customer;

    #[test]
    fn test_typed_where_clause_creation() {
        #[cfg(feature = "macros")]
        let clause = qb_where!(Customer, display_name, Operator::Like);
        #[cfg(not(feature = "macros"))]
        let clause: TypedWhereClause<Customer> =
            unsafe { TypedWhereClause::new("DisplayName", Operator::Like) };
        assert_eq!(clause.field, "DisplayName");
        assert_eq!(clause.operator, Operator::Like);
    }

    #[test]
    fn test_nested_typed_where_clause_creation() {
        #[cfg(feature = "macros")]
        let clause = qb_where!(Customer, primary_email_addr.address, Operator::Equal);
        #[cfg(not(feature = "macros"))]
        let clause: TypedWhereClause<Customer> =
            unsafe { TypedWhereClause::new("PrimaryEmailAddr.Address", Operator::Equal) };
        assert_eq!(clause.field, "PrimaryEmailAddr.Address");
        assert_eq!(clause.operator, Operator::Equal);
    }
}

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
