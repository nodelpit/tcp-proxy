use anyhow::Result;
use clap::Parser;
use tcp_proxy::{cli::Config, run};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!(
        listener = %config.listener,
        target = %config.target,
        "proxy starting",
    );

    run(config).await?;

    Ok(())
}
