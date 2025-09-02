use std::{net::SocketAddr, time::Duration};

use tokio::time::sleep;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use cl0_node::{control_plane::new_service_instance, node_client::NodeClient};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_node_registration_and_heartbeat() -> Result<(), Box<dyn std::error::Error>> {
    let (service, shared_state) = new_service_instance();

    // Dynamically pick a port
    let addr: SocketAddr = "[::1]:0".parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;

    // Start the server in background
    tokio::spawn(async move {
        Server::builder()
            .add_service(service)
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    // Create and register client
    let mut client = NodeClient::new(&format!("http://{}", local_addr), "test-version").await?;
    client.register().await?;

    // Start a heartbeat and wait for a few ticks
    let mut hb_client = client;
    tokio::spawn(async move {
        hb_client.send_heartbeat_loop().await.unwrap();
    });

    // Wait for heartbeats to propagate
    sleep(Duration::from_secs(6)).await;

    // Check the shared control plane state
    let state = shared_state.read().await;
    let nodes: Vec<_> = state.nodes.iter().collect();
    assert_eq!(nodes.len(), 1, "There should be one registered node");

    let (_id, node) = nodes[0];
    assert_eq!(node.version, "test-version");
    assert!(node.last_heartbeat > 0);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_multi_node_registration_and_heartbeat() -> Result<(), Box<dyn std::error::Error>> {
    let (service, shared_state) = new_service_instance();

    // Dynamically pick a port
    let addr: SocketAddr = "[::1]:0".parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;

    // Start the server in background
    tokio::spawn(async move {
        Server::builder()
            .add_service(service)
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    // Create and register first client
    let mut client1 = NodeClient::new(&format!("http://{}", local_addr), "test-version-1").await?;
    client1.register().await?;

    // Create and register second client
    let mut client2 = NodeClient::new(&format!("http://{}", local_addr), "test-version-2").await?;
    client2.register().await?;

    // Start a heartbeat and wait for a few ticks
    let mut hb_client1 = client1;
    tokio::spawn(async move {
        hb_client1.send_heartbeat_loop().await.unwrap();
    });

    // Start a heartbeat and wait for a few ticks
    let mut hb_client2 = client2;
    tokio::spawn(async move {
        hb_client2.send_heartbeat_loop().await.unwrap();
    });

    // Wait for heartbeats to propagate
    sleep(Duration::from_secs(6)).await;

    // Check the shared control plane state
    let state = shared_state.read().await;
    let nodes: Vec<_> = state.nodes.iter().collect();
    assert_eq!(nodes.len(), 2, "There should be two registered nodes");

    let (_id1, node1) = nodes[0];
    assert_eq!(node1.version, "test-version-1");
    assert!(node1.last_heartbeat > 0);

    let (_id2, node2) = nodes[1];
    assert_eq!(node2.version, "test-version-2");
    assert!(node2.last_heartbeat > 0);

    Ok(())
}
