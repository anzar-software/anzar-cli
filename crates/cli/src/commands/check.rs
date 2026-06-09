use owo_colors::OwoColorize;
use shared::error::{CoreError, InternalError};
use std::path::Path;

use crate::error::Result;
use crate::shared::support;
use shared::config::AnzarConfiguration;

pub fn run(verbose: bool) -> Result<()> {
    println!("{}", "Running checks...".dimmed());

    //1. Check existence of git
    let exists = Path::new(".git").exists();
    println!();
    println!("  Git");
    support::print_result("Repository", exists, None);

    //2. Validate anzar.yml configuration
    validate_anzar(verbose);
    Ok(())
}

fn validate_anzar(verbose: bool) {
    let config = match support::load_config() {
        Ok(config) => config,
        Err(e) => {
            support::print_result("Invalid anzar.yml", false, Some(&e.to_string()));
            return;
        }
    };

    check_config_values(verbose, &config);
}

fn check_config_values(verbose: bool, config: &AnzarConfiguration) {
    // Use validate() for structured validation (auth, security, etc.)
    match config.validate() {
        Ok(_) => {
            support::print_result("Config values", true, None);
        }

        Err(CoreError::Internal(InternalError::InvalidConfig(errors))) => {
            // Err(Error::InvalidAppConfig(errors)) => {
            for error in &errors {
                if let CoreError::Internal(InternalError::MissingField { field, reason }) = error {
                    support::print_result(field, false, Some(reason));
                }
            }
        }
        Err(e) => {
            support::print_result("Config values", false, Some(&e.to_string()));
        }
    }

    if verbose {
        println!("    {} {}", "url:".dimmed(), config.app.url.cyan());
        println!(
            "    {} {}",
            "environment:".dimmed(),
            config.app.environment.cyan()
        );
        // println!(
        //     "    {} {}",
        //     "secret_key length:".dimmed(),
        //     config.security.secret_key.len().to_string().cyan()
        // );
    }
}
