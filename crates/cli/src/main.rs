//! `rusty-automation-tool` CLI entry-point.
//!
//! Available sub-commands:
//! - `serve`    — start the API server.
//! - `worker`   — start a queue worker.
//! - `migrate`  — run pending database migrations.
//! - `validate` — validate a workflow JSON file.

use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Parser)]
#[command(
    name = "rusty-automation-tool",
    about = "High-performance workflow automation engine",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the REST API server.
    Serve {
        #[arg(long, default_value = "0.0.0.0:8080")]
        bind: String,
    },
    /// Start a background worker that processes queued jobs.
    Worker,
    /// Run pending database migrations.
    Migrate {
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
    },
    /// Validate a workflow definition JSON file.
    Validate {
        /// Path to the workflow JSON file.
        path: std::path::PathBuf,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Command::Serve { bind } => {
            info!("Starting API server on {bind}");
            let database_url = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/rusty_automation".to_string());
            let pool = db::pool::create_pool(&database_url, 10)
                .await
                .expect("failed to connect to database");
            api::serve(&bind, pool).await.unwrap();
        }
        Command::Worker => {
            info!("Starting background worker");
            // TODO: wire up engine::worker::run().await
            todo!("Worker not yet implemented");
        }
        Command::Migrate { database_url } => {
            info!("Running migrations against {database_url}");
            let pool = db::pool::create_pool(&database_url, 2)
                .await
                .expect("failed to connect to database");
            db::pool::run_migrations(&pool)
                .await
                .expect("migration failed");
            info!("Migrations applied successfully");
        }
        Command::Validate { path } => {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read file {}: {e}", path.display()));

            let workflow: engine::Workflow = serde_json::from_str(&content)
                .unwrap_or_else(|e| panic!("invalid JSON: {e}"));

            match engine::validate_dag(&workflow) {
                Ok(order) => {
                    println!("✅ Workflow is valid. Execution order: {order:?}");
                }
                Err(e) => {
                    eprintln!("❌ Validation failed: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
