use syn::Expr;
use syn::LitInt;
use syn::parse::Parse;
use syn::parse::ParseStream;

use crate::kw;

/// LIMIT clause with optional OFFSET
pub struct LimitClause {
    pub(crate) number: LitInt,
    pub(crate) offset: Option<Expr>,
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
