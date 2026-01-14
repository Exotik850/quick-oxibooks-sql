extern crate self as quick_oxibooks_sql;

// Re-export the procedural macro
#[cfg(feature = "macros")]
pub use quick_oxibooks_sql_macro::qb_sql;

mod query;
pub use query::Query;
mod limit;
pub(crate) use limit::Limit;
mod order;
pub use order::{Order, OrderClause};
mod condition;
pub use condition::{Operator, TypedWhereClause, WhereClause};

#[cfg(feature = "macros")]
pub use pastey::paste;

pub mod traits {
    // --- 1. Wrapping Traits (_qb_wrap) ---
    // Normalizes values into Vec<T>

    pub trait QbWrapVec {
        type Item;
        fn _qb_wrap(self) -> Vec<Self::Item>;
    }
    impl<T> QbWrapVec for Vec<T> {
        type Item = T;
        fn _qb_wrap(self) -> Vec<T> {
            self
        }
    }

    pub trait QbWrapOpt {
        type Item;
        fn _qb_wrap(self) -> Vec<Self::Item>;
    }
    impl<T> QbWrapOpt for Option<T> {
        type Item = T;
        fn _qb_wrap(self) -> Vec<T> {
            self.into_iter().collect()
        }
    }

    pub trait QbWrapScalar {
        type Item;
        fn _qb_wrap(&self) -> Vec<Self::Item>;
    }
    impl<T> QbWrapScalar for &T {
        type Item = T;
        fn _qb_wrap(&self) -> Vec<T> {
            Vec::new()
        }
    }

    // --- 2. Access pub Traits (_qb_access) ---
    // Handles chaining and flattening

    // Priority 1: Nested Vec<Vec<T>> (Matches by Value)
    // Flattens two levels: Vec<Vec<T>> -> T
    pub trait QbAccessNested {
        type Inner;
        fn _qb_access<R, F>(self, f: F) -> Vec<R>
        where
            F: FnMut(Self::Inner) -> Vec<R>;
    }
    impl<T> QbAccessNested for Vec<Vec<T>> {
        type Inner = T;
        fn _qb_access<R, F>(self, f: F) -> Vec<R>
        where
            F: FnMut(T) -> Vec<R>,
        {
            self.into_iter().flatten().flat_map(f).collect()
        }
    }

