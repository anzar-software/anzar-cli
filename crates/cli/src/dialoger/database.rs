use dialoguer::{Input, Password, Select};
use owo_colors::OwoColorize;
use std::fs::OpenOptions;
use std::io::Write;

use crate::shared::support;
use crate::theme::theme;

use shared::config::database::DatabaseDriver;

pub fn select_database() -> (DatabaseDriver, String) {
    let databases: Vec<DatabaseDriver> = vec![
        DatabaseDriver::MongoDB,
        DatabaseDriver::PostgreSQL,
        DatabaseDriver::SQLite,
    ];
    let choice = Select::with_theme(&theme())
        .with_prompt("Select database")
        .items(&databases)
        .default(0)
        .interact()
        .unwrap();
    let driver = databases[choice];

    if driver == DatabaseDriver::SQLite {
        let db_path: String = Input::with_theme(&theme())
            .with_prompt("  › file path")
            .default("data/data.db".to_string())
            .show_default(true)
            .validate_with(|input: &String| -> Result<(), &str> {
                let path = std::path::Path::new(input);

                // must have a parent folder (not just a filename)
                if path
                    .parent()
                    .map(|p| p.as_os_str().is_empty())
                    .unwrap_or(true)
                {
                    return Err("must include a folder, e.g. data/data.db");
                }

                // must have .db extension
                match path.extension().and_then(|e| e.to_str()) {
                    Some("db") => Ok(()),
                    _ => Err("file must have a .db extension, e.g. data/anzar.db"),
                }
            })
            .interact_text()
            .unwrap();

        println!(
            "\n  {} {}\n  {} {}\n",
            "⚠️".yellow(),
            "Important:".bold().yellow(),
            "→".yellow(),
            format!(
                "Make sure `{}` exists before starting the container",
                db_path.cyan()
            )
            .bold()
            .white()
        );

        write_env_file(&driver, "", "", &db_path);
        return (driver, format!("/app/{db_path}"));
    }

    let db_user: String = Input::with_theme(&theme())
        .with_prompt("  › Username")
        .interact_text()
        .unwrap();

    let db_password: String = Password::with_theme(&theme())
        .with_prompt("  › Password")
        .with_confirmation("  › Confirm password", "Passwords do not match")
        .interact()
        .unwrap();

    let db_name: String = Input::with_theme(&theme())
        .with_prompt("  › Database name")
        .default("dev".to_string())
        .show_default(true)
        .interact_text()
        .unwrap();

    let uri = match driver {
        DatabaseDriver::MongoDB => {
            format!("mongodb://{db_user}:{db_password}@db:27017/{db_name}?authSource=admin")
        }
        DatabaseDriver::PostgreSQL => {
            format!("postgres://{db_user}:{db_password}@db:5432/{db_name}")
        }
        DatabaseDriver::SQLite => unreachable!(),
    };

    write_env_file(&driver, &db_user, &db_password, &db_name);
    (driver, uri.to_string())
}

fn write_env_file(driver: &DatabaseDriver, user: &str, password: &str, name: &str) {
    let contents = match driver {
        DatabaseDriver::MongoDB => format!(
            r#"
MONGO_INITDB_ROOT_USERNAME={user}
MONGO_INITDB_ROOT_PASSWORD={password}
MONGO_INITDB_DATABASE={name}
"#
        ),
        DatabaseDriver::PostgreSQL => format!(
            r#"
POSTGRES_USER={user}
POSTGRES_PASSWORD={password}
POSTGRES_DB={name}
"#
        ),
        DatabaseDriver::SQLite => format!(
            r#"
DATABASE_PATH={name}
"#
        ),
    };

    let result = OpenOptions::new()
        .create(true) // create if it doesn't exist
        .append(true) // append if it does
        .open(".env")
        .and_then(|mut file| file.write_all(contents.as_bytes()));

    match result {
        Ok(_) => support::print_result(".env file written", true, None),
        Err(e) => support::print_result(".env file written", false, Some(&e.to_string())),
    }
}
