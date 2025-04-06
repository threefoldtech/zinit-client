use std::collections::HashMap;
use std::time::Duration;
use tempfile::tempdir;
use zinit_client::{Result, StreamExt, ZinitClient};

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

    println!("Starting log streaming example with mock server");

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

    // List available services
    println!("Available services:");
    let services = client.list().await?;
    for (name, state) in &services {
        println!("  - {}: {:?}", name, state);
    }

    // Ask for a service name to filter logs (optional)
    println!("\nEnter a service name to filter logs (or press Enter for all services):");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    let input = input.trim();

    let filter = if input.is_empty() {
        println!("Streaming logs for all services");
        None
    } else {
        println!("Streaming logs for service: {}", input);
        Some(input)
    };

    // Ask for streaming duration
    println!("\nEnter streaming duration in seconds (default: 5):");
    let mut duration_input = String::new();
    std::io::stdin()
        .read_line(&mut duration_input)
        .expect("Failed to read input");
    let duration_input = duration_input.trim();

    let duration = if duration_input.is_empty() {
        5
    } else {
        duration_input.parse::<u64>().unwrap_or(5)
    };

    println!("\nStreaming logs for {} seconds...", duration);
    println!("Press Ctrl+C to stop");

    // Stream logs
    match client.logs(true, filter).await {
        Ok(mut logs) => {
            // Set up a timeout
            let timeout = tokio::time::sleep(Duration::from_secs(duration));
            tokio::pin!(timeout);

            // Set up a counter for log entries
            let mut count = 0;

            // Stream logs until timeout
            loop {
                tokio::select! {
                    Some(log_result) = logs.next() => {
                        match log_result {
                            Ok(log) => {
                                count += 1;
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
                        println!("\nStreaming timeout reached after {} seconds", duration);
                        break;
                    }
                }
            }

            println!("\nReceived {} log entries", count);
        }
        Err(e) => {
            println!("Error streaming logs: {}", e);
        }
    }

    // Now demonstrate snapshot logs (non-following)
    println!("\nGetting log snapshot (non-following)...");
    match client.logs(false, filter).await {
        Ok(mut logs) => {
            let mut count = 0;

            // Process all logs in the snapshot
            while let Some(log_result) = logs.next().await {
                match log_result {
                    Ok(log) => {
                        count += 1;
                        println!(
                            "[{}] {}: {}",
                            log.timestamp.format("%H:%M:%S"),
                            log.service,
                            log.message
                        );
                    }
                    Err(e) => {
                        println!("Error reading log: {}", e);
                        break;
                    }
                }
            }

            println!("\nReceived {} log entries in snapshot", count);
        }
        Err(e) => {
            println!("Error getting log snapshot: {}", e);
        }
    }

    // Stop the mock server
    println!("\nStopping mock server");
    server.stop().await;

    println!("\nExample completed successfully");
    Ok(())
}
