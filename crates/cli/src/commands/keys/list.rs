use chrono::Utc;
use shared::{domain::model::SigningKey, intern::key::KeyService};

use crate::error::Result;

pub async fn run(key_service: KeyService) -> Result<()> {
    match key_service.list_keys().await {
        Ok(keys) => print_keys(keys),
        Err(_) => println!("no keys found"),
    }

    Ok(())
}

fn print_keys(keys: Vec<SigningKey>) {
    // Column headers
    println!(" ");
    println!(
        "  {:<45} {:<11} {:<10} {:<18} {:<18}",
        "ID", "Algorithm", "Status", "Created", "Expires"
    );
    println!("  {}", "─".repeat(100));

    // Data rows
    for key in keys {
        let created = key.created_at.format("%Y-%m-%d %H:%M");
        let expires = key.expires_at.map(|dt| {
            if Utc::now() > dt {
                "Expired".to_string()
            } else {
                dt.format("%Y-%m-%d %H:%M").to_string()
            }
        });

        println!(
            "  {:<45} {:<11} {:<10} {:<18} {:<18}",
            key.kid,
            key.algorithm,
            key.status,
            created,
            expires.unwrap_or("─".to_string()),
        );
    }
}
