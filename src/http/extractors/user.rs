use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload};
use std::future::{Ready, ready};

use crate::application::model::User;
use crate::error::{AuthError, CredentialField, Error};

pub struct AuthenticatedUser(pub User);

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        tracing::error!("user is not found");
        match req.extensions().get::<User>() {
            Some(user) => ready(Ok(AuthenticatedUser(user.clone()))),
            None => ready(Err(Error::Unauthenticated(AuthError::InvalidCredentials {
                field: CredentialField::Token,
            }))),
        }
    }
}
