use quote::quote;
use syn::{
    Expr, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{OptionField, kw};

/// A single WHERE condition
pub struct Condition {
    pub(crate) field: OptionField,
    pub(crate) operator: Operator,
    pub(crate) values: Vec<Expr>,
}

/// Operator types
pub enum Operator {
    Equal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    In,
    Like,
}

impl Operator {
    pub(crate) fn to_tokens(&self) -> proc_macro2::TokenStream {
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

impl Parse for Condition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field: OptionField = input.parse()?;
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
