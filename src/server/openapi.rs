use utoipa::OpenApi;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa_swagger_ui::SwaggerUi;

use crate::config::Configuration;
use crate::scopes::auth::TokenQuery;
use crate::scopes::{auth, email, user};

struct SecurityAddon;
impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);

        // FIXME use configuraiton.auth.session.name not "id"
        components.add_security_scheme(
            "session_auth",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("id"))),
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
        version = "0.7.15",
        description = "REST API for the Anzar platform. All protected routes require a Bearer token.",
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
        email::verify_email
    ),
    components(
        schemas(TokenQuery),
        schemas(Configuration),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Anzar Software", description = "This is a Swagger integration"),
        (name = "Auth", description = "Authentication & session management — login, register, tokens, password reset"),
        (name = "Users", description = "User lookup and profile management"),
        (name = "Email", description = "Email verification flows")
    ),
    external_docs(
        url = "https://anzar_software.gitlab.io/python-sdk/",
        description = "Full Anzar developer documentation"
    ),
)]
pub struct ApiDoc;

pub fn swagger_service() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi())
}
