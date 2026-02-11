use anyhow::Result;
use netdout::{api, cli, config::AppConfig, db::ResumeStore, download::DownloadEngine};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = cli::Cli::parse_args();
    let config = AppConfig::default();
    let db = Arc::new(ResumeStore::new(&config.database_path)?);
    let engine = Arc::new(DownloadEngine::new(config.clone(), db));

    match args.command {
        cli::Command::Daemon => {
            api::serve(config.api_bind_addr, engine).await?;
        }
        cli::Command::Download { url, output } => {
            let id = engine.enqueue(url, output).await?;
            println!("Queued download: {id}");
        }
        cli::Command::Status { id } => {
            let status = engine.status(&id).await;
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
    }

    Ok(())
}
