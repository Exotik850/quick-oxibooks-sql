use std::fmt::Write;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Limit {
    pub(crate) number: u32,
    pub(crate) offset: Option<u32>,
}

impl Limit {
    pub fn extend_query(&self, query: &mut String) {
        write!(query, " LIMIT {}", self.number).unwrap();
        if let Some(offset) = self.offset {
            write!(query, " OFFSET {offset}").unwrap();
        }
    }
}
