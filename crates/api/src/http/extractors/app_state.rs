use std::future::{Ready, ready};

use actix_web::{FromRequest, HttpRequest, dev::Payload, web::Data};
use actix_web::{dev::ServiceRequest, web};
use shared::error::{CoreError, InternalError};

use crate::error::Error;
use crate::state::AppState;

pub struct AppStateExtractor(pub AppState);

impl FromRequest for AppStateExtractor {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let result = req
            .app_data::<Data<AppState>>()
            .map(|state| state.get_ref())
            .map(|sm| AppStateExtractor(sm.clone()))
            .ok_or_else(|| {
                Error::Core(CoreError::Internal(InternalError::MissingConfiguration(
                    "AppState not registered".into(),
                )))
            });

        ready(result)
    }
}

pub fn extract_app_state(req: &ServiceRequest) -> Result<AppState, Error> {
    req.app_data::<web::Data<AppState>>()
        .map(|state| state.get_ref().clone())
        .ok_or_else(|| {
            tracing::error!(
                error.code = "InternalError::MissingAppData",
                "Failed to extract configuration from app_data"
            );
            Error::Core(CoreError::Internal(InternalError::MissingConfiguration(
                "AppState not registered".into(),
            )))
        })
}
