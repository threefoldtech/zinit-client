use serde_json::json;
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

    println!("Starting comprehensive service management example with mock server");
    println!("This example demonstrates all service operations: control, CRUD, and monitoring");

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
        println!("=== Service Control ===");
        println!("1. Start service");
        println!("2. Stop service");
        println!("3. Restart service");
        println!("4. Get status");
        println!("5. Send signal (SIGTERM)");
        println!("=== Service Management ===");
        println!("6. Create new service");
        println!("7. Get service configuration");
        println!("8. Delete service");
        println!("9. List all services");
        println!("=== Other ===");
        println!("0. Exit");

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
                println!("Creating a new service...");
                println!("Enter service name:");
                let mut new_service_name = String::new();
                std::io::stdin()
                    .read_line(&mut new_service_name)
                    .expect("Failed to read input");
                let new_service_name = new_service_name.trim();

                if new_service_name.is_empty() {
                    println!("Service name cannot be empty");
                    continue;
                }

                println!("Enter executable path (e.g., /usr/bin/nginx):");
                let mut exec_path = String::new();
                std::io::stdin()
                    .read_line(&mut exec_path)
                    .expect("Failed to read input");
                let exec_path = exec_path.trim();

                if exec_path.is_empty() {
                    println!("Executable path cannot be empty");
                    continue;
                }

                // Create a basic service configuration
                let service_config = json!({
                    "exec": exec_path,
                    "oneshot": false,
                    "env": {
                        "CREATED_BY": "zinit-client-example"
                    }
                });

                match client
                    .create_service(new_service_name, service_config)
                    .await
                {
                    Ok(_) => {
                        println!("✓ Service '{}' created successfully", new_service_name);
                        println!("You can now manage it using the other menu options");
                    }
                    Err(e) => println!("✗ Error creating service: {}", e),
                }
            }
            "7" => {
                println!("Getting service configuration for '{}'...", service_name);
                match client.get_service(&service_name).await {
                    Ok(config) => {
                        println!("✓ Service configuration:");
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&config)
                                .unwrap_or_else(|_| "Failed to format JSON".to_string())
                        );
                    }
                    Err(e) => println!("✗ Error getting service configuration: {}", e),
                }
            }
            "8" => {
                println!(
                    "⚠️  WARNING: This will permanently delete the service '{}'",
                    service_name
                );
                println!("Are you sure? (y/N):");
                let mut confirmation = String::new();
                std::io::stdin()
                    .read_line(&mut confirmation)
                    .expect("Failed to read input");
                let confirmation = confirmation.trim().to_lowercase();

                if confirmation == "y" || confirmation == "yes" {
                    println!("Deleting service '{}'...", service_name);
                    match client.delete_service(&service_name).await {
                        Ok(_) => {
                            println!("✓ Service '{}' deleted successfully", service_name);
                            println!("Note: You may need to select a different service for further operations");
                        }
                        Err(e) => println!("✗ Error deleting service: {}", e),
                    }
                } else {
                    println!("Delete operation cancelled");
                }
            }
            "9" => {
                println!("Listing all services...");
                match client.list().await {
                    Ok(services) => {
                        if services.is_empty() {
                            println!("No services found");
                        } else {
                            println!("✓ Found {} services:", services.len());
                            for (name, state) in &services {
                                let indicator = if name == &service_name {
                                    " <- current"
                                } else {
                                    ""
                                };
                                println!("  - {}: {:?}{}", name, state, indicator);
                            }
                        }
                    }
                    Err(e) => println!("✗ Error listing services: {}", e),
                }
            }
            "0" => {
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
