use std::collections::HashMap;
use std::net::TcpListener;

use anzar::config::{AppConfig, AppState};
use anzar::error::Result;
use anzar::startup;
use anzar::telemetry::{get_subscriber, init_subscriber};
// use tracing_subscriber::fmt::writer::MakeWriterExt;

use anzar::application::traits::{
    PermissionServiceTrait, RolePermissionServiceTrait, RoleServiceTrait, SigningKeysServiceTrait,
};
use anzar::utils::crypto::Crypto;

async fn sync_rbac(app_state: &AppState) -> Result<()> {
    if app_state.configuration.auth.rbac.enabled {
        let rbac = &app_state.configuration.auth.rbac;

        if !rbac.enabled {
            return Ok(());
        }

        let mut hashmap: HashMap<String, Vec<String>> = rbac
            .roles
            .iter()
            .map(|role| (role.name.clone(), role.permissions.clone()))
            .collect();

        for role in &rbac.roles {
            for role_name in &role.inherits {
                let inherited_permissions = hashmap.get(role_name).cloned().unwrap_or_default();

                let entry = hashmap.entry(role.name.clone()).or_default();
                for perm in inherited_permissions {
                    if !entry.contains(&perm) {
                        entry.push(perm);
                    }
                }
            }
        }

        for role in &rbac.roles {
            let permissions = hashmap.get(&role.name).cloned().unwrap_or_default();

            let role_id = app_state.upsert_role(&role.name).await?;
            let permission_ids = app_state.upsert_permissions(permissions.clone()).await?;

            app_state
                .upsert_role_permissions(&role_id, permission_ids)
                .await?;
        }
    }

    Ok(())
}

async fn sync_signing_keys(app_state: &AppState) -> Result<Crypto> {
    let crypto = if let Ok(jwt_config) = app_state.configuration.auth.jwt() {
        let (private_key, signing_key) = match app_state.load_active_key().await {
            Ok(response) => response,
            Err(_) => {
                let (private, public) = app_state.crypto.openssl.gen_prv_pub_key();
                app_state.insert_signing_keys(&private, &public).await?;

                let (_, key) = app_state.load_active_key().await?;
                (private, key)
            }
        };

        Crypto::from_configuration(&app_state.configuration).with_jwt(
            &private_key,
            &signing_key,
            jwt_config,
        )
    } else {
        Crypto::from_configuration(&app_state.configuration)
    };

    Ok(crypto)
}

async fn startup(app_state: &AppState) -> Result<Crypto> {
    let _ = sync_rbac(app_state).await;

    let crypto = sync_signing_keys(app_state).await?;
    Ok(crypto)
}

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

    let mut app_state = AppState::production(&app_config).await?;

    let crypto = startup(&app_state).await?;
    app_state.crypto = crypto;

    let server = startup::run(listener, app_state).await?;

    drop(app_config);
    server.await
}
