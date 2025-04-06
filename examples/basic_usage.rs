use std::collections::HashMap;
use tempfile::tempdir;
use tokio::time::Duration;
use zinit_client_rs::{Result, ServiceState, StreamExt, ZinitClient};

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

    println!("Starting basic usage example with mock server");

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

    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);

    // List all services
    println!("Listing all services:");
    let services = client.list().await?;
    for (name, state) in &services {
        println!("  - {}: {:?}", name, state);
    }

    // If there are any services, get detailed status of the first one
    if let Some((name, _)) = services.iter().next() {
        println!("\nGetting detailed status for service '{}':", name);
        match client.status(name).await {
            Ok(status) => {
                println!("  Name: {}", status.name);
                println!("  PID: {}", status.pid);
                println!("  State: {:?}", status.state);
                println!("  Target: {:?}", status.target);
                println!("  Dependencies:");
                for (dep, state) in &status.after {
                    println!("    - {}: {}", dep, state);
                }
            }
            Err(e) => {
                println!("  Error getting status: {}", e);
            }
        }

        // Try to restart the service if it's running
        if let Some(state) = services.get(name) {
            if *state == ServiceState::Running {
                println!("\nRestarting service '{}'...", name);
                // First stop the service
                match client.stop(name).await {
                    Ok(_) => println!("  Service stopped successfully"),
                    Err(e) => println!("  Error stopping service: {}", e),
                }

                // Wait a moment for the service to stop
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Then start it again
                match client.start(name).await {
                    Ok(_) => println!("  Service started successfully"),
                    Err(e) => println!("  Error starting service: {}", e),
                }

                // Get the new status
                match client.status(name).await {
                    Ok(status) => {
                        println!("  New state: {:?}", status.state);
                    }
                    Err(e) => {
                        println!("  Error getting status: {}", e);
                    }
                }
            }
        }
    } else {
        println!("\nNo services found");
    }

    // Stream logs for a short time
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

    println!("\nExample completed successfully");
    Ok(())
}
