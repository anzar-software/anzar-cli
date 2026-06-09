use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload};
use std::future::{Ready, ready};

use crate::error::{CredentialField, Error};
use shared::{
    application::model::User,
    error::{AuthError, CoreError},
};

pub struct AuthenticatedUser(pub User);

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        tracing::error!("user is not found");
        match req.extensions().get::<User>() {
            Some(user) => ready(Ok(AuthenticatedUser(user.clone()))),

            None => ready(Err(CoreError::from(AuthError::InvalidCredentials {
                field: CredentialField::Token,
            })
            .into())),
        }
    }
}
