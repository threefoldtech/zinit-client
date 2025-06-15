use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tempfile::tempdir;
use zinit_client::{ClientConfig, Result, ServiceState, StreamExt, ZinitClient};

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

#[tokio::test]
async fn test_create_service() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);

    // Test creating a simple service
    let service_config = json!({
        "exec": "/usr/bin/test-app",
        "args": ["--port", "8080"],
        "oneshot": false
    });

    // Create the service
    client.create_service("test-app", service_config).await?;

    // Verify the service was created by listing services
    let services = client.list().await?;
    assert!(
        services.contains_key("test-app"),
        "Service should be created"
    );

    // Verify the service is in Unknown state initially
    assert_eq!(services["test-app"], ServiceState::Unknown);

    // Stop the mock server
    server.stop().await;

    Ok(())
}

#[tokio::test]
async fn test_create_service_already_exists() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Add an existing service
    server.add_service(MockService {
        name: "existing-service".to_string(),
        pid: 1001,
        state: MockServiceState::Running,
        target: MockServiceTarget::Up,
        after: HashMap::new(),
    });

    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);

    // Try to create a service with the same name
    let service_config = json!({
        "exec": "/usr/bin/test-app",
        "oneshot": false
    });

    // This should fail
    let result = client
        .create_service("existing-service", service_config)
        .await;
    assert!(result.is_err(), "Creating duplicate service should fail");

    // Stop the mock server
    server.stop().await;

    Ok(())
}

#[tokio::test]
async fn test_create_service_invalid_config() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);

    // Try to create a service with invalid JSON (this will be caught by serde_json)
    let invalid_config = json!("invalid");

    // This should work at the client level but might fail at the server level
    // depending on how the mock server validates the config
    let result = client
        .create_service("invalid-service", invalid_config)
        .await;

    // The mock server should accept any valid JSON, so this should succeed
    // but the service will be created with default values
    assert!(result.is_ok(), "Valid JSON should be accepted");

    // Stop the mock server
    server.stop().await;

    Ok(())
}

#[tokio::test]
async fn test_get_service() -> Result<()> {
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

    // Get the service configuration
    let service_info = client.get_service("test-service").await?;

    // Verify the service information
    assert!(
        service_info.is_object(),
        "Service info should be a JSON object"
    );

    let service_obj = service_info.as_object().unwrap();
    assert_eq!(service_obj["name"], "test-service");
    assert_eq!(service_obj["pid"], 1001);
    assert_eq!(service_obj["state"], "Running");
    assert_eq!(service_obj["target"], "Up");

    // Stop the mock server
    server.stop().await;

    Ok(())
}

#[tokio::test]
async fn test_get_service_not_found() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);

    // Try to get a non-existent service
    let result = client.get_service("non-existent-service").await;
    assert!(result.is_err(), "Getting non-existent service should fail");

    // Stop the mock server
    server.stop().await;

    Ok(())
}

#[tokio::test]
async fn test_delete_service() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Add a test service that's stopped (so it can be deleted)
    server.add_service(MockService {
        name: "test-service".to_string(),
        pid: 0,
        state: MockServiceState::Success,
        target: MockServiceTarget::Down,
        after: HashMap::new(),
    });

    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);

    // Verify the service exists
    let services = client.list().await?;
    assert!(
        services.contains_key("test-service"),
        "Service should exist"
    );

    // Delete the service
    client.delete_service("test-service").await?;

    // Verify the service is gone
    let services = client.list().await?;
    assert!(
        !services.contains_key("test-service"),
        "Service should be deleted"
    );

    // Stop the mock server
    server.stop().await;

    Ok(())
}

#[tokio::test]
async fn test_delete_running_service() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Add a running test service
    server.add_service(MockService {
        name: "running-service".to_string(),
        pid: 1001,
        state: MockServiceState::Running,
        target: MockServiceTarget::Up,
        after: HashMap::new(),
    });

    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);

    // Verify the service exists and is running
    let status = client.status("running-service").await?;
    assert_eq!(status.state, ServiceState::Running);

    // Delete the service (should stop it first, then delete)
    client.delete_service("running-service").await?;

    // Verify the service is gone
    let services = client.list().await?;
    assert!(
        !services.contains_key("running-service"),
        "Service should be deleted"
    );

    // Stop the mock server
    server.stop().await;

    Ok(())
}

#[tokio::test]
async fn test_delete_service_not_found() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);

    // Try to delete a non-existent service
    let result = client.delete_service("non-existent-service").await;
    assert!(result.is_err(), "Deleting non-existent service should fail");

    // Stop the mock server
    server.stop().await;

    Ok(())
}

#[tokio::test]
async fn test_service_crud_lifecycle() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");

    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");

    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);

    let service_name = "lifecycle-test-service";

    // 1. CREATE: Create a new service
    let service_config = json!({
        "exec": "/usr/bin/test-app",
        "args": ["--config", "/etc/test.conf"],
        "env": {
            "PORT": "8080",
            "ENV": "test"
        },
        "oneshot": false,
        "working_dir": "/opt/test"
    });

    client.create_service(service_name, service_config).await?;

    // Verify service was created
    let services = client.list().await?;
    assert!(
        services.contains_key(service_name),
        "Service should be created"
    );
    assert_eq!(services[service_name], ServiceState::Unknown);

    // 2. READ: Get service information
    let service_info = client.get_service(service_name).await?;
    assert!(
        service_info.is_object(),
        "Service info should be a JSON object"
    );

    let service_obj = service_info.as_object().unwrap();
    assert_eq!(service_obj["name"], service_name);
    assert_eq!(service_obj["state"], "Unknown");
    assert_eq!(service_obj["target"], "Down");
    assert_eq!(service_obj["pid"], 0);

    // 3. UPDATE: Start the service (this changes its state)
    client.start(service_name).await?;

    // Verify the service is now running
    let updated_info = client.get_service(service_name).await?;
    let updated_obj = updated_info.as_object().unwrap();
    assert_eq!(updated_obj["state"], "Running");
    assert_eq!(updated_obj["target"], "Up");
    assert!(updated_obj["pid"].as_u64().unwrap() > 0);

    // 4. DELETE: Delete the service
    client.delete_service(service_name).await?;

    // Verify the service is gone
    let services = client.list().await?;
    assert!(
        !services.contains_key(service_name),
        "Service should be deleted"
    );

    // Verify we can't get the deleted service
    let result = client.get_service(service_name).await;
    assert!(result.is_err(), "Getting deleted service should fail");

    // Stop the mock server
    server.stop().await;

    Ok(())
}
