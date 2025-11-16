use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, Token,
    parse::{Parse, ParseStream},
};

use crate::query::SqlQuery;

mod condition;
mod limit;
mod orderby;
mod query;

/// Builds a type-safe `QuickBooks` Online query at compile time.
///
/// This macro parses SQL-like syntax and generates a `Query<T>` struct that can be used to query
/// the `QuickBooks` Online API. Field names are automatically validated at compile time and converted
/// from `snake_case` to CamelCase to match `QuickBooks` naming conventions.
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
/// - Field names are automatically converted from `snake_case` to CamelCase (e.g., `display_name` → `DisplayName`)
/// - All field names are validated at compile time against the entity type
/// - The generated query can be converted to a string with `.query_string()` or by displaying it
/// - For the `in` operator, use a tuple for literals or a single iterator expression
#[proc_macro]
pub fn qb_sql(input: TokenStream) -> TokenStream {
    let query = syn::parse_macro_input!(input as SqlQuery);
    let expanded = query.expand();
    TokenStream::from(expanded)
}

/// Represents a `ExprField` that is type checked with options for nested fields
///
/// At least one field is required, and additional fields can be chained using dot notation.
/// For example: `field1.field2.field3`
struct OptionField(Vec<Ident>);

impl OptionField {
    fn to_camel_case_string(&self) -> String {
        let camel_parts: Vec<String> = self
            .0
            .iter()
            .map(|ident| snake_to_camel_case(&ident.to_string()))
            .collect();
        camel_parts.join(".")
    }
}

impl Parse for OptionField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut idents = Vec::new();

        // Parse the first identifier
        idents.push(input.parse::<Ident>()?);

        // Parse any additional identifiers separated by dots
        while input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            idents.push(input.parse::<Ident>()?);
        }

        Ok(OptionField(idents))
    }
}

fn snake_to_camel_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

impl std::fmt::Display for OptionField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_camel_case_string())
    }
}

// Type Check for OptionExprField
impl quote::ToTokens for OptionField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.0.is_empty() {
            return;
        }

        let mut iter = self.0.iter();
        let first = iter.next().unwrap();

        let combined = iter.fold(quote! { #first }, |acc, ident| {
            quote! { #acc .unwrap(). #ident }
        });

        tokens.extend(combined);
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
