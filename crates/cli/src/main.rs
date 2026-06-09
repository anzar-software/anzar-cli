use clap::{Parser, Subcommand};
use error::Result;

mod commands;
mod dialoger;
mod error;
mod shared;
mod startup;
mod theme;

#[derive(Parser)]
#[command(name = "anzar")]
#[command(about = "Anzar is a lightweight authentication and authorization framework that runs as a separate microservice", long_about = None)]
#[command(author, version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize Anzar for your project", long_about = None)]
    Init {
        #[arg(short, long)]
        name: Option<String>,
    },

    #[command(about = "Check current configuration and setup", long_about = None)]
    Check {
        #[arg(short, long)]
        verbose: bool,
    },

    #[command(about = "Show Anzar service status", long_about = None)]
    Status {},

    #[command(about = "Generate database schemas", long_about = None, visible_alias = "gen")]
    Generate {},

    #[command(about = "Apply database migrations", long_about = None)]
    Migrate {
        #[arg(short, long)]
        path: Option<String>,
    },

    #[command(about = "Manage API keys", long_about = None)]
    Keys {
        #[command(subcommand)]
        command: Option<KeysCommands>,
    },
}

#[derive(Subcommand)]
enum KeysCommands {
    #[command(about = "Rotate the current key, generating a new one", long_about = None)]
    Rotate,
    #[command(about = "Remove all keys past their expiry date", long_about = None)]
    Prune,

    #[command(about = "Revoke a key", long_about = None)]
    Revoke {
        /// Revoke a specific key by ID
        #[arg(long)]
        id: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => commands::init::run(name),
        Commands::Check { verbose } => commands::check::run(verbose),
        Commands::Status {} => commands::status::run(),
        Commands::Generate {} => commands::generate::run(),
        Commands::Migrate { path } => commands::migrate::run(path).await,
        Commands::Keys { command } => {
            let key_service = startup::start().await;
            match command {
                Some(KeysCommands::Rotate) => commands::keys::rotate::run(key_service).await,
                Some(KeysCommands::Prune) => commands::keys::prune::run(key_service).await,
                Some(KeysCommands::Revoke { id }) => {
                    commands::keys::revoke::run(key_service, id).await
                }
                None => commands::keys::list::run(key_service).await,
            }
        }
    }
}