    // Priority 2: Generic Vec<T> (Matches by Ref)
    // Flattens one level: Vec<T> -> T
    // Passes &T to closure to avoid moving out of Vec if we don't have ownership of elements (although here we do,
    // but using Ref ensures we don't conflict with Value match on Vec<Vec>)
    pub trait QbAccessGeneric {
        type Inner;
        fn _qb_access<R, F>(&self, f: F) -> Vec<R>
        where
            F: FnMut(&Self::Inner) -> Vec<R>;
    }
    impl<T> QbAccessGeneric for Vec<T> {
        type Inner = T;
        fn _qb_access<R, F>(&self, f: F) -> Vec<R>
        where
            F: FnMut(&T) -> Vec<R>,
        {
            self.iter().flat_map(f).collect()
        }
    }
}
pub use traits::*;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "macros")]
    use quickbooks_types::Attachable;
    use quickbooks_types::Customer;

    #[test]
    fn test_empty_query() {
        #[cfg(feature = "macros")]
        let qry = qb_sql!(select * from Customer);
        #[cfg(not(feature = "macros"))]
        let qry: Query<Customer> = Query::new();
        assert_eq!(qry.condition.len(), 0);
        assert_eq!(qry.order.len(), 0);
        assert!(qry.limit.is_none());
    }

    #[test]
    fn test_basic_query() {
        #[cfg(feature = "macros")]
        let qry = qb_sql!(
            select * from Customer
            where display_name like "John%"
        );
        #[cfg(not(feature = "macros"))]
        let qry: Query<Customer> = unsafe {
            Query::new().condition(WhereClause {
                field: "DisplayName",
                operator: Operator::Like,
                values: vec!["John%".to_string()],
            })
        };

        assert_eq!(qry.condition.len(), 1);
        assert_eq!(qry.condition[0].field, "DisplayName");
    }

    #[test]
    fn test_multiple_conditions() {
        let balance_min = 1000.0;
        #[cfg(feature = "macros")]
        let qry = qb_sql!(
            select * from Customer
            where display_name like "John%"
            and balance >= balance_min
        );
        #[cfg(not(feature = "macros"))]
        let qry: Query<Customer> = unsafe {
            Query::new()
                .condition(WhereClause {
                    field: "DisplayName",
                    operator: Operator::Like,
                    values: vec!["John%".into()],
                })
                .condition(WhereClause {
                    field: "Balance",
                    operator: Operator::GreaterEqual,
                    values: vec![balance_min.to_string()],
                })
        };

        assert_eq!(qry.condition.len(), 2);
    }

    #[test]
    fn test_order_by() {
        #[cfg(feature = "macros")]
        let qry = qb_sql!(
            select * from Customer
            where display_name like "John%"
            order by display_name asc, balance desc
        );
        #[cfg(not(feature = "macros"))]
        let qry: Query<Customer> = unsafe {
            Query::new()
                .condition(WhereClause {
                    field: "DisplayName",
                    operator: Operator::Like,
                    values: vec!["John%".into()],
                })
                .order("DisplayName", Order::Asc)
                .order("Balance", Order::Desc)
        };

        assert_eq!(qry.order.len(), 2);
        assert_eq!(qry.order[0].field, "DisplayName");
        assert_eq!(qry.order[0].order, Order::Asc);
    }

    #[test]
    fn test_limit_and_offset() {
        let offset_val = 5;
        #[cfg(feature = "macros")]
        let qry = qb_sql!(
            select * from Customer
            where display_name like "John%"
            limit 10 offset offset_val
        );
        #[cfg(not(feature = "macros"))]
        let qry: Query<Customer> = unsafe {
            Query::new()
                .condition(WhereClause {
                    field: "DisplayName",
                    operator: Operator::Like,
                    values: vec!["John%".into()],
                })
                .limit(10, Some(offset_val))
        };

        assert!(qry.limit.is_some());
        let limit = qry.limit.unwrap();
        assert_eq!(limit.number, 10);
        assert_eq!(limit.offset, Some(5));
    }

    #[test]
    fn test_qry_string_generation() {
        #[cfg(feature = "macros")]
        let qry = qb_sql!(
            select * from Customer
            where display_name like "John%"
            and id in (1, 2, 3)
            and balance >= 1000.0
            order by display_name asc, balance desc
            limit 10 offset 5
        );
        #[cfg(not(feature = "macros"))]
        let qry: Query<Customer> = unsafe {
            Query::new()
                .condition(WhereClause {
                    field: "DisplayName",
                    operator: Operator::Like,
                    values: vec!["John%".into()],
                })
                .condition(WhereClause {
                    field: "Id",
                    operator: Operator::In,
                    values: vec!["1".into(), "2".into(), "3".into()],
                })
                .condition(WhereClause {
                    field: "Balance",
                    operator: Operator::GreaterEqual,
                    values: vec!["1000".into()],
                })
                .order("DisplayName", Order::Asc)
                .order("Balance", Order::Desc)
                .limit(10, Some(5))
        };

        let qry_string = qry.query_string();
        let expected = "select * from Customer where DisplayName LIKE 'John%' and Id IN ('1', '2', '3') and Balance >= '1000' order by DisplayName ASC, Balance DESC LIMIT 10 OFFSET 5";
        assert_eq!(qry_string, expected);
    }

    #[test]
    fn test_in_operator() {
        #[cfg(feature = "macros")]
        let qry = qb_sql!(
            select * from Customer
            where id in (1, 2, 3, 4, 5)
        );

        #[cfg(not(feature = "macros"))]
        let qry: Query<Customer> = unsafe {
            Query::new().condition(WhereClause {
                field: "Id",
                operator: Operator::In,
                values: vec!["1".into(), "2".into(), "3".into(), "4".into(), "5".into()],
            })
        };

        assert_eq!(qry.condition.len(), 1);
        assert_eq!(qry.condition[0].field, "Id");
        assert_eq!(qry.condition[0].operator, Operator::In);
        assert_eq!(qry.condition[0].values.len(), 5);

        let qry_string = qry.query_string();
        assert_eq!(
            qry_string,
            "select * from Customer where Id IN ('1', '2', '3', '4', '5')"
        );
    }

    #[test]
    fn test_in_operator_with_strings() {
        let title1 = "Mr";
        let title2 = "Mrs";
        #[cfg(feature = "macros")]
        let qry = qb_sql!(
            select * from Customer
            where title in (title1, title2, "Dr")
        );

        #[cfg(not(feature = "macros"))]
        let qry: Query<Customer> = unsafe {
            Query::new().condition(WhereClause {
                field: "Title",
                operator: Operator::In,
                values: vec![title1.into(), title2.into(), "Dr".into()],
            })
        };

        assert_eq!(qry.condition.len(), 1);
        assert_eq!(qry.condition[0].values.len(), 3);

        let qry_string = qry.query_string();
        assert_eq!(
            qry_string,
            "select * from Customer where Title IN ('Mr', 'Mrs', 'Dr')"
        );
    }

    #[test]
    fn test_in_iterator() {
        let ids = vec![1, 2, 3, 4, 5];
        #[cfg(feature = "macros")]
        let qry = qb_sql!(
            select * from Customer
            where id in (ids)
        );
        #[cfg(not(feature = "macros"))]
        let qry: Query<Customer> = unsafe {
            Query::new().condition(WhereClause {
                field: "Id",
                operator: Operator::In,
                values: ids.iter().map(|id| id.to_string()).collect(),
            })
        };

        assert_eq!(qry.condition.len(), 1);
        assert_eq!(qry.condition[0].field, "Id");
        assert_eq!(qry.condition[0].operator, Operator::In);
        assert_eq!(qry.condition[0].values.len(), 5);

        let qry_string = qry.query_string();
        assert_eq!(
            qry_string,
            "select * from Customer where Id IN ('1', '2', '3', '4', '5')"
        );
    }

    #[test]
    fn test_nested_fields() {
        #[cfg(feature = "macros")]
        let qry = qb_sql!(
            select * from Customer
            where primary_email_addr.address like "%@example.com"
        );

        #[cfg(not(feature = "macros"))]
        let qry: Query<Customer> = unsafe {
            Query::new().condition(WhereClause {
                field: "PrimaryEmailAddr.Address",
                operator: Operator::Like,
                values: vec!["%@example.com".into()],
            })
        };

        assert_eq!(qry.condition.len(), 1);
        assert_eq!(qry.condition[0].field, "PrimaryEmailAddr.Address");

        let qry_string = qry.query_string();
        assert_eq!(
            qry_string,
            "select * from Customer where PrimaryEmailAddr.Address LIKE '%@example.com'"
        );
    }

    #[test]
    fn test_vec_fields() {
        let ids = vec!["1", "2", "3"];
        #[cfg(feature = "macros")]
        let qry = qb_sql!(
            select * from Attachable // comments work
            where attachable_ref.entity_ref.value in (ids)
        );

        #[cfg(not(feature = "macros"))]
        let qry: Query<Attachable> = unsafe {
            Query::new().condition(WhereClause {
                field: "AttachableRef.EntityRef.Value",
                operator: Operator::In,
                values: ids.iter().map(|f| f.to_string()).collect(),
            })
        };

        assert_eq!(qry.condition.len(), 1);
        assert_eq!(qry.condition[0].field, "AttachableRef.EntityRef.Value");
        assert_eq!(qry.condition[0].operator, Operator::In);
        assert_eq!(qry.condition[0].values.len(), 3);

        let qry_string = qry.query_string();
        assert_eq!(
            qry_string,
            "select * from Attachable where AttachableRef.EntityRef.Value IN ('1', '2', '3')"
        );
    }
}
