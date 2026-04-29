use actix_web::{dev::ServiceRequest, web};

use crate::{
    config::{AnzarConfiguration, AppState},
    error::{Error, InternalError},
    scopes::auth::service::AuthService,
};

pub fn extract_auth_service(req: &ServiceRequest) -> Result<AuthService, Error> {
    req.app_data::<web::Data<AppState>>()
        .map(|state| state.auth_service.clone())
        .ok_or_else(|| {
            tracing::error!(
                error.code = "InternalError::MissingAppData",
                "Failed to extract auth_service from app_data"
            );
            Error::Internal(InternalError::MissingAppData(
                "AppState not registered".into(),
            ))
        })
}
pub fn extract_configuration_service(req: &ServiceRequest) -> Result<AnzarConfiguration, Error> {
    req.app_data::<web::Data<AppState>>()
        .map(|state| state.configuration.clone())
        .ok_or_else(|| {
            tracing::error!(
                error.code = "InternalError::MissingAppData",
                "Failed to extract configuration from app_data"
            );
            Error::Internal(InternalError::MissingAppData(
                "AppState not registered".into(),
            ))
        })
}
