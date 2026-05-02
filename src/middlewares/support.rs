use actix_web::{dev::ServiceRequest, web};

use crate::{
    config::AppState,
    error::{Error, InternalError},
};

pub fn extract_app_state(req: &ServiceRequest) -> Result<AppState, Error> {
    req.app_data::<web::Data<AppState>>()
        .map(|state| state.get_ref().clone())
        .ok_or_else(|| {
            tracing::error!(
                error.code = "InternalError::MissingAppData",
                "Failed to extract configuration from app_data"
            );
            Error::Internal(InternalError::MissingConfiguration(
                "AppState not registered".into(),
            ))
        })
}
