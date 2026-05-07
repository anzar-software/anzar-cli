use actix_web::{
    HttpResponse, Scope,
    web::{self},
};

use crate::{
    error::{ErrorResponse, Result},
    extractors::{AppStateExtractor, AuthenticatedUser},
    scopes::user::{User, UserRoleServiceTrait},
};

use super::user_role::model::RoleRequest;

#[utoipa::path(
        get,
        path = "/user",
        tag = "Users",
        summary = "Get current User",
        description = "Returns the currently authenticated user's data. Requires a valid Bearer token.",
        security(
            ("session_auth" = []),  // OR
            ("bearer_auth"  = []),
        ),
        responses(
            (status = 200, description = "User Found", body = User),
            (status = UNAUTHORIZED, description = "invalid request", body = ErrorResponse),
        ),
    )]
#[tracing::instrument(name = "Find user", skip(user))]
#[actix_web::get("")]
async fn find_user(user: AuthenticatedUser) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(user.0))
}

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
