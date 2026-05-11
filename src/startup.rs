use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer, web};

use tracing_actix_web::TracingLogger;

use crate::application::traits::{
    PermissionServiceTrait, RolePermissionServiceTrait, RoleServiceTrait,
};
use crate::config::AppState;
use crate::scopes::{auth, email, health, permission, role, user};

use crate::http;

pub async fn run(listener: TcpListener, app_state: AppState) -> Result<Server, std::io::Error> {
    // TODO
    // Read RBAC from AnzarConfiguration and save permissions to DB
    if app_state.configuration.auth.rbac.enabled {
        for role in app_state.clone().configuration.auth.rbac.roles {
            let role_id = app_state.upsert_role(&role.name).await?;
            let permission_ids = app_state.upsert_permissions(role.permissions).await?;

            app_state
                .upsert_role_permissions(&role_id, permission_ids)
                .await?;
        }
    }

    let config = app_state.configuration.clone();
    let data = web::Data::new(app_state);

    let http_server = HttpServer::new(move || {
        let app = App::new();

        app.wrap(TracingLogger::default())
            // .wrap(from_fn(ip_rate_limit_middleware))
            .wrap(http::configure_cors(&data.configuration))
            .wrap(http::configure_cookie_session(&data.configuration))
            .wrap(from_fn(http::middlewares::validate_content_type))
            .wrap(http::build_default_headers(&data.configuration))
            .app_data(data.clone())
            .service(http::swagger_service())
            .service(health::health_scope())
            .service(auth::auth_scope())
            .service(
                user::user_scope()
                    .wrap(from_fn(http::middlewares::authorization_middleware))
                    .wrap(from_fn(http::middlewares::auth_middleware)),
            )
            .service(email::email_scope())
            .service(role::role_scope())
            .service(permission::permission_scope())
    });

    let https_cfg = config.server.https;
    if !https_cfg.enabled {
        tracing::warn!("HTTPS disabled — falling back to HTTP");
        let server = http_server.listen(listener)?.run();
        return Ok(server);
    }

    let server = match (&https_cfg.cert_path, &https_cfg.key_path) {
        (Some(cert), Some(key)) => {
            let tls_config = http::configure_tls(cert, key)?;
            tracing::info!("HTTPS enabled");
            http_server.listen_rustls_0_23(listener, tls_config)?.run()
        }
        _ => {
            tracing::warn!("HTTPS enabled but missing certificate or key — falling back to HTTP");
            http_server.listen(listener)?.run()
        }
    };

    Ok(server)
}
