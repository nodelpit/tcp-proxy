use std::net::SocketAddr;
use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};
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

    let mut outbound = match TcpStream::connect(target).await {
        Ok(stream) => stream,

        Err(error) => {
            warn!(%error, "failed to connect to target");
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
