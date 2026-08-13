use tcp_proxy::proxy::accept_loop;
use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;

#[tokio::test]
async fn accepts_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(accept_loop(listener));

    TcpStream::connect(addr).await.unwrap();
}

#[tokio::test]
async fn accepts_several_connections() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(accept_loop(listener));

    for _ in 0..5 {
        TcpStream::connect(addr).await.unwrap();
    }
}

#[tokio::test]
async fn survives_client_disconnect() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(accept_loop(listener));

    let client = TcpStream::connect(addr).await.unwrap();

    drop(client);

    TcpStream::connect(addr).await.unwrap();
}