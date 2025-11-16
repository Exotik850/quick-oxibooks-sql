use syn::{
    Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{OptionField, kw};

/// ORDER BY clause
pub struct OrderBy {
    pub(crate) orders: Vec<OrderField>,
}

pub struct OrderField {
    pub(crate) field: OptionField,
    pub(crate) direction: Option<OrderDirection>,
}

pub enum OrderDirection {
    Asc,
    Desc,
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
        let field: OptionField = input.parse()?;

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
