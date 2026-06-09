use std::net::TcpListener;

use api::startup;
use api::state::AppState;
use api::telemetry::{get_subscriber, init_subscriber};

use shared::config::AppConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // FIXME allow users to send emails, make some callbacks in you SDK
    let app_config = AppConfig::load().expect("Failed to read configuration");

    // let file = std::fs::OpenOptions::new()
    //     .create(true)
    //     .append(true)
    //     .open("/logs/anzar.log")
    //     .expect("Failed to open log file");
    // let sink = std::io::stdout.and(file);
    let sink = std::io::stdout;

    let subscriber = get_subscriber(&app_config.name, "info", sink);
    init_subscriber(subscriber);

    let address = format!("{}:{}", app_config.server.host, app_config.server.port);
    let listener = TcpListener::bind(address)?;

    let app_state = AppState::new(&app_config.config_path).await?;
    let jwt_signer = app_state.clone().startup().await?;
    app_state.crypto.rotate_jwt(jwt_signer);

    let server = startup::run(listener, app_state).await?;

    drop(app_config);
    server.await
}
