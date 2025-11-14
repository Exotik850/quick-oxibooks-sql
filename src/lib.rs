// Re-export the procedural macro
pub use quick_oxibooks_sql_macro::qb_sql;
use quickbooks_types::QBItem;

/// Struct representing a SQL-like query for QuickBooks entities
#[derive(Debug, PartialEq, Clone)]
pub struct Query<QB> {
    fields: Vec<&'static str>,
    condition: Vec<WhereClause>,
    order: Vec<OrderClause>,
    limit: Option<Limit>,
    _phantom: std::marker::PhantomData<QB>,
}

impl<QB: QBItem> Query<QB> {
    pub fn new() -> Self {
        Query {
            fields: Vec::new(),
            condition: Vec::new(),
            order: Vec::new(),
            limit: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub unsafe fn field(mut self, field: &'static str) -> Self {
        self.fields.push(field);
        self
    }

    pub unsafe fn condition(mut self, condition: WhereClause) -> Self {
        self.condition.push(condition);
        self
    }

    pub unsafe fn order(mut self, field: &'static str, order: Order) -> Self {
        self.order.push(OrderClause { field, order });
        self
    }

    pub fn limit(mut self, number: u32, offset: Option<u32>) -> Self {
        self.limit = Some(Limit { number, offset });
        self
    }

    pub fn query_string(&self) -> String {
        let mut query = String::new();

        match &self.fields[..] {
            [] => query.push_str("select *"),
            fields => {
                query.push_str("select ");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        query.push_str(", ");
                    }
                    query.push_str(field);
                }
            }
        }

        query.push_str(&format!(" from {}", QB::name()));

        if !self.condition.is_empty() {
            query.push_str(" where");
            for (i, cond) in self.condition.iter().enumerate() {
                if i > 0 {
                    query.push_str(" and");
                }
                cond.extend_query(&mut query);
            }
        }

        if !self.order.is_empty() {
            query.push_str(" order by");
            for (i, ord) in self.order.iter().enumerate() {
                if i > 0 {
                    query.push_str(",");
                }
                ord.extend_query(&mut query);
            }
        }

        if let Some(limit) = &self.limit {
            limit.extend_query(&mut query);
        }

        query
    }

    #[cfg(feature = "api")]
    pub fn execute(
        &self,
        qb: &quick_oxibooks::QBContext,
        client: &ureq::Agent,
    ) -> Result<Vec<QB>, quick_oxibooks::error::APIError> {
        unsafe { quick_oxibooks::functions::query::qb_query_raw::<QB>(self, qb, client) }
    }
}

impl<QB: QBItem> std::fmt::Display for Query<QB> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.query_string())
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Limit {
    number: u32,
    offset: Option<u32>,
}

impl Limit {
    fn extend_query(&self, query: &mut String) {
        query.push_str(&format!(" LIMIT {}", self.number));
        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrderClause {
    field: &'static str,
    order: Order,
}

impl OrderClause {
    fn extend_query(&self, query: &mut String) {
        query.push_str(&format!(
            " {} {}",
            self.field,
            match self.order {
                Order::Asc => "ASC",
                Order::Desc => "DESC",
            }
        ));
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WhereClause {
    pub field: &'static str,
    pub operator: Operator,
    pub values: Vec<String>,
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
            query.push_str(&format!(" {} IN (", self.field));
            for (i, value) in self.values.iter().enumerate() {
                if i > 0 {
                    query.push_str(", ");
                }
                query.push_str(&format!("'{}'", value));
            }
            query.push(')');
        } else {
            query.push_str(&format!(" {} {} '{}'", self.field, op_str, self.values[0]));
        }
    }
}

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
    fn test_field_selection() {
        let query = qb_sql!(
            select display_name, balance from Customer
            where display_name like "John%"
        );

        assert_eq!(query.fields.len(), 2);
        assert_eq!(query.fields[0], "DisplayName");
        assert_eq!(query.fields[1], "Balance");
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
            select display_name, balance from Customer
            where display_name like "John%"
            and id in (1, 2, 3)
            and balance >= 1000.0
            order by display_name asc, balance desc
            limit 10 offset 5
        );

        let query_string = query.query_string();
        let expected = "select DisplayName, Balance from Customer where DisplayName LIKE 'John%' and Id IN ('1', '2', '3') and Balance >= '1000' order by DisplayName ASC, Balance DESC LIMIT 10 OFFSET 5";
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
            select display_name from Customer
            where title in (title1, title2, "Dr")
        );

        assert_eq!(query.condition.len(), 1);
        assert_eq!(query.condition[0].values.len(), 3);

        let query_string = query.query_string();
        assert_eq!(
            query_string,
            "select DisplayName from Customer where Title IN ('Mr', 'Mrs', 'Dr')"
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
}
