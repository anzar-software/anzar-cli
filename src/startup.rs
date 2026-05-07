use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer, web};

use tracing_actix_web::TracingLogger;

use crate::config::AppState;
use crate::middlewares::{auth_middleware, authorization_middleware, validate_content_type};
use crate::scopes::{auth, email, health, role, user};
use crate::server;

pub async fn run(listener: TcpListener, app_state: AppState) -> Result<Server, std::io::Error> {
    let config = app_state.configuration.clone();
    let data = web::Data::new(app_state);

    let http_server = HttpServer::new(move || {
        let app = App::new();

        app.wrap(TracingLogger::default())
            // .wrap(from_fn(ip_rate_limit_middleware))
            .wrap(server::configure_cors(&data.configuration))
            .wrap(server::configure_cookie_session(&data.configuration))
            .wrap(from_fn(validate_content_type))
            .wrap(server::build_default_headers(&data.configuration))
            .app_data(data.clone())
            .service(server::swagger_service())
            .service(health::health_scope())
            .service(auth::auth_scope())
            .service(
                user::user_scope()
                    .wrap(from_fn(authorization_middleware))
                    .wrap(from_fn(auth_middleware)),
            )
            .service(email::email_scope())
            .service(role::role_scope())
    });

    let https_cfg = config.server.https;
    if !https_cfg.enabled {
        tracing::warn!("HTTPS disabled — falling back to HTTP");
        let server = http_server.listen(listener)?.run();
        return Ok(server);
    }

    let server = match (&https_cfg.cert_path, &https_cfg.key_path) {
        (Some(cert), Some(key)) => {
            let tls_config = server::configure_tls(cert, key)?;
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
