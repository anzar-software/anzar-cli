use dialoguer::Confirm;
use owo_colors::OwoColorize;
use shared::{domain::model::SigningKey, intern::key::KeyService};

use crate::error::Result;

pub async fn run(key_service: KeyService, kid: Option<String>) -> Result<()> {
    println!();

    if let Some(id) = kid {
        println!(
            "  {} Hard revoking key {}. All tokens signed with this key will be immediately rejected. This cannot be undone.",
            "⚠".yellow().bold(),
            id,
        );
        println!();

        let confirmation = Confirm::new().with_prompt("Confirm?").interact().unwrap();
        if !confirmation {
            return Ok(());
        }

        let keys = key_service.list_keys().await?;
        if keys
            .into_iter()
            .filter(|k| k.kid == id)
            .collect::<Vec<SigningKey>>()
            .is_empty()
        {
            println!("  {} Key {} is invalid.", "✖  Error:".red().bold(), id);
            println!();
            println!(
                "  Tip: run {} to inspect all signing keys.",
                "`anzar keys`".red().bold()
            );

            return Ok(());
        }

        let revoked_key = key_service.revoke(&id).await?;

        println!();
        println!("  {} Key {} revoked", "✔".green().bold(), revoked_key.kid);
        println!("  {} JWKS endpoint updated", "✔".green().bold());
    } else {
        println!(
            "  {} You must specify a key ID to revoke.",
            "✖  Error:".red().bold()
        );
        println!();
        println!("  Usage: {}", "`anzar keys revoke <ID>`".red().bold());
    }

    Ok(())
}
