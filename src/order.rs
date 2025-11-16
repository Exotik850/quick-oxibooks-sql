use std::fmt::Write;

/// Struct representing an order clause in a query
#[derive(Debug, PartialEq, Clone)]
pub struct OrderClause {
    pub(crate) field: &'static str,
    pub(crate) order: Order,
}

impl OrderClause {
    pub fn extend_query(&self, query: &mut String) {
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
