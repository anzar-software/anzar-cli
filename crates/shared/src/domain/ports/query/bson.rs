use mongodb::bson::{self, Document};

pub trait IntoBsonDocument {
    fn into_bson_document(self) -> Result<Document, bson::ser::Error>;
}
