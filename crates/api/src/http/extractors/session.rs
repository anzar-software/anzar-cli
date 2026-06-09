use crate::error::Error;
use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload};
use std::future::{Ready, ready};

use shared::domain::model::Session;

pub struct SessionExtractor(pub Session);

impl FromRequest for SessionExtractor {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<Session>() {
            Some(session) => ready(Ok(SessionExtractor(session.clone()))),
            None => ready(Ok(SessionExtractor(Session::default()))),
        }
    }
}
