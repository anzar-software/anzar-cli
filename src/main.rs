#![warn(unused_imports)]

use std::net::TcpListener;

use anzar::config::{AppConfig, AppState};
use anzar::startup;
use anzar::telemetry::{get_subscriber, init_subscriber};
use tracing_subscriber::fmt::writer::MakeWriterExt;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // FIXME allow users to send emails, make some callbacks in you SDK
    let app_config = AppConfig::load().expect("Failed to read configuration");

    let dummy = bcrypt::hash("dummy_password", 12).unwrap();
    println!("{}", dummy);

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/anzar.log")
        .expect("Failed to open log file");

    let sink = std::io::stdout.and(file);
    let subscriber = get_subscriber(&app_config.name, "info", sink);
    init_subscriber(subscriber);

    let address = format!("{}:{}", app_config.server.host, app_config.server.port);
    let listener = TcpListener::bind(address)?;

    let app_state = AppState::production(&app_config).await?;
    let server = startup::run(listener, app_state).await?;

    drop(app_config);
    server.await
}
