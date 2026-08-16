use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tracing::{debug, warn};

pub async fn accept_loop(listener: TcpListener, target: SocketAddr) {
    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                tokio::spawn(async move {
                    handle_connection(stream, peer, target).await;
                });
            }

            Err(error) => {
                warn!(%error, "failed to accept connection");
            }
        }
    }
}

pub async fn handle_connection(stream: TcpStream, peer: SocketAddr, target: SocketAddr) {
    debug!(%peer, "connection accepted");

    let mut outbound = match timeout(Duration::from_secs(5), TcpStream::connect(target)).await {
        Ok(result) => match result {
            Ok(stream) => stream,

            Err(error) => {
                warn!(%peer, %target, %error, "failed to connect to target");
                return;
            }
        },

        Err(_) => {
            warn!(%peer, %target, "target connection timed out");
            return;
        }
    };

    let mut inbound = stream;

    match copy_bidirectional(&mut outbound, &mut inbound).await {
        Ok((from_target, from_client)) => {
            debug!(%peer, %target, from_target, from_client, "connection closed");
        }
        Err(error) => {
            warn!(%peer, %target, %error, "relay failed");
        }
    }
}
