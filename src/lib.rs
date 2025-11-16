use std::fmt::Display;
use std::fmt::Write;

// Re-export the procedural macro
pub use quick_oxibooks_sql_macro::qb_sql;

mod query;
pub use query::Query;
mod limit;
pub(crate) use limit::Limit;

/// Struct representing an order clause in a query
#[derive(Debug, PartialEq, Clone)]
struct OrderClause {
    field: &'static str,
    order: Order,
}

impl OrderClause {
    fn extend_query(&self, query: &mut String) {
        write!(
            query,
            " {} {}",
            self.field,
            match self.order {
                Order::Asc => "ASC",
                Order::Desc => "DESC",
            }
        )
        .unwrap();
    }
}

/// Enum representing the order direction in a query
#[derive(Debug, PartialEq, Clone)]
pub enum Order {
    Asc,
    Desc,
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
    fn extend_query(&self, query: &mut String) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use quickbooks_types::Customer;

    #[test]
    fn test_empty_query() {
        let query = qb_sql!(select * from Customer);
        assert_eq!(query.condition.len(), 0);
        assert_eq!(query.order.len(), 0);
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_basic_query() {
        let query = qb_sql!(
            select * from Customer
            where display_name like "John%"
        );

        assert_eq!(query.condition.len(), 1);
        assert_eq!(query.condition[0].field, "DisplayName");
    }

    #[test]
    fn test_multiple_conditions() {
        let balance_min = 1000.0;
        let query = qb_sql!(
            select * from Customer
            where display_name like "John%"
            and balance >= balance_min
        );

        assert_eq!(query.condition.len(), 2);
    }

    #[test]
    fn test_order_by() {
        let query = qb_sql!(
            select * from Customer
            where display_name like "John%"
            order by display_name asc, balance desc
        );

        assert_eq!(query.order.len(), 2);
        assert_eq!(query.order[0].field, "DisplayName");
        assert_eq!(query.order[0].order, Order::Asc);
    }

    #[test]
    fn test_limit_and_offset() {
        let offset_val = 5;
        let query = qb_sql!(
            select * from Customer
            where display_name like "John%"
            limit 10 offset offset_val
        );

        assert!(query.limit.is_some());
        let limit = query.limit.unwrap();
        assert_eq!(limit.number, 10);
        assert_eq!(limit.offset, Some(5));
    }

    #[test]
    fn test_query_string_generation() {
        let query = qb_sql!(
            select * from Customer
            where display_name like "John%"
            and id in (1, 2, 3)
            and balance >= 1000.0
            order by display_name asc, balance desc
            limit 10 offset 5
        );

        let query_string = query.query_string();
        let expected = "select * from Customer where DisplayName LIKE 'John%' and Id IN ('1', '2', '3') and Balance >= '1000' order by DisplayName ASC, Balance DESC LIMIT 10 OFFSET 5";
        assert_eq!(query_string, expected);
    }

    #[test]
    fn test_in_operator() {
        let query = qb_sql!(
            select * from Customer
            where id in (1, 2, 3, 4, 5)
        );

        assert_eq!(query.condition.len(), 1);
        assert_eq!(query.condition[0].field, "Id");
        assert_eq!(query.condition[0].operator, Operator::In);
        assert_eq!(query.condition[0].values.len(), 5);

        let query_string = query.query_string();
        assert_eq!(
            query_string,
            "select * from Customer where Id IN ('1', '2', '3', '4', '5')"
        );
    }

    #[test]
    fn test_in_operator_with_strings() {
        let title1 = "Mr";
        let title2 = "Mrs";
        let query = qb_sql!(
            select * from Customer
            where title in (title1, title2, "Dr")
        );

        assert_eq!(query.condition.len(), 1);
        assert_eq!(query.condition[0].values.len(), 3);

        let query_string = query.query_string();
        assert_eq!(
            query_string,
            "select * from Customer where Title IN ('Mr', 'Mrs', 'Dr')"
        );
    }

    #[test]
    fn test_in_iterator() {
        let ids = vec![1, 2, 3, 4, 5];
        let query = qb_sql!(
            select * from Customer
            where id in (ids)
        );

        assert_eq!(query.condition.len(), 1);
        assert_eq!(query.condition[0].field, "Id");
        assert_eq!(query.condition[0].operator, Operator::In);
        assert_eq!(query.condition[0].values.len(), 5);

        let query_string = query.query_string();
        assert_eq!(
            query_string,
            "select * from Customer where Id IN ('1', '2', '3', '4', '5')"
        );
    }

    #[test]
    fn test_nested_fields() {
        let query = qb_sql!(
            select * from Customer
            where primary_email_addr.address like "%@example.com"
        );

        assert_eq!(query.condition.len(), 1);
        assert_eq!(query.condition[0].field, "PrimaryEmailAddr.Address");

        let query_string = query.query_string();
        assert_eq!(
            query_string,
            "select * from Customer where PrimaryEmailAddr.Address LIKE '%@example.com'"
        );
    }
}
