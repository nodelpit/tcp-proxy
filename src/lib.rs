pub mod cli;
pub mod proxy;

use anyhow::Result;
use tokio::net::TcpListener;
use tokio::signal::ctrl_c;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::info;

use crate::proxy::accept_loop;

pub async fn run(config: cli::Config) -> Result<()> {
    let token = CancellationToken::new();
    let tracker = TaskTracker::new();

    let listener = TcpListener::bind(config.listener).await?;

    info!(listener = %config.listener, "Proxy listening..");

    let shutdown_token = token.clone();

    tokio::spawn(async move {
        let _ = ctrl_c().await;
        info!("Graceful shutdown initiated");
        shutdown_token.cancel();
    });

    accept_loop(listener, config.target, token, tracker.clone()).await;

    info!("Proxy stopped accepting connections");

    tracker.close();
    tracker.wait().await;

    Ok(())
}
