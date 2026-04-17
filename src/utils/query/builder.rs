use super::value::{DbValue, Op, Operation};

#[derive(Clone)]
pub struct QueryBuilder {
    pub filters: Vec<Operation>,
    pub updates: Vec<Operation>,
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            filters: vec![],
            updates: vec![],
        }
    }

    pub fn eq(mut self, field: &'static str, value: impl Into<DbValue>) -> Self {
        self.filters.push(Operation {
            field,
            op: Op::Eq,
            value: value.into(),
        });
        self
    }

    pub fn set(mut self, field: &'static str, value: impl Into<DbValue>) -> Self {
        self.updates.push(Operation {
            field,
            op: Op::Set,
            value: value.into(),
        });
        self
    }
}
