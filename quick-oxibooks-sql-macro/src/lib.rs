use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, LitInt, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

/// Main procedural macro entry point
#[proc_macro]
pub fn qb_sql(input: TokenStream) -> TokenStream {
    let query = syn::parse_macro_input!(input as SqlQuery);
    let expanded = query.expand();
    TokenStream::from(expanded)
}

/// Represents the entire SQL query
struct SqlQuery {
    fields: FieldSelection,
    item_type: Type,
    conditions: Vec<Condition>,
    order_by: Option<OrderBy>,
    limit: Option<LimitClause>,
}

/// Field selection (SELECT * or SELECT field1, field2, ...)
enum FieldSelection {
    All,
    Specific(Vec<Ident>),
}

/// A single WHERE condition
struct Condition {
    field: Ident,
    operator: Operator,
    values: Vec<syn::Expr>,
}

/// Operator types
enum Operator {
    Equal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    In,
    Like,
}

/// ORDER BY clause
struct OrderBy {
    orders: Vec<OrderField>,
}

struct OrderField {
    field: Ident,
    direction: Option<OrderDirection>,
}

enum OrderDirection {
    Asc,
    Desc,
}

/// LIMIT clause with optional OFFSET
struct LimitClause {
    number: LitInt,
    offset: Option<syn::Expr>,
}

impl Parse for SqlQuery {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse SELECT
        input.parse::<kw::select>()?;

        // Parse field selection
        let fields = if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            FieldSelection::All
        } else {
            let field_list = Punctuated::<Ident, Token![,]>::parse_separated_nonempty(input)?;
            FieldSelection::Specific(field_list.into_iter().collect())
        };

        // Parse FROM
        input.parse::<kw::from>()?;
        let item_type: Type = input.parse()?;

        // Parse WHERE
        input.parse::<Token![where]>()?;

        // Parse first condition
        let mut conditions = vec![Condition::parse(input)?];

        // Parse additional AND conditions
        while input.peek(kw::and) {
            input.parse::<kw::and>()?;
            conditions.push(Condition::parse(input)?);
        }

        // Parse optional ORDER BY
        let order_by = if input.peek(kw::order) {
            Some(OrderBy::parse(input)?)
        } else {
            None
        };

        // Parse optional LIMIT
        let limit = if input.peek(kw::limit) {
            Some(LimitClause::parse(input)?)
        } else {
            None
        };

        Ok(SqlQuery {
            fields,
            item_type,
            conditions,
            order_by,
            limit,
        })
    }
}

impl Parse for Condition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field: Ident = input.parse()?;
        let operator = Operator::parse(input)?;

        let values = if matches!(operator, Operator::In) {
            // Parse parenthesized list for IN operator
            let content;
            syn::parenthesized!(content in input);
            let exprs = Punctuated::<syn::Expr, Token![,]>::parse_separated_nonempty(&content)?;
            exprs.into_iter().collect()
        } else {
            // Parse single value for other operators
            vec![input.parse()?]
        };

        Ok(Condition {
            field,
            operator,
            values,
        })
    }
}

impl Parse for Operator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Ok(Operator::Equal)
        } else if lookahead.peek(Token![<]) {
            input.parse::<Token![<]>()?;
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                Ok(Operator::LessEqual)
            } else {
                Ok(Operator::Less)
            }
        } else if lookahead.peek(Token![>]) {
            input.parse::<Token![>]>()?;
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                Ok(Operator::GreaterEqual)
            } else {
                Ok(Operator::Greater)
            }
        } else if lookahead.peek(Token![in]) {
            input.parse::<Token![in]>()?;
            Ok(Operator::In)
        } else if lookahead.peek(kw::like) {
            input.parse::<kw::like>()?;
            Ok(Operator::Like)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for OrderBy {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::order>()?;
        input.parse::<kw::by>()?;

        let orders = Punctuated::<OrderField, Token![,]>::parse_separated_nonempty(input)?;

        Ok(OrderBy {
            orders: orders.into_iter().collect(),
        })
    }
}

impl Parse for OrderField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field: Ident = input.parse()?;

        let direction = if input.peek(kw::asc) {
            input.parse::<kw::asc>()?;
            Some(OrderDirection::Asc)
        } else if input.peek(kw::desc) {
            input.parse::<kw::desc>()?;
            Some(OrderDirection::Desc)
        } else {
            None
        };

        Ok(OrderField { field, direction })
    }
}

