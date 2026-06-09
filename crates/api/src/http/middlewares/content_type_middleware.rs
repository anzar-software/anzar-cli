use actix_web::{
    Error,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http,
    middleware::Next,
    mime,
};
use shared::error::CoreError;

use crate::error::Error as AuthError;

pub async fn validate_content_type(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // pre-processing
    let header = req
        .headers()
        .get("content-type")
        .or_else(|| req.headers().get("Content-Type"))
        .and_then(|v| v.to_str().ok());

    if let Some(content_type) = header {
        if req.path() == "/auth/password/reset" {
            if content_type != mime::APPLICATION_WWW_FORM_URLENCODED {
                return Err(AuthError::Core(CoreError::UnsupportedMediaType(
                    "Only application/x-www-form-urlencoded supported for this endpoint".into(),
                ))
                .into());
            }
        } else if req.method() == http::Method::POST && content_type != mime::APPLICATION_JSON {
            return Err(AuthError::Core(CoreError::UnsupportedMediaType(
                "Only application/json supported".into(),
            ))
            .into());
        }
    }

    next.call(req).await
    // post-processing
}
