use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

use crate::error::ErrorResponse;
use shared::application::model::TokenQuery;
use shared::config::AnzarConfiguration;
use shared::domain::model::{ExpiringLink, LoginRequest, ResetPasswordRequest, Session, User};

use crate::scopes::auth::{AuthResponse, EmailRequest, RefreshTokenRequest, SessionTokens};
use crate::scopes::{auth, email, permission, role, user};
use crate::scopes::{
    permission::PermissionName,
    role::{PermissionId, RoleName},
    user::RoleRequest,
};

pub struct SecurityAddon {
    pub cookie_name: String,
}
impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);

        components.add_security_scheme(
            "session_auth",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(&self.cookie_name))),
        );

        // JWT: Bearer token in Authorization header
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT") // optional, just for docs
                    .build(),
            ),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Anzar Software API",
        version = env!("CARGO_PKG_VERSION"),
        description = "REST API for the Anzar platform. \
                       Protected routes require either a Bearer token (`bearer_auth`) \
                       or a valid session cookie (`session_auth`).",
        contact(name = "Anzar Team", email = "dev@anzar.io"),
        license(name = "GPL-3.0", identifier="GPL"),
    ),
    paths(
        auth::login,
        auth::register,
        auth::get_session,
        auth::refresh_token,
        auth::logout,
        auth::request_password_reset,
        auth::render_reset_form,
        auth::submit_new_password,

        user::find_user,
        user::assign_role,

        email::verify_email,

        permission::create,

        role::create,
        role::assign_permission,
    ),
    components(
        schemas(
            AnzarConfiguration,
            TokenQuery,
            RoleRequest,
            User,
            ErrorResponse,
            PermissionName,
            PermissionId,
            RoleName,
            LoginRequest,
            AuthResponse,
            SessionTokens,
            ExpiringLink,
            RefreshTokenRequest,
            EmailRequest,
            ResetPasswordRequest,
            Session,
        ),
    ),
    // modifiers(&SecurityAddon { cookie_name: String::new() }), 
    tags(
        (name = "Auth", description = "Authentication & session management — login, register, tokens, password reset"),
        (name = "Users", description = "User lookup and profile management"),
        (name = "Email", description = "Email verification flows"),
        (name = "Rbac", description = "Role Based Access Control - roles and permissions")
    ),
    external_docs(
        url = "https://anzar_software.gitlab.io/docs/",
        description = "Full Anzar developer documentation"
    ),
)]
pub struct ApiDoc;

pub fn swagger_service(configuration: &AnzarConfiguration) -> SwaggerUi {
    let mut openapi = ApiDoc::openapi();

    let cookie_name = match configuration.clone().auth.strategy {
        shared::config::AuthStrategy::Session(session_config) => session_config.name,
        shared::config::AuthStrategy::Jwt(_) => String::new(),
    };

    SecurityAddon { cookie_name }.modify(&mut openapi);
    SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi)
}
