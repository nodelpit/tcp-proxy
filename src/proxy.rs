use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, warn};

pub async fn handle_connection(stream: TcpStream, peer: SocketAddr) {
    debug!(%peer, "connection accepted");

    drop(stream);

    debug!(%peer, "connection closed");
}

pub async fn accept_loop(listener: TcpListener) {
    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                tokio::spawn(async move{
                    handle_connection(stream, peer).await;
                });
            }

            Err(error) => {
                warn!(%error, "failed to accept connection");
            }
        }
    }
}