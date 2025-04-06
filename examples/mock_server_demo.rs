use std::collections::HashMap;
use tempfile::tempdir;
use tokio::time::Duration;
use zinit_client_rs::{Result, StreamExt, ZinitClient};

// Import the mock server types from the tests module
// Note: In a real application, you would import these from the zinit-client crate
mod mock_server {
    include!("../tests/mock_server.rs");
}
use mock_server::{MockService, MockServiceState, MockServiceTarget, MockZinitServer};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Starting mock Zinit server demo");

    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    println!("Using socket path: {:?}", socket_path);

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Add some mock services
    server.add_service(MockService {
        name: "web-server".to_string(),
        pid: 1001,
        state: MockServiceState::Running,
        target: MockServiceTarget::Up,
        after: HashMap::new(),
    });

    server.add_service(MockService {
        name: "database".to_string(),
        pid: 1002,
        state: MockServiceState::Running,
        target: MockServiceTarget::Up,
        after: HashMap::new(),
    });

    server.add_service(MockService {
        name: "cache".to_string(),
        pid: 0,
        state: MockServiceState::Success,
        target: MockServiceTarget::Down,
        after: HashMap::new(),
    });

    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);

    // List all services
    println!("\nListing all services:");
    let services = client.list().await?;
    for (name, state) in &services {
        println!("  - {}: {:?}", name, state);
    }

    // Get status of a service
    println!("\nGetting status of web-server:");
    let status = client.status("web-server").await?;
    println!("  PID: {}", status.pid);
    println!("  State: {:?}", status.state);
    println!("  Target: {:?}", status.target);

    // Start a service
    println!("\nStarting cache service:");
    match client.start("cache").await {
        Ok(_) => println!("  Service started successfully"),
        Err(e) => println!("  Error starting service: {}", e),
    }

    // Check the new status
    let status = client.status("cache").await?;
    println!("  New state: {:?}", status.state);
    println!("  New PID: {}", status.pid);

    // Stop a service
    println!("\nStopping web-server:");
    match client.stop("web-server").await {
        Ok(_) => println!("  Service stopped successfully"),
        Err(e) => println!("  Error stopping service: {}", e),
    }

    // Check the new status
    let status = client.status("web-server").await?;
    println!("  New state: {:?}", status.state);
    println!("  New PID: {}", status.pid);

    // Try to forget a running service (should fail)
    println!("\nTrying to forget database (running):");
    match client.forget("database").await {
        Ok(_) => println!("  Service forgotten successfully"),
        Err(e) => println!("  Error forgetting service: {}", e),
    }

    // Stop the service first
    client.stop("database").await?;

    // Now try to forget it
    println!("\nTrying to forget database (stopped):");
    match client.forget("database").await {
        Ok(_) => println!("  Service forgotten successfully"),
        Err(e) => println!("  Error forgetting service: {}", e),
    }

    // List services again
    println!("\nListing services after operations:");
    let services = client.list().await?;
    for (name, state) in &services {
        println!("  - {}: {:?}", name, state);
    }

    // Stream logs (mock server returns fixed log entries)
    println!("\nStreaming logs for 2 seconds:");
    match client.logs(true, None::<&str>).await {
        Ok(mut logs) => {
            let timeout = tokio::time::sleep(Duration::from_secs(2));
            tokio::pin!(timeout);

            loop {
                tokio::select! {
                    Some(log_result) = logs.next() => {
                        match log_result {
                            Ok(log) => {
                                println!("[{}] {}: {}",
                                    log.timestamp.format("%H:%M:%S"),
                                    log.service,
                                    log.message);
                            }
                            Err(e) => {
                                println!("Error reading log: {}", e);
                                break;
                            }
                        }
                    }
                    _ = &mut timeout => {
                        println!("Log streaming timeout reached");
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("Error streaming logs: {}", e);
        }
    }

    // Stop the mock server
    println!("\nStopping mock server");
    server.stop().await;

    println!("\nDemo completed successfully");
    Ok(())
}
