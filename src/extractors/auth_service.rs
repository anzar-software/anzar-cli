use std::future::{Ready, ready};

use actix_web::{FromRequest, HttpRequest, dev::Payload, web::Data};

use crate::{
    config::AppState,
    error::{Error, InternalError},
};

pub struct AppStateExtractor(pub AppState);

impl FromRequest for AppStateExtractor {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let result = req
            .app_data::<Data<AppState>>()
            .map(|state| state.get_ref())
            .map(|sm| AppStateExtractor(sm.clone()))
            .ok_or(Error::Internal(InternalError::MissingConfiguration(
                "AppState not registered".into(),
            )));

        ready(result)
    }
}
