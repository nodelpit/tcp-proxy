use std::net::SocketAddr;

use tcp_proxy::proxy::accept_loop;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

#[tokio::test]
async fn accepts_connection() {
    let target_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_addr: SocketAddr = target_listener.local_addr().unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();

    let token = CancellationToken::new();
    let tracker = TaskTracker::new();

    tokio::spawn(accept_loop(
        listener,
        target_addr,
        token.clone(),
        tracker.clone(),
    ));

    TcpStream::connect(addr).await.unwrap();

    token.cancel();
    tracker.close();
}

#[tokio::test]
async fn accepts_several_connections() {
    let target_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_addr: SocketAddr = target_listener.local_addr().unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();

    let token = CancellationToken::new();
    let tracker = TaskTracker::new();

    tokio::spawn(accept_loop(
        listener,
        target_addr,
        token.clone(),
        tracker.clone(),
    ));

    for _ in 0..5 {
        TcpStream::connect(addr).await.unwrap();
    }

    token.cancel();
    tracker.close();
}

#[tokio::test]
async fn survives_client_disconnect() {
    let target_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_addr: SocketAddr = target_listener.local_addr().unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();

    let token = CancellationToken::new();
    let tracker = TaskTracker::new();

    tokio::spawn(accept_loop(
        listener,
        target_addr,
        token.clone(),
        tracker.clone(),
    ));

    let client = TcpStream::connect(addr).await.unwrap();

    drop(client);

    TcpStream::connect(addr).await.unwrap();

    token.cancel();
    tracker.close();
}

#[tokio::test]
async fn round_trip() {
    let echo_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let echo_addr = echo_listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (mut stream, _) = echo_listener.accept().await.unwrap();

        let mut buffer = [0; 1024];

        loop {
            let n = stream.read(&mut buffer).await.unwrap();

            if n == 0 {
                break;
            }

            stream.write_all(&buffer[..n]).await.unwrap();
        }
    });

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr: SocketAddr = proxy_listener.local_addr().unwrap();

    let token = CancellationToken::new();
    let tracker = TaskTracker::new();

    tokio::spawn(accept_loop(
        proxy_listener,
        echo_addr,
        token.clone(),
        tracker.clone(),
    ));

    let mut client = TcpStream::connect(proxy_addr).await.unwrap();

    let message = b"hello proxy";

    client.write_all(message).await.unwrap();

    let mut response = vec![0; message.len()];

    client.read_exact(&mut response).await.unwrap();

    assert_eq!(response, message);

    token.cancel();
    tracker.close();
}

#[tokio::test]
async fn proxy_alives_with_large_volume() {
    let echo_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let echo_addr = echo_listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (mut stream, _) = echo_listener.accept().await.unwrap();

        let mut buffer = [0; 1024];

        loop {
            let n = stream.read(&mut buffer).await.unwrap();

            if n == 0 {
                break;
            }

            stream.write_all(&buffer[..n]).await.unwrap();
        }
    });

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr: SocketAddr = proxy_listener.local_addr().unwrap();

    let token = CancellationToken::new();
    let tracker = TaskTracker::new();

    tokio::spawn(accept_loop(
        proxy_listener,
        echo_addr,
        token.clone(),
        tracker.clone(),
    ));

    let mut client = TcpStream::connect(proxy_addr).await.unwrap();

    let payload = vec![42u8; 10_000];

    client.write_all(&payload).await.unwrap();

    let mut response = Vec::new();
    let mut buffer = [0; 1024];

    while response.len() < payload.len() {
        let n = client.read(&mut buffer).await.unwrap();

        if n == 0 {
            break;
        }

        response.extend_from_slice(&buffer[..n]);
    }

    assert_eq!(response, payload);

    token.cancel();
    tracker.close();
}

#[tokio::test]
async fn survives_with_unreachable_target() {
    let target_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_addr: SocketAddr = target_listener.local_addr().unwrap();

    drop(target_listener);

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr: SocketAddr = proxy_listener.local_addr().unwrap();

    let token = CancellationToken::new();
    let tracker = TaskTracker::new();

    tokio::spawn(accept_loop(
        proxy_listener,
        target_addr,
        token.clone(),
        tracker.clone(),
    ));

    let _client = TcpStream::connect(proxy_addr).await.unwrap();
    let _client = TcpStream::connect(proxy_addr).await.unwrap();

    token.cancel();
    tracker.close();
}

#[tokio::test]
async fn preserve_half_close() {
    let target_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_addr: SocketAddr = target_listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (mut target, _) = target_listener.accept().await.unwrap();

        let mut request = Vec::new();
        let mut buffer = [0; 1024];

        loop {
            let n = target.read(&mut buffer).await.unwrap();

            if n == 0 {
                break;
            }

            request.extend_from_slice(&buffer[..n]);
        }

        target.write_all(b"response").await.unwrap();
    });

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr: SocketAddr = proxy_listener.local_addr().unwrap();

    let token = CancellationToken::new();
    let tracker = TaskTracker::new();

    tokio::spawn(accept_loop(
        proxy_listener,
        target_addr,
        token.clone(),
        tracker.clone(),
    ));

    let mut client = TcpStream::connect(proxy_addr).await.unwrap();

    client.write_all(b"response").await.unwrap();
    client.shutdown().await.unwrap();

    let mut response = Vec::new();
    client.read_to_end(&mut response).await.unwrap();

    assert_eq!(response, b"response");

    token.cancel();
    tracker.close();
}

#[tokio::test]
async fn stops_accepting_after_shutdown() {
    let target_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_addr = target_listener.local_addr().unwrap();

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();

    let token = CancellationToken::new();
    let tracker = TaskTracker::new();

    let accept_task = tokio::spawn(accept_loop(
        proxy_listener,
        target_addr,
        token.clone(),
        tracker.clone(),
    ));

    TcpStream::connect(proxy_addr).await.unwrap();

    token.cancel();

    accept_task.await.unwrap();

    assert!(TcpStream::connect(proxy_addr).await.is_err());
}
