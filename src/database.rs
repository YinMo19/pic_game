use std::error::Error;

use rocket_db_pools::sqlx::{self, migrate::MigrateDatabase};
use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions, Sqlite};
use std::path::Path;

use colored::*;

const CORE_DB_URL: &str = "sqlite://database/core.db";

/// Initialize the database.
pub async fn init_check_database_all() {
    let _ = create_database(CORE_DB_URL).await;
    let _ = check_database(CORE_DB_URL).await;
}

/// check if database exists, create if not.
async fn create_database(database_name: &str) -> Result<(), Box<dyn Error>> {
    if !Sqlite::database_exists(database_name)
        .await
        .unwrap_or(false)
    {
        println!(
            "{} {}",
            "Creating database".green().bold(),
            database_name.blue()
        );
        match Sqlite::create_database(database_name).await {
            Ok(_) => println!(
                "{} {} {}",
                "Create db:".green().bold(),
                database_name.blue(),
                "success".green().bold()
            ),
            Err(error) => panic!("error: {}", error),
        }
    }
    Ok(())
}

/// Read the directory /database/migrations/<path>/<<timestamp>-<name>.sql>
/// and execute the sql file to migrate.
async fn check_database(database_name: &str) -> Result<(), Box<dyn Error>> {
    let m = Migrator::new(Path::new("./database/migrations")).await?;
    let pool = SqlitePoolOptions::new().connect(database_name).await?;
    let _ = m.run(&pool).await;
    println!(
        "{} {} {}",
        "Migrate".green().bold(),
        database_name.blue(),
        "successfully.".green().bold()
    );

    Ok(())
}
