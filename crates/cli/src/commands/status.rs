use crate::error::Result;
use owo_colors::OwoColorize;

pub fn run() -> Result<()> {
    const BASE_URL: &str = "http://localhost:3000";
    println!("{}", "Checking Anzar service...".bold());

    match minreq::get(format!("{}/health_check", BASE_URL)).send() {
        Ok(response) => {
            if response.status_code == 200 {
                println!(
                    "\n  {} {}",
                    "✔".green().bold(),
                    "Service is up and running!".green()
                );
                println!("  {} {}", "→".dimmed(), BASE_URL.dimmed());
            } else {
                println!(
                    "\n  {} {} ({})",
                    "⚠".yellow().bold(),
                    "Service responded but something is off".yellow(),
                    response.status_code
                );
                println!("  {} Try restarting the server", "→".dimmed());
            }
        }
        Err(err) => {
            println!(
                "\n  {} {}",
                "✖".red().bold(),
                "Could not reach Anzar service".red()
            );
            println!("  {} {}", "→".dimmed(), BASE_URL);
            println!("  {} {}", "→".dimmed(), format!("Error: {}", err).dimmed());
            println!(
                "  {} {} {}",
                "→".dimmed(),
                "Make sure the server is running (start it with:".italic(),
                "docker compose up -d)".bold()
            );
        }
    }

    Ok(())
}
