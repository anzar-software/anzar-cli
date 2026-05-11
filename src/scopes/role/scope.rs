use actix_web::{HttpResponse, Scope, web};

use crate::application::traits::{RolePermissionServiceTrait, RoleServiceTrait};
use crate::error::Result;
use crate::http::extractors::AppStateExtractor;

use super::model::{PermissionId, RoleName};

#[actix_web::post("")]
async fn create(
    AppStateExtractor(app_state): AppStateExtractor,
    req: web::Json<RoleName>,
) -> Result<HttpResponse> {
    let name = req.into_inner().name;

    app_state
        .upsert_role(&name)
        .await
        .map(|_| Ok(HttpResponse::Ok().finish()))?
}

#[actix_web::post("/{id}/permission")]
async fn assign_permission(
    AppStateExtractor(app_state): AppStateExtractor,
    req: web::Json<PermissionId>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let role_id = path.into_inner();
    let permission_id = req.into_inner().permission_id;

    app_state
        .insert_role_permission(&role_id, &permission_id)
        .await
        .map(|_| Ok(HttpResponse::Ok().finish()))?
}

pub fn role_scope() -> Scope {
    web::scope("/role")
        .service(create)
        .service(assign_permission)
}
