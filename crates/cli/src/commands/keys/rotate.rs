use chrono::Utc;
use owo_colors::OwoColorize;

use crate::error::{Error, Result};
use shared::intern::key::KeyService;

pub async fn run(key_service: KeyService) -> Result<()> {
    println!();
    println!("  {}", "Rotating signing key...".dimmed().bold());
    println!();

    let (retired_key, activekey) = key_service.rotate().await.map_err(Error::from)?;
    println!(
        "  {} New key generated             kid: {}  algorithm: {}",
        "✔".green().bold(),
        activekey.kid,
        activekey.algorithm
    );

    let expires = retired_key
        .expires_at
        .map(|dt| {
            if Utc::now() > dt {
                "Expired".to_string()
            } else {
                dt.format("%Y-%m-%d %H:%M").to_string()
            }
        })
        .unwrap_or("─".to_string());
    println!(
        "  {} Old key retired               kid: {}  expires: {}",
        "✓".green().bold(),
        retired_key.kid,
        expires
    );

    println!("  {} JWKS endpoint updated", "✔".green().bold(),);
    println!();
    println!("  Active key : {}", activekey.kid);
    println!("  Retired key: {}", retired_key.kid);
    println!();
    println!(
        "  Tip: run {} to inspect all signing keys.",
        "`anzar keys`".red().bold()
    );

    Ok(())
}
