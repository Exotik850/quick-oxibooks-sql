// Re-export the procedural macro
pub use quick_oxibooks_sql_macro::qb_sql;

mod query;
pub use query::Query;
mod limit;
pub(crate) use limit::Limit;
mod order;
pub use order::{Order, OrderClause};
mod condition;
pub use condition::{Operator, WhereClause};

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
