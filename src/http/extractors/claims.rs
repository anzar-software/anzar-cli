use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload};
use std::future::{Ready, ready};

use crate::domain::model::Claims;
use crate::error::Error;

impl FromRequest for Claims {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<Claims>() {
            Some(claims) => ready(Ok(claims.clone())),
            None => ready(Ok(Claims::default())),
        }
    }
}
