use crate::error::Error;
use actix_web::{
    FromRequest, HttpRequest,
    dev::Payload,
    web::{Json, Query},
};
use serde::Deserialize;
use std::pin::Pin;
use validator::Validate;

use shared::error::{CoreError, InternalError};

// TODO to be removed
#[derive(Debug, Clone, Copy, Default)]
struct _ValidatedPayload<T>(pub T);

impl<T> FromRequest for _ValidatedPayload<T>
where
    T: for<'de> Deserialize<'de> + Validate + 'static,
{
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = Json::<T>::from_request(req, payload);

        Box::pin(async move {
            let json = fut.await.map_err(|e| {
                Error::Core(CoreError::Internal(InternalError::MissingConfiguration(
                    e.to_string(),
                )))
            })?;

            json.validate()?;
            Ok(_ValidatedPayload(json.into_inner()))
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

impl<T> FromRequest for ValidatedQuery<T>
where
    T: for<'de> Deserialize<'de> + Validate + 'static,
{
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = Query::<T>::from_request(req, payload);

        Box::pin(async move {
            let json = fut.await.map_err(|e| {
                Error::Core(CoreError::Internal(InternalError::MissingConfiguration(
                    e.to_string(),
                )))
            })?;

            json.validate()?;
            Ok(ValidatedQuery(json.into_inner()))
        })
    }
}
