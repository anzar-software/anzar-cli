use crate::error::Result;
use crate::shared::configuration::{AuthStrategy, CacheDriver, DatabaseDriver};
use crate::shared::{constants, support};
use std::fs;

use dialoguer::Confirm;
use owo_colors::OwoColorize;

use crate::{
    dialoger::{cache::select_cache, database::select_database, strategy::select_strategy},
    theme::theme,
};

pub fn run(app_name: Option<String>) -> Result<()> {
    if !std::path::Path::new(".git").exists() {
        eprintln!(
            "{} not a git repository. Run {} first.",
            "Error:".red().bold(),
            "`git init`".yellow()
        );
        std::process::exit(1);
    }

    build_anzar();
    build_compose(app_name)?;
    print_post_init_message();

    Ok(())
}

fn build_anzar() {
    if std::path::Path::new("anzar.yml").exists() {
        let confirmation = Confirm::new()
            .with_prompt("anzar.yml already exists. Overwrite?")
            .interact()
            .unwrap();

        if !confirmation {
            return;
        }
    }

    let (db_driver, db_uri) = select_database();
    let (cache_driver, cache_uri) = select_cache();
    let strategy = select_strategy();

    let auth = if matches!(strategy, AuthStrategy::Jwt(..)) {
        constants::JWT_AUTH
    } else {
        constants::SESSION_AUTH
    };

    let config_content = constants::CONFIG_TEMPLATE
        .replace("{{DATABASE_DRIVER}}", &db_driver.to_string())
        .replace("{{DATABASE_URI}}", &db_uri)
        .replace("{{CACHE_DRIVER}}", &cache_driver.to_string())
        .replace("{{CACHE_URI}}", &cache_uri)
        .replace("{{STRATEGY}}", &strategy.to_string())
        .replace("{{AUTH}}", auth);

    println!();
    match fs::write("anzar.yml", config_content) {
        Ok(_) => println!("{} {}", "✓ Created".green().bold(), "anzar.yml".cyan()),
        Err(e) => eprintln!("{} {}", "✗ Failed to create file:".red().bold(), e),
    }
}

fn build_compose(app_name: Option<String>) -> Result<()> {
    if std::path::Path::new("compose.yml").exists() {
        let confirmation = Confirm::new()
            .with_prompt("compose.yml already exists. Overwrite?")
            .interact()
            .unwrap();

        if !confirmation {
            return Ok(());
        }
    }
    let config = support::load_config()?;

    let name: String = match app_name {
        Some(n) => n,
        None => dialoguer::Input::with_theme(&theme())
            .with_prompt("Your app name?")
            .interact_text()
            .unwrap(),
    };
    let database = match config.database.driver {
        DatabaseDriver::MongoDB => constants::MONGO_COMPOSE.replace("{{NAME}}", &name),
        DatabaseDriver::PostgreSQL => constants::POSTGRES_COMPOSE.replace("{{NAME}}", &name),
        DatabaseDriver::SQLite => "".to_string(),
    };
    let cache = match config.database.cache.driver {
        CacheDriver::MemCached => constants::MEMCACHED,
        CacheDriver::Redis => constants::REDIS,
    };

    let db_depends_on = match config.database.driver {
        DatabaseDriver::MongoDB => "\n      db:\n        condition: service_healthy",
        DatabaseDriver::PostgreSQL => "\n      db:\n        condition: service_healthy",
        DatabaseDriver::SQLite => "",
    };
    let volume = match config.database.driver {
        DatabaseDriver::MongoDB => "",
        DatabaseDriver::PostgreSQL => "",
        DatabaseDriver::SQLite => "\n      - ./data/:/app/data",
    };

    let content = constants::COMPOSE
        .replace("{{NAME}}", &name)
        .replace("{{VOLUME}}", volume)
        .replace("{{DB_CONDITION}}", db_depends_on)
        .replace("{{DATABASE}}", &database)
        .replace("{{CACHE}}", cache);

    match fs::write("compose.yml", content) {
        Ok(_) => println!("{} {}", "✓ Created".green().bold(), "compose.yml".cyan()),
        Err(e) => eprintln!("{} {}", "✗ Failed to create file:".red().bold(), e),
    }

    Ok(())
}

fn print_post_init_message() {
    let (os_hint, command) = support::openssl_instruction();

    println!("\n{}", "Action required:".yellow().bold());
    println!(
        "  Generate a secure secret key and add it to {}:",
        "anzar.yml".cyan()
    );

    println!("\n  {}", os_hint.dimmed());
    println!("  {}", command.on_black().white().bold());

    println!("\n  Then set it in {}:", "anzar.yml".cyan());
    println!("  {}", "secret_key: <paste output here>".dimmed());
    println!();
}
