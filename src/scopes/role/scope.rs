use actix_web::{HttpResponse, Scope, web};

use crate::application::traits::{RolePermissionServiceTrait, RoleServiceTrait};
use crate::error::{ErrorResponse, Result};
use crate::http::extractors::AppStateExtractor;

use super::model::{PermissionId, RoleName};

#[utoipa::path(
    post,
    path = "/role",
    tag = "Rbac",
    operation_id = "createRole",
    summary = "Create or update a role",
    description = "Upserts a role by name. \
                   If it already exists it is updated, otherwise created. \
                   Requires admin privileges.",
    security(
        ("bearer_auth" = []),
    ),
    security(
        ("session_auth" = []),
    ),
    request_body(
        description = "Role definition",
        content = RoleName,
        content_type = "application/json"
    ),
    responses(
        (
            status = OK,
            description = "Role upserted successfully",
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
#[tracing::instrument(name = "Create role", skip(app_state, req))]
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

#[utoipa::path(
    post,
    path = "/role/{id}/permission",
    tag = "Rbac",
    operation_id = "assignPermissionToRole",
    summary = "Assign a permission to a role",
    description = "Assigns an existing permission to the specified role. \
                   Requires admin privileges.",
    security(
        ("bearer_auth" = []),
    ),
    security(
        ("session_auth" = []),
    ),
    params(
        ("id" = String, Path, description = "Role ID to assign the permission to"),
    ),
    request_body(
        description = "Permission to assign",
        content = PermissionId,
        content_type = "application/json"
    ),
    responses(
        (
            status = OK,
            description = "Permission successfully assigned to role",
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
            status = NOT_FOUND,
            description = "Role or permission not found",
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
#[tracing::instrument(name = "Assign permission to role", skip(app_state, req))]
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
