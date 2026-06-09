use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload};
use std::future::{Ready, ready};

use crate::error::Error;
use shared::domain::model::Claims;

struct _ClaimsExtractor(pub Claims);

impl FromRequest for _ClaimsExtractor {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<Claims>() {
            Some(claims) => ready(Ok(_ClaimsExtractor(claims.clone()))),
            None => ready(Ok(_ClaimsExtractor(Claims::default()))),
        }
    }
}
