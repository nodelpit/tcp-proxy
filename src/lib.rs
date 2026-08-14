pub mod cli;
pub mod proxy;
use anyhow::Result;
use tokio::net::TcpListener;
use tracing::info;

use crate::proxy::accept_loop;

pub async fn run(config: cli::Config) -> Result<()> {
    let listener = TcpListener::bind(config.listener).await?;

    info!(listener = %config.listener, "Proxy listening..");

    accept_loop(listener, config.target).await;

    Ok(())
}
