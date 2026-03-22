use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer, web};

use tracing_actix_web::TracingLogger;

use crate::config::AppState;
use crate::middlewares::{auth_middleware, authorization_middleware, validate_content_type};
use crate::scopes::{auth, email, health, user};
use crate::server;

pub async fn run(listener: TcpListener, app_state: AppState) -> Result<Server, std::io::Error> {
    // FIXME use Arc to remove these multiple cloning
    let app_state_inner = app_state.clone();
    let config = app_state.configuration.clone();

    let http_server = HttpServer::new(move || {
        // .wrap(TracingLogger::<CustomRootSpanBuilder>::new())
        // .wrap(from_fn(ip_rate_limit_middleware))
        App::new()
            .wrap(TracingLogger::default())
            .wrap(server::configure_cors(&app_state.configuration))
            .wrap(server::configure_session(&app_state.configuration))
            .wrap(from_fn(validate_content_type))
            .wrap(server::build_default_headers(&app_state.configuration))
            .app_data(web::Data::new(app_state_inner.clone()))
            .service(server::swagger_service())
            .service(health::health_scope())
            .service(auth::auth_scope())
            .service(
                user::user_scope()
                    .wrap(from_fn(authorization_middleware))
                    .wrap(from_fn(auth_middleware)),
            )
            .service(email::email_scope())
    });

    let https_cfg = config.server.https;
    if !https_cfg.enabled {
        tracing::warn!("HTTPS disabled — falling back to HTTP");
        let server = http_server.listen(listener)?.run();
        return Ok(server);
    }

    let server = match (&https_cfg.cert_path, &https_cfg.key_path) {
        (Some(cert), Some(key)) => {
            let tls_config = server::configure_tls(cert.into(), key.into())?;
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
