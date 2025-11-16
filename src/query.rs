use quickbooks_types::QBItem;

use crate::{Limit, Order, OrderClause, WhereClause};

/// Struct representing a SQL-like query for `QuickBooks` entities
#[derive(Debug, PartialEq, Clone)]
pub struct Query<QB> {
    pub(crate) condition: Vec<WhereClause>,
    pub(crate) order: Vec<OrderClause>,
    pub(crate) limit: Option<Limit>,
    _phantom: std::marker::PhantomData<QB>,
}

impl<QB: QBItem> Default for Query<QB> {
    fn default() -> Self {
        Self::new()
    }
}

impl<QB: QBItem> Query<QB> {
    /// Create a new empty query
    #[must_use]
    pub fn new() -> Self {
        Query {
            condition: Vec::new(),
            order: Vec::new(),
            limit: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add a condition to the query
    ///
    /// # Safety
    /// This function is unsafe because it accepts a raw `WhereClause`.
    /// The caller must ensure that the `WhereClause` is valid and corresponds to the `QuickBooks` entity.
    #[must_use]
    pub unsafe fn condition(mut self, condition: WhereClause) -> Self {
        self.condition.push(condition);
        self
    }

    /// Add a typed condition to the query
    ///
    /// This is safe because the typed where clause ensures that the field and values are valid for the `QuickBooks` entity.
    #[must_use]
    pub fn typed_condition(mut self, condition: crate::TypedWhereClause<QB>) -> Self {
        self.condition.push(condition.into());
        self
    }

    /// Add an order clause to the query
    ///
    /// # Safety
    /// This function is unsafe because it accepts a raw string slice as the field name.
    /// The caller must ensure that the field name is valid and corresponds to a field in the `QuickBooks` entity.
    #[must_use]
    pub unsafe fn order(mut self, field: &'static str, order: Order) -> Self {
        self.order.push(OrderClause { field, order });
        self
    }

    /// Set a limit on the number of results returned by the query
    #[must_use]
    pub fn limit(mut self, number: u32, offset: Option<u32>) -> Self {
        self.limit = Some(Limit { number, offset });
        self
    }

    /// Generate the query string
    #[must_use]
    pub fn query_string(&self) -> String {
        let mut query = format!("select * from {}", QB::name());

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
                    query.push(',');
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
    /// Execute the query against the `QuickBooks` API, returning a vector of results or an error
    ///
    /// # Errors
    /// This function will return an error if the API request fails or if the response cannot be parsed.
    pub fn execute(
        &self,
        qb: &quick_oxibooks::QBContext,
        client: &ureq::Agent,
    ) -> Result<Vec<QB>, quick_oxibooks::error::APIError> {
        // Safety: The query has been constructed using the provided methods,
        // ensuring that it is valid for the QuickBooks entity QB.
        unsafe { quick_oxibooks::functions::query::qb_query_raw::<QB>(self, qb, client) }
    }
}

impl<QB: QBItem> std::fmt::Display for Query<QB> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.query_string())
    }
}
