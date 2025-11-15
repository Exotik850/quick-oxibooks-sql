use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, Token,
    parse::{Parse, ParseStream},
};

use crate::query::SqlQuery;

mod limit;
mod orderby;
mod query;
mod condition;

/// Builds a type-safe QuickBooks Online query at compile time.
///
/// This macro parses SQL-like syntax and generates a `Query<T>` struct that can be used to query
/// the QuickBooks Online API. Field names are automatically validated at compile time and converted
/// from snake_case to CamelCase to match QuickBooks naming conventions.
///
/// # Syntax
///
/// ```text
/// qb_sql!(
///     select [* | field1, field2, ...]
///     from EntityType
///     [where condition [and condition ...]]
///     [order by field [asc|desc] [, field [asc|desc] ...]]
///     [limit number [offset number]]
/// )
/// ```
///
/// # Supported Operators
///
/// - `=` - Equality comparison
/// - `>`, `<`, `>=`, `<=` - Numeric comparisons
/// - `like` - Pattern matching (use `%` as wildcard)
/// - `in` - Match against multiple values: `field in (val1, val2, ...)` or `field in (iterator)`
///
/// # Examples
///
/// Basic query with field selection:
/// ```ignore
/// use quick_oxibooks_sql::qb_sql;
/// use quickbooks_types::Customer;
///
/// let query = qb_sql!(
///     select display_name, balance from Customer
///     where balance >= 1000.0
///     order by display_name asc
///     limit 10
/// );
/// ```
///
/// Using Rust variables in conditions:
/// ```ignore
/// let min_balance = 500.0;
/// let name_pattern = "Acme%";
///
/// let query = qb_sql!(
///     select * from Customer
///     where balance >= min_balance
///     and display_name like name_pattern
/// );
/// ```
///
/// Using the `in` operator with a tuple or iterator:
/// ```ignore
/// // With literal values
/// let query = qb_sql!(
///     select * from Customer
///     where id in (1, 2, 3)
/// );
///
/// // With an iterator (single expression)
/// let ids = vec!["1", "2", "3"];
/// let query = qb_sql!(
///     select * from Customer
///     where id in (ids)
/// );
/// ```
///
/// Executing a query (requires the `api` feature):
/// ```ignore
/// use quick_oxibooks::{Environment, QBContext};
/// use ureq::Agent;
///
/// let client = Agent::new();
/// let qb = QBContext::new(Environment::SANDBOX, "company_id".into(), "token".into(), &client)?;
///
/// let results = query.execute(&qb, &client)?;
/// ```
///
/// # Notes
///
/// - Field names are automatically converted from snake_case to CamelCase (e.g., `display_name` → `DisplayName`)
/// - All field names are validated at compile time against the entity type
/// - The generated query can be converted to a string with `.query_string()` or by displaying it
/// - For the `in` operator, use a tuple for literals or a single iterator expression
#[proc_macro]
pub fn qb_sql(input: TokenStream) -> TokenStream {
    let query = syn::parse_macro_input!(input as SqlQuery);
    let expanded = query.expand();
    TokenStream::from(expanded)
}


/// Represents a field, possibly nested (e.g., address.city)
enum Field {
    Root(Ident),
    Nested(Ident, Box<Field>),
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let root: Ident = input.parse()?;
        if !input.peek(Token![.]) {
            return Ok(Field::Root(root));
        }
        input.parse::<Token![.]>()?;
        let nested = Field::parse(input)?;
        Ok(Field::Nested(root, Box::new(nested)))
    }
}

impl ToString for Field {
    fn to_string(&self) -> String {
        match self {
            Field::Root(ident) => ident.to_string(),
            Field::Nested(ident, nested) => format!("{}.{}", ident, nested.to_string()),
        }
    }
}

// Type Check for Field
impl quote::ToTokens for Field {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Field::Root(ident) => {
                ident.to_tokens(tokens);
            }
            Field::Nested(ident, nested) => {
                ident.to_tokens(tokens);
                tokens.extend(quote! { .unwrap() });
                nested.to_tokens(tokens);
            }
        }
    }
}

// Custom keywords
mod kw {
    syn::custom_keyword!(select);
    syn::custom_keyword!(from);
    syn::custom_keyword!(and);
    syn::custom_keyword!(order);
    syn::custom_keyword!(by);
    syn::custom_keyword!(limit);
    syn::custom_keyword!(offset);
    syn::custom_keyword!(asc);
    syn::custom_keyword!(desc);
    syn::custom_keyword!(like);
}