impl Parse for LimitClause {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::limit>()?;
        let number: LitInt = input.parse()?;

        let offset = if input.peek(kw::offset) {
            input.parse::<kw::offset>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(LimitClause { number, offset })
    }
}

impl SqlQuery {
    fn expand(&self) -> proc_macro2::TokenStream {
        let item_type = &self.item_type;

        // Collect all fields for type checking
        let all_fields: Vec<&Ident> = {
            let mut fields = Vec::new();

            if let FieldSelection::Specific(ref select_fields) = self.fields {
                fields.extend(select_fields.iter());
            }

            fields.extend(self.conditions.iter().map(|c| &c.field));

            if let Some(ref order_by) = self.order_by {
                fields.extend(order_by.orders.iter().map(|o| &o.field));
            }

            fields
        };

        // Generate type checking code
        let type_check = if !all_fields.is_empty() {
            quote! {
                const _: () = {
                    fn _check_fields(v: #item_type) {
                        #(let _ = v.#all_fields;)*
                    }
                };
            }
        } else {
            quote! {}
        };

        // Generate field selection code
        let field_code = match &self.fields {
            FieldSelection::All => quote! {},
            FieldSelection::Specific(fields) => {
                let field_names: Vec<_> = fields
                    .iter()
                    .map(|f| {
                        let name = to_camel_case(&f.to_string());
                        quote! { stringify!(#name) }
                    })
                    .collect();

                quote! {
                    #(
                        unsafe {
                            query = query.field(#field_names);
                        }
                    )*
                }
            }
        };

        // Generate condition code
        let condition_code: Vec<_> = self
            .conditions
            .iter()
            .map(|c| {
                let field = &c.field;
                let field_name = to_camel_case(&field.to_string());
                let operator = c.operator.to_tokens();
                let values = &c.values;

                // For IN operator with a single expression, treat it as an iterator
                let values_code = if matches!(c.operator, Operator::In) && values.len() == 1 {
                    let expr = &values[0];
                    quote! {
                        {
                            let mut vals = Vec::new();
                            for v in #expr {
                                vals.push(v.to_string());
                            }
                            vals
                        }
                    }
                } else {
                    // Multiple values or non-IN operators: call to_string on each
                    quote! { vec![#(#values.to_string()),*] }
                };

                quote! {
                    let clause = WhereClause {
                        field: stringify!(#field_name),
                        operator: #operator,
                        values: #values_code,
                    };
                    unsafe {
                        query = query.condition(clause);
                    }
                }
            })
            .collect();

        // Generate order by code
        let order_code = if let Some(ref order_by) = self.order_by {
            let orders: Vec<_> = order_by
                .orders
                .iter()
                .map(|o| {
                    let field = &o.field;
                    let field_name = to_camel_case(&field.to_string());
                    let direction = match &o.direction {
                        Some(OrderDirection::Asc) => quote! { Order::Asc },
                        Some(OrderDirection::Desc) => quote! { Order::Desc },
                        None => quote! { Order::Asc },
                    };

                    quote! {
                        unsafe {
                            query = query.order(stringify!(#field_name), #direction);
                        }
                    }
                })
                .collect();

            quote! { #(#orders)* }
        } else {
            quote! {}
        };

        // Generate limit code
        let limit_code = if let Some(ref limit) = self.limit {
            let number = &limit.number;
            let offset_code = if let Some(ref offset) = limit.offset {
                quote! { Some(#offset) }
            } else {
                quote! { None }
            };

            quote! {
                query = query.limit(#number, #offset_code);
            }
        } else {
            quote! {}
        };

        quote! {
            {
                #type_check

                let mut query = Query::<#item_type>::new();

                #field_code
                #(#condition_code)*
                #order_code
                #limit_code

                query
            }
        }
    }
}

impl Operator {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        match self {
            Operator::Equal => quote! { Operator::Equal },
            Operator::Less => quote! { Operator::Less },
            Operator::Greater => quote! { Operator::Greater },
            Operator::LessEqual => quote! { Operator::LessEqual },
            Operator::GreaterEqual => quote! { Operator::GreaterEqual },
            Operator::In => quote! { Operator::In },
            Operator::Like => quote! { Operator::Like },
        }
    }
}

/// Convert snake_case to CamelCase
fn to_camel_case(s: &str) -> syn::Ident {
    let camel = s
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<String>();

    syn::Ident::new(&camel, proc_macro2::Span::call_site())
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
