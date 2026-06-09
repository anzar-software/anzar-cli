use owo_colors::OwoColorize;
use shared::intern::key::KeyService;

use crate::error::Result;

pub async fn run(key_service: KeyService) -> Result<()> {
    println!();
    println!("  {}", "Pruning expired keys...".dimmed().bold());
    println!();

    let removed_keys = key_service.prune().await?;
    if removed_keys.is_empty() {
        println!("{}", "  No key was removed".dimmed().bold());

        return Ok(());
    }

    for key in removed_keys {
        println!("  {} Key {} removed", "✔".green().bold(), key.kid);
    }
    println!("  {} JWKS endpoint updated", "✔".green().bold());

    Ok(())
}
