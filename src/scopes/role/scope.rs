use actix_web::{HttpResponse, Scope, web};

use crate::application::traits::RoleServiceTrait;
use crate::error::Result;
use crate::http::extractors::AppStateExtractor;

use super::model::RoleName;

#[actix_web::post("")]
async fn create(
    AppStateExtractor(app_state): AppStateExtractor,
    req: web::Json<RoleName>,
) -> Result<HttpResponse> {
    let name = req.into_inner().name;

    app_state
        .insert_role(&name)
        .await
        .map(|_| Ok(HttpResponse::Ok().finish()))?
}

pub fn role_scope() -> Scope {
    web::scope("/role").service(create)
}
