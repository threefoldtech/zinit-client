use std::collections::HashMap;
use std::time::Duration;
use tempfile::tempdir;
use zinit_client::{Result, ZinitClient};

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

    println!("Starting service management example with mock server");

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

    // Ask for a service name to manage
    println!("Enter a service name to manage (or press Enter to use the first available service):");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    let input = input.trim();

    // Get the service name
    let service_name = if input.is_empty() {
        // Use the first available service
        let services = client.list().await?;
        if services.is_empty() {
            println!("No services found");
            server.stop().await;
            return Ok(());
        }

        let name = services.keys().next().unwrap().clone();
        println!("Using service: {}", name);
        name
    } else {
        input.to_string()
    };

    // Get the current status
    println!("\nGetting current status for '{}'...", service_name);
    match client.status(&service_name).await {
        Ok(status) => {
            println!("Current status:");
            println!("  PID: {}", status.pid);
            println!("  State: {:?}", status.state);
            println!("  Target: {:?}", status.target);
        }
        Err(e) => {
            println!("Error getting status: {}", e);
            server.stop().await;
            return Ok(());
        }
    }

    // Menu of operations
    loop {
        println!("\nChoose an operation:");
        println!("1. Start service");
        println!("2. Stop service");
        println!("3. Restart service");
        println!("4. Get status");
        println!("5. Send signal (SIGTERM)");
        println!("6. Exit");

        let mut choice = String::new();
        std::io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read input");
        let choice = choice.trim();

        match choice {
            "1" => {
                println!("Starting service '{}'...", service_name);
                match client.start(&service_name).await {
                    Ok(_) => println!("Service started successfully"),
                    Err(e) => println!("Error starting service: {}", e),
                }

                // Wait a moment for the service to start
                tokio::time::sleep(Duration::from_secs(1)).await;

                // Get the new status
                match client.status(&service_name).await {
                    Ok(status) => {
                        println!("New status:");
                        println!("  PID: {}", status.pid);
                        println!("  State: {:?}", status.state);
                        println!("  Target: {:?}", status.target);
                    }
                    Err(e) => {
                        println!("Error getting status: {}", e);
                    }
                }
            }
            "2" => {
                println!("Stopping service '{}'...", service_name);
                match client.stop(&service_name).await {
                    Ok(_) => println!("Service stopped successfully"),
                    Err(e) => println!("Error stopping service: {}", e),
                }

                // Wait a moment for the service to stop
                tokio::time::sleep(Duration::from_secs(1)).await;

                // Get the new status
                match client.status(&service_name).await {
                    Ok(status) => {
                        println!("New status:");
                        println!("  PID: {}", status.pid);
                        println!("  State: {:?}", status.state);
                        println!("  Target: {:?}", status.target);
                    }
                    Err(e) => {
                        println!("Error getting status: {}", e);
                    }
                }
            }
            "3" => {
                println!("Restarting service '{}'...", service_name);
                // First stop the service
                match client.stop(&service_name).await {
                    Ok(_) => println!("Service stopped successfully"),
                    Err(e) => println!("Error stopping service: {}", e),
                }

                // Wait a moment for the service to stop
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Then start it again
                match client.start(&service_name).await {
                    Ok(_) => println!("Service started successfully"),
                    Err(e) => println!("Error starting service: {}", e),
                }

                // Wait a moment for the service to restart
                tokio::time::sleep(Duration::from_secs(1)).await;

                // Get the new status
                match client.status(&service_name).await {
                    Ok(status) => {
                        println!("New status:");
                        println!("  PID: {}", status.pid);
                        println!("  State: {:?}", status.state);
                        println!("  Target: {:?}", status.target);
                    }
                    Err(e) => {
                        println!("Error getting status: {}", e);
                    }
                }
            }
            "4" => {
                println!("Getting status for '{}'...", service_name);
                match client.status(&service_name).await {
                    Ok(status) => {
                        println!("Current status:");
                        println!("  PID: {}", status.pid);
                        println!("  State: {:?}", status.state);
                        println!("  Target: {:?}", status.target);
                        println!("  Dependencies:");
                        for (dep, state) in &status.after {
                            println!("    - {}: {}", dep, state);
                        }
                    }
                    Err(e) => {
                        println!("Error getting status: {}", e);
                    }
                }
            }
            "5" => {
                println!("Sending SIGTERM to '{}'...", service_name);
                match client.kill(&service_name, "SIGTERM").await {
                    Ok(_) => println!("Signal sent successfully"),
                    Err(e) => println!("Error sending signal: {}", e),
                }

                // Wait a moment for the signal to take effect
                tokio::time::sleep(Duration::from_secs(1)).await;

                // Get the new status
                match client.status(&service_name).await {
                    Ok(status) => {
                        println!("New status:");
                        println!("  PID: {}", status.pid);
                        println!("  State: {:?}", status.state);
                        println!("  Target: {:?}", status.target);
                    }
                    Err(e) => {
                        println!("Error getting status: {}", e);
                    }
                }
            }
            "6" => {
                println!("Exiting...");
                break;
            }
            _ => {
                println!("Invalid choice, please try again");
            }
        }
    }

    // Stop the mock server
    println!("\nStopping mock server");
    server.stop().await;

    println!("\nExample completed successfully");
    Ok(())
}
