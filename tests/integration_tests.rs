use std::collections::HashMap;
use std::time::Duration;
use tempfile::tempdir;
use zinit_client_rs::{ClientConfig, Result, ServiceState, StreamExt, ZinitClient};

// Import the mock server
mod mock_server;
use mock_server::{MockService, MockServiceState, MockServiceTarget, MockZinitServer};

#[tokio::test]
async fn test_client_list_services() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

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

    // List services
    let services = client.list().await?;
    println!("Found {} services", services.len());

    // Verify we got some services
    assert!(!services.is_empty(), "No services found");
    assert_eq!(services.len(), 2, "Expected 2 services");
    assert!(
        services.contains_key("web-server"),
        "Expected web-server service"
    );
    assert!(
        services.contains_key("database"),
        "Expected database service"
    );

    // Stop the mock server
    server.stop().await;

    Ok(())
}

#[tokio::test]
async fn test_client_service_lifecycle() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Add a test service
    let service_name = "test-service";
    server.add_service(MockService {
        name: service_name.to_string(),
        pid: 1001,
        state: MockServiceState::Running,
        target: MockServiceTarget::Up,
        after: HashMap::new(),
    });

    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);

    // Get initial status
    let status = client.status(service_name).await?;
    assert_eq!(
        status.state,
        ServiceState::Running,
        "Service should be running"
    );
    assert!(status.pid > 0, "Service should have a valid PID");

    // Stop the service
    client.stop(service_name).await?;

    // Get status again
    let status = client.status(service_name).await?;
    assert_ne!(
        status.state,
        ServiceState::Running,
        "Service should not be running"
    );

    // Start the service again
    client.start(service_name).await?;

    // Get status again
    let status = client.status(service_name).await?;
    assert_eq!(
        status.state,
        ServiceState::Running,
        "Service should be running again"
    );

    // Stop the service for forget test
    client.stop(service_name).await?;

    // Forget the service
    client.forget(service_name).await?;

    // Verify the service is forgotten
    let services = client.list().await?;
    assert!(
        !services.contains_key(service_name),
        "Service should be forgotten"
    );

    // Stop the mock server
    server.stop().await;

    Ok(())
}

#[tokio::test]
async fn test_client_reconnection() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Add a test service
    server.add_service(MockService {
        name: "test-service".to_string(),
        pid: 1001,
        state: MockServiceState::Running,
        target: MockServiceTarget::Up,
        after: HashMap::new(),
    });

    // Create a client with custom configuration for quick retries
    let config = ClientConfig {
        socket_path: socket_path.to_str().unwrap().to_string().into(),
        connection_timeout: Duration::from_secs(1),
        operation_timeout: Duration::from_secs(5),
        max_retries: 3,
        retry_delay: Duration::from_millis(100),
        max_retry_delay: Duration::from_secs(1),
        retry_jitter: false,
    };

    let client = ZinitClient::with_config(config);

    // List services
    let services = client.list().await?;
    println!("Found {} services", services.len());
    assert_eq!(services.len(), 1, "Expected 1 service");

    // Stop the mock server
    server.stop().await;

    Ok(())
}

#[tokio::test]
async fn test_client_logs() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Add a test service
    server.add_service(MockService {
        name: "test-service".to_string(),
        pid: 1001,
        state: MockServiceState::Running,
        target: MockServiceTarget::Up,
        after: HashMap::new(),
    });

    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);

    // Get a snapshot of logs
    let mut logs = client.logs(false, None::<&str>).await?;

    // Count log entries
    let mut count = 0;
    while let Some(log_result) = logs.next().await {
        let _ = log_result?;
        count += 1;
        if count >= 10 {
            break; // Limit to 10 entries for the test
        }
    }

    println!("Received {} log entries", count);

    // The mock server returns 2 log entries
    assert!(count > 0, "Expected at least one log entry");

    // Stop the mock server
    server.stop().await;

    Ok(())
}

#[tokio::test]
async fn test_client_custom_config() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Add a test service
    server.add_service(MockService {
        name: "test-service".to_string(),
        pid: 1001,
        state: MockServiceState::Running,
        target: MockServiceTarget::Up,
        after: HashMap::new(),
    });

    // Create a client with custom configuration
    let config = ClientConfig {
        socket_path: socket_path.to_str().unwrap().to_string().into(),
        connection_timeout: Duration::from_secs(2),
        operation_timeout: Duration::from_secs(10),
        max_retries: 5,
        retry_delay: Duration::from_millis(200),
        max_retry_delay: Duration::from_secs(2),
        retry_jitter: true,
    };

    let client = ZinitClient::with_config(config);

    // Test that the client works with the custom configuration
    let services = client.list().await?;
    assert_eq!(services.len(), 1, "Expected 1 service");

    // Stop the mock server
    server.stop().await;

    Ok(())
}
