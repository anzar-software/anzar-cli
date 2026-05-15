use actix_web::{
    HttpResponse, Scope,
    web::{self},
};

use crate::error::{ErrorResponse, Result};
use crate::http::extractors::{AppStateExtractor, AuthenticatedUser};

use super::model::RoleRequest;
use crate::application::model::User;
use crate::application::traits::UserRoleServiceTrait;

#[utoipa::path(
        get,
        path = "/user",
        tag = "Users",
        summary = "Get current User",
        description = "Returns the currently authenticated user's profile. \
        Requires either a valid Bearer token or an active session cookie.",
        security(
            ("bearer_auth" = []),
        ),
        security(
            ("session_auth" = []),
        ),
        responses(
            (
                status = OK,
                description = "Authenticated user's profile returned successfully",
                body = User,
                content_type = "application/json"
            ),
            (
                status = UNAUTHORIZED,
                description = "Missing or expired credentials",
                body = ErrorResponse,
                content_type = "application/json"
            ),
            (
                status = NOT_FOUND,
                description = "Authenticated token is valid but user no longer exists",
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
#[tracing::instrument(name = "Find user", skip(user))]
#[actix_web::get("")]
async fn find_user(user: AuthenticatedUser) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(user.0))
}

#[utoipa::path(
        post,
        path = "/user/role",
        tag = "Rbac",
        summary = "Assign a role to user",
        description = "Assigns the specified role to a target user. \
                   Requires an authenticated session with sufficient privileges (admin only).",
        security(
            ("bearer_auth" = []),
        ),
        security(
            ("session_auth" = []),
        ),
        request_body(
            description = "Role assignment payload",
            content = RoleRequest,
            content_type = "application/json"
        ),
        responses(
            (
                status = OK,
                description = "Role successfully assigned — returns updated user profile",
                body = User,
                content_type = "application/json"
            ),
            (
                status = UNAUTHORIZED,
                description = "Missing or expired credentials",
                body = ErrorResponse,
                content_type = "application/json"
            ),
            (
                status = FORBIDDEN,
                description = "Authenticated user does not have permission to assign roles",
                body = ErrorResponse,
                content_type = "application/json"
            ),
            (
                status = NOT_FOUND,
                description = "Target user not found",
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
#[tracing::instrument(name = "Assign role", skip(app_state, req, user))]
#[actix_web::post("/role")]
async fn assign_role(
    AppStateExtractor(app_state): AppStateExtractor,
    req: web::Json<RoleRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let user_id = &user.0.id()?;
    let role_name = req.into_inner().role;

    app_state
        .insert_user_role(user_id, &role_name)
        .await
        .map(|_| Ok(HttpResponse::Ok().finish()))?
}

#[actix_web::delete("/{id}/role")]
async fn remove_role() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::get("/{id}/role/{role}")]
async fn get_roles() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

pub fn user_scope() -> Scope {
    web::scope("/user")
        .service(find_user)
        .service(assign_role)
        .service(remove_role)
        .service(get_roles)
}
