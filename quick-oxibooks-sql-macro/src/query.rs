use quote::quote;
use syn::{
    Token, Type,
    parse::{Parse, ParseStream},
};

use crate::{
    OptionField,
    condition::{Condition, Operator},
    kw,
    limit::LimitClause,
    orderby::{OrderBy, OrderDirection},
};

/// Represents the entire SQL query
pub struct SqlQuery {
    pub(crate) item_type: Type,
    pub(crate) conditions: Vec<Condition>,
    pub(crate) order_by: Option<OrderBy>,
    pub(crate) limit: Option<LimitClause>,
}

impl Parse for SqlQuery {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse select * from
        input.parse::<kw::select>()?;
        input.parse::<Token![*]>()?;
        input.parse::<kw::from>()?;

        let item_type: Type = input.parse()?;

        let mut conditions = vec![];
        if input.peek(Token![where]) {
            // Parse WHERE
            input.parse::<Token![where]>()?;
            // Parse first condition
            conditions.push(Condition::parse(input)?);
            // Parse additional AND conditions
            while input.peek(kw::and) {
                input.parse::<kw::and>()?;
                conditions.push(Condition::parse(input)?);
            }
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
            item_type,
            conditions,
            order_by,
            limit,
        })
    }
}

impl SqlQuery {
    pub(crate) fn expand(&self) -> proc_macro2::TokenStream {
        let item_type = &self.item_type;

        // Collect all fields for type checking
        let all_fields: Vec<&OptionField> = {
            let mut fields = Vec::new();

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

        // Generate condition code
        let condition_code: Vec<_> = self
            .conditions
            .iter()
            .map(|c| {
                let field = &c.field;
                let operator = c.operator.to_tokens();
                let values = &c.values;

                // For IN operator with a single expression, treat it as an iterator
                let values_code = if matches!(c.operator, Operator::In) && values.len() == 1 {
                    let expr = &values[0];
                    quote! {
                      #expr.into_iter().map(|v| v.to_string()).collect::<Vec<String>>()
                    }
                } else {
                    // Multiple values or non-IN operators: call to_string on each
                    quote! { vec![#(#values.to_string()),*] }
                };

                let field_str = field.to_string();
                quote! {
                    let clause = WhereClause {
                        field: #field_str,
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
                    let field_str = field.to_string();
                    let direction = match &o.direction {
                        Some(OrderDirection::Asc) => quote! { Order::Asc },
                        Some(OrderDirection::Desc) => quote! { Order::Desc },
                        None => quote! { Order::Asc },
                    };

                    quote! {
                        unsafe {
                            query = query.order(#field_str, #direction);
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

                #(#condition_code)*
                #order_code
                #limit_code

                query
            }
        }
    }
}
