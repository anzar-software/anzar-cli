use actix_web::{HttpResponse, Scope, web};

use crate::application::traits::PermissionServiceTrait;
use crate::error::Result;
use crate::http::extractors::AppStateExtractor;

use super::model::PermissionName;

#[actix_web::post("")]
async fn create(
    AppStateExtractor(app_state): AppStateExtractor,
    req: web::Json<PermissionName>,
) -> Result<HttpResponse> {
    let permission = format!("{}:{}", &req.0.resource, &req.0.action);

    app_state
        .upsert_permission(&permission)
        .await
        .map(|_| Ok(HttpResponse::Ok().finish()))?
}

pub fn permission_scope() -> Scope {
    web::scope("/permission").service(create)
}
