use actix_web::{HttpResponse, Scope, web};

use crate::error::{ErrorResponse, Result};
use crate::http::extractors::AppStateExtractor;

use super::model::PermissionName;

#[utoipa::path(
    post,
    path = "/permission",
    tag = "Rbac",
    operation_id = "createPermission",
    summary = "Create or update a permission",
    description = "Upserts a permission by action and resource. \
                   If it already exists it is updated, otherwise created. \
                   Requires admin privileges.",
    security(
        ("bearer_auth" = []),
    ),
    security(
        ("session_auth" = []),
    ),
    request_body(
        description = "Permission definition",
        content = PermissionName,
        content_type = "application/json"
    ),
    responses(
        (
            status = OK,
            description = "Permission upserted successfully",
        ),
        (
            status = UNAUTHORIZED,
            description = "Missing or expired credentials",
            body = ErrorResponse,
            content_type = "application/json"
        ),
        (
            status = FORBIDDEN,
            description = "Insufficient privileges",
            body = ErrorResponse,
            content_type = "application/json"
        ),
        (
            status = INTERNAL_SERVER_ERROR,
            description = "Unexpected server error",
            body = ErrorResponse,
            content_type = "application/json"
        ),
    ),
)]
#[actix_web::post("")]
async fn create(
    AppStateExtractor(app_state): AppStateExtractor,
    req: web::Json<PermissionName>,
) -> Result<HttpResponse> {
    let permission = format!("{}:{}", &req.0.resource, &req.0.action);

    app_state
        .rbac_service
        .upsert_permission(&permission)
        .await
        .map(|_| Ok(HttpResponse::Ok().finish()))?
}

pub fn permission_scope() -> Scope {
    web::scope("/permission").service(create)
}
