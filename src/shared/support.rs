use owo_colors::OwoColorize;
use std::{fs, path::Path};

use crate::{
    error::{Error, Result},
    shared::configuration::AnzarConfiguration,
};

pub fn load_config() -> Result<AnzarConfiguration> {
    let candidates = ["anzar.yaml", "anzar.yml"];
    let path = candidates.iter().find(|p| Path::new(p).exists());

    let path = match path {
        Some(p) => p,
        None => {
            print_result(
                "anzar.yml not found",
                false,
                Some("run `anzar init` to create one"),
            );
            return Err(Error::FileNotFound {
                path: std::path::PathBuf::from("anzar.yml"),
            });
        }
    };

    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            print_result("Failed to read anzar.yml", false, Some(&e.to_string()));
            return Err(Error::InvalidConfig {
                key: "key".to_string(),
                reason: e.to_string(),
            });
        }
    };

    serde_yaml::from_str::<AnzarConfiguration>(&raw).map_err(Into::into)
}

pub fn print_result(label: &str, passed: bool, hint: Option<&str>) {
    if passed {
        println!("  {} {}", "✓".green().bold(), label);
    } else {
        match hint {
            Some(h) => println!(
                "  {} {} {}",
                "✗".red().bold(),
                label.red(),
                format!("({})", h).dimmed()
            ),
            None => println!("  {} {}", "✗".red().bold(), label.red()),
        }
    }
}

pub fn openssl_instruction() -> (&'static str, &'static str) {
    match std::env::consts::OS {
        "windows" => {
            // Check if openssl is actually available
            let has_openssl = std::process::Command::new("openssl")
                .arg("version")
                .output()
                .is_ok();

            if has_openssl {
                ("Windows (openssl found in PATH):", "$ openssl rand -hex 32")
            } else {
                (
                    "Windows — openssl not found, alternatives:",
                    "# Git Bash / WSL:\n  $ openssl rand -hex 32\n\n  \
             # PowerShell:\n  $ [System.BitConverter]::ToString([System.Security.Cryptography.RandomNumberGenerator]::GetBytes(32)) -replace '-', ''",
                )
            }
        }
        "macos" => (
            "macOS (openssl ships with Homebrew or LibreSSL via Xcode):",
            "$ openssl rand -hex 32",
        ),
        _ => ("Linux:", "$ openssl rand -hex 32"),
    }
}
