pub mod cli;
pub mod proxy;
use anyhow::Result;
use tokio::net::TcpListener;
use tracing::info;

use crate::proxy::accept_loop;

pub async fn run(config: cli::Config) -> Result<()> {
    let listener = TcpListener::bind(config.listener).await?;

    info!(%config.listener, "En écoute..");

    accept_loop(listener).await;

    Ok(())
}
