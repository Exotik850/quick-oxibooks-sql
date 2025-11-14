# Quick-Oxibooks-SQL: SQL Integration for QuickBooks Online API in Rust

`quick-oxibooks-sql` is a Rust library that extends the functionality of the `quick-oxibooks` crate by providing SQL-like capabilities for interacting with QuickBooks Online (QBO) data. It allows developers to perform SQL-style queries and operations on QuickBooks entities, making it easier to manage and manipulate accounting data programmatically.

## Features

- **SQL-Like Querying**: Perform SQL-style queries on QuickBooks entities.
- **Type-Safe Operations**: Leverages Rust's type system for safe data handling.
- **Integration with quick-oxibooks**: Seamlessly works with the `quick-oxibooks` crate for API interactions.

## Usage

This crate provides two primary macros for building QuickBooks Online queries: `qb_sql!` and `qb_sql_str!`.

### Building a Query Object with `qb_sql!`

The `qb_sql!` macro parses a SQL-like query at compile time and generates a `Query` struct. This allows you to inspect the query components before generating the final string. You can use Rust variables directly within the query.

```rust
use quick_oxibooks_sql::qb_sql;
use quickbooks_types::Customer;

let min_balance = 1000.0;
let ids = vec!["1", "2", "3"];

// You can pass in an iterator or a slice for the `in` clause, or a tuple of literals.
let query = qb_sql!(
    select display_name, balance from Customer
    where balance >= min_balance
    and id in (ids)
    order by display_name asc
    limit 10
);

// The above is equivalent to:
// let query = qb_sql!(
//     select display_name, balance from Customer
//     where balance >= min_balance
//     and id in (1, 2, 3)
//     order by display_name asc
//     limit 10
// );

// The `query` variable is now a `Query<Customer>` struct.
// You can generate the final query string to be sent to the QBO API.
let query_string = query.query_string();

assert_eq!(
    query_string,
    "select DisplayName, Balance from Customer where Balance >= '1000' and Id IN ('1', '2', '3') order by DisplayName ASC LIMIT 10"
);
```

### Getting a Query String Directly with `qb_sql_str!`

If you just need the final query string, the `qb_sql_str!` macro provides a convenient shortcut. It works just like `qb_sql!` but returns the `String` directly.

```rust
use quick_oxibooks_sql::qb_sql_str;
use quickbooks_types::Customer;

let name_filter = "John%";
let query_string = qb_sql_str!(
    select * from Customer
    where display_name like name_filter
);

assert_eq!(
    query_string,
    "select * from Customer where DisplayName LIKE 'John%'"
);
```

### Supported SQL Syntax

The macros support a subset of SQL syntax relevant to the QuickBooks Online API:

- **`SELECT`**: Select all fields (`*`) or a comma-separated list of specific fields (e.g., `select display_name, balance`).
- **`FROM`**: Specify the QuickBooks entity (e.g., `from Customer`). The entity must implement the `QBItem` trait from `quickbooks-types`.
- **`WHERE`**: Filter results using one or more conditions joined by `and`.
  - **Operators**: `=`, `like`, `>`, `<`, `>=`, `<=`, `in`.
  - The `in` operator accepts a tuple of literals or a variable that is a `Vec` or slice (e.g., `id in (1, 2, 3)` or `id in (my_ids)`).
- **`ORDER BY`**: Sort results by one or more fields, with `asc` or `desc` direction (e.g., `order by display_name asc, balance desc`).
- **`LIMIT`**: Restrict the number of records returned.
- **`OFFSET`**: Start the result set at a specific offset, for pagination.

more information about the syntax can be found in the [QuickBooks Online API documentation](https://developer.intuit.com/app/developer/qbo/docs/learn/explore-the-quickbooks-online-api/data-queries).
