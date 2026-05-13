use mongodb::bson::{Document, doc};

use super::{
    builder::QueryBuilder,
    value::{DbValue, Op},
};

pub trait IntoDbFilter {
    fn into_mongo_filter(self) -> Document;
    fn into_mongo_update(self) -> Document;
    fn into_postgres_filter(self, offset: usize) -> (String, Vec<DbValue>);
    fn into_postgres_update(self) -> (String, Vec<DbValue>);
    fn into_sqlite_filter(self) -> (String, Vec<DbValue>);
    fn into_sqlite_update(self) -> (String, Vec<DbValue>);
}

impl IntoDbFilter for QueryBuilder {
    fn into_mongo_filter(self) -> Document {
        let mut doc = Document::new();

        // filters
        for op in self.filters {
            let value = op.value.into_boson(op.field);
            dbg!(&value);
            let field = if op.field == "id" { "_id" } else { op.field };

            match op.op {
                Op::Eq => {
                    doc.insert(field, value);
                }
                Op::Ne => {
                    doc.insert(field, doc! { "$ne": value });
                }
                Op::Gt => {
                    doc.insert(field, doc! { "$gt": value });
                }
                Op::Lt => {
                    doc.insert(field, doc! { "$lt": value });
                }
                Op::In => {
                    doc.insert(field, doc! { "$in": value });
                }
                Op::Null => {
                    doc.insert(field, doc! { "$in": [mongodb::bson::Bson::Null] });
                }
                _ => {}
            }
        }

        doc
    }

    fn into_mongo_update(self) -> Document {
        let mut doc = Document::new();

        // updates
        if !self.updates.is_empty() {
            let mut set_doc = Document::new();
            let mut inc_doc = Document::new();
            let mut unset_doc = Document::new();

            for op in self.updates {
                let value = op.value.into_boson(op.field);

                match op.op {
                    Op::Set => {
                        set_doc.insert(op.field, value);
                    }
                    Op::Inc => {
                        inc_doc.insert(op.field, value);
                    }
                    Op::Unset => {
                        unset_doc.insert(op.field, "");
                    }
                    _ => {}
                }
            }

            if !set_doc.is_empty() {
                doc.insert("$set", set_doc);
            }
            if !inc_doc.is_empty() {
                doc.insert("$inc", inc_doc);
            }
            if !unset_doc.is_empty() {
                doc.insert("$unset", unset_doc);
            }
        }

        doc
    }
    fn into_postgres_filter(self, offset: usize) -> (String, Vec<DbValue>) {
        let clause = self
            .filters
            .iter()
            .enumerate()
            .map(|(i, f)| match f.op {
                Op::Eq => format!("\"{}\" = ${}", f.field, i + 1 + offset),
                Op::Null => format!("\"{}\" IS NULL", f.field), // no binding needed
                Op::Ne => format!("\"{}\" != ${}", f.field, i + 1 + offset),
                Op::Gt => format!("\"{}\" > ${}", f.field, i + 1 + offset),
                Op::Lt => format!("\"{}\" < ${}", f.field, i + 1 + offset),
                Op::In => format!("\"{}\" = ANY(${})", f.field, i + 1 + offset),
                _ => unreachable!("update ops not valid in filter context"),
            })
            .collect::<Vec<_>>()
            .join(" AND ");

        let values = self.filters.iter().map(|f| f.value.clone()).collect();
        (clause, values)
    }
    fn into_postgres_update(self) -> (String, Vec<DbValue>) {
        let clause = self
            .updates
            .iter()
            .enumerate()
            .map(|(i, f)| match f.op {
                Op::Set => format!("\"{}\" = ${}", f.field, i + 1),
                Op::Inc => format!("\"{}\" = \"{}\" + ${}", f.field, f.field, i + 1),
                Op::Unset => format!("\"{}\" = NULL", f.field),
                _ => unreachable!("filter ops not valid in update context"),
            })
            .collect::<Vec<_>>()
            .join(" , ");

        let values = self.updates.iter().map(|f| f.value.clone()).collect();
        (clause, values)
    }

    // -- Sqlite
    fn into_sqlite_filter(self) -> (String, Vec<DbValue>) {
        let clause = self
            .filters
            .iter()
            .map(|f| match f.op {
                Op::Eq => format!("\"{}\" = ?", f.field),
                Op::Null => format!("\"{}\" IS NULL", f.field),
                Op::Ne => format!("\"{}\" != ?", f.field),
                Op::Gt => format!("\"{}\" > ?", f.field),
                Op::Lt => format!("\"{}\" < ?", f.field),
                // Op::In => format!("\"{}\" = ANY(?{})", f.field, i + 1),
                Op::In => match &f.value {
                    DbValue::List(list) => {
                        let placeholders = list.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                        format!("\"{}\" IN ({})", f.field, placeholders)
                    }
                    _ => unreachable!("In op must have List value"),
                },
                _ => unreachable!("update ops not valid in filter context"),
            })
            .collect::<Vec<_>>()
            .join(" AND ");

        // let values = self.filters.iter().map(|f| f.value.clone()).collect();
        let values = self
            .filters
            .iter()
            .flat_map(|f| match &f.value {
                DbValue::List(list) => list.clone(),
                _ => vec![f.value.clone()],
            })
            .collect();

        (clause, values)
    }
    fn into_sqlite_update(self) -> (String, Vec<DbValue>) {
        let clause = self
            .updates
            .iter()
            .map(|f| match f.op {
                Op::Set => format!("\"{}\" = ?", f.field),
                Op::Inc => format!("\"{}\" = \"{}\" + ?", f.field, f.field),
                Op::Unset => format!("\"{}\" = NULL", f.field),
                _ => unreachable!("filter ops not valid in update context"),
            })
            .collect::<Vec<_>>()
            .join(" , ");

        let values = self.updates.iter().map(|f| f.value.clone()).collect();
        (clause, values)
    }
}
