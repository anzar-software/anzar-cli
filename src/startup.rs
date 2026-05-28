use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer, web};

use tracing_actix_web::TracingLogger;

use crate::config::AppState;
use crate::scopes::{auth, email, health, jwks, permission, role, user};

use crate::http;

pub async fn run(listener: TcpListener, app_state: AppState) -> Result<Server, std::io::Error> {
    let config = app_state.configuration.clone();
    let appstate_data = web::Data::new(app_state);

    let http_server = HttpServer::new(move || {
        let app = App::new();

        app.wrap(from_fn(http::middlewares::validate_content_type))
            .wrap(http::build_default_headers(&appstate_data.configuration))
            .wrap(http::configure_cookie_session(&appstate_data.configuration))
            .wrap(http::configure_cors(&appstate_data.configuration))
            .wrap(from_fn(http::middlewares::ip_rate_limit_middleware))
            .wrap(TracingLogger::default())
            .app_data(appstate_data.clone())
            .service(http::swagger_service(&appstate_data.configuration))
            .service(health::health_scope())
            .service(jwks::jwks_scope())
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
