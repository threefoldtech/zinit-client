# Zinit Client

A Rust client library for interacting with the [Zinit](https://github.com/threefoldtech/zinit) service manager.

## Features

- Complete API coverage for all Zinit operations
- Robust error handling with custom error types
- Automatic reconnection on socket errors
- Retry mechanisms for transient failures
- Async/await support using Tokio
- Strongly typed service states and responses
- Efficient log streaming

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zinit-client = "0.1.0"
```

## Building

To build the library, you need Rust and Cargo installed. Then run:

```bash
# Build the library
cargo build

# Build with optimizations
cargo build --release

# Build the examples
cargo build --examples
```

## Usage

```rust
use zinit_client::{ZinitClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a client with default configuration
    let client = ZinitClient::new("/var/run/zinit.sock");
    
    // List all services
    let services = client.list().await?;
    println!("Services: {:?}", services);
    
    // Start a service
    client.start("nginx").await?;
    
    // Get service status
    let status = client.status("nginx").await?;
    println!("Nginx status: {:?}", status);
    
    // Stream logs
    let mut logs = client.logs(true, Some("nginx")).await?;
    while let Some(log) = logs.next().await {
        println!("{}: {}", log?.timestamp, log?.message);
    }
    
    Ok(())
}
```

## Configuration

You can customize the client behavior using `ClientConfig`:

```rust
use zinit_client::{ZinitClient, ClientConfig};
use std::time::Duration;

let config = ClientConfig {
    socket_path: "/var/run/zinit.sock".into(),
    connection_timeout: Duration::from_secs(5),
    operation_timeout: Duration::from_secs(30),
    max_retries: 3,
    retry_delay: Duration::from_millis(100),
    max_retry_delay: Duration::from_secs(5),
    retry_jitter: true,
};

let client = ZinitClient::with_config(config);
```

## Examples

See the [examples](./examples) directory for more usage examples:

### Running Examples

To run the examples, you need a running Zinit instance. The examples will try to connect to Zinit at the default socket path (`/var/run/zinit.sock`).

```bash
# Basic usage example (requires Zinit)
cargo run --example basic_usage

# Service management example (interactive, requires Zinit)
cargo run --example service_management

# Log streaming example (interactive, requires Zinit)
cargo run --example log_streaming

# Mock server demo (doesn't require Zinit)
cargo run --example mock_server_demo
```

If you want to use a different socket path, you'll need to modify the examples. Open the example file and change the socket path in the `ZinitClient::new()` call:

```rust
// Change this line
let client = ZinitClient::new("/var/run/zinit.sock");

// To use a custom socket path
let client = ZinitClient::new("/path/to/your/zinit.sock");
```

### Running Without Zinit

If you don't have Zinit running, you can still test the client by using the mock server provided in the tests directory. The mock server simulates a Zinit instance for testing purposes.

Here's how to use it:

```rust
use std::path::PathBuf;
use tempfile::tempdir;
use zinit_client::ZinitClient;
use zinit_client::tests::MockZinitServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a temporary directory for the socket
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let socket_path = temp_dir.path().join("mock-zinit.sock");
    
    // Create and start the mock server
    let mut server = MockZinitServer::new(&socket_path).await;
    server.start().await.expect("Failed to start mock server");
    
    // Add some mock services
    server.add_service(MockService {
        name: "test-service".to_string(),
        pid: 1001,
        state: MockServiceState::Running,
        target: MockServiceTarget::Up,
        after: HashMap::new(),
    });
    
    // Create a client to connect to the mock server
    let client = ZinitClient::new(&socket_path);
    
    // Use the client as normal
    let services = client.list().await?;
    println!("Services: {:?}", services);
    
    // Stop the mock server when done
    server.stop().await;
    
    Ok(())
}
```

Note: The examples assume Zinit is running and listening on the default socket path (`/var/run/zinit.sock`). If your Zinit instance is using a different socket path, you'll need to modify the examples accordingly.

## Testing

The library includes both integration tests and a mock server for testing without requiring a real Zinit instance.

### Running Tests

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_client_reconnection
```

All tests use the mock server, so they can be run without requiring a real Zinit instance. The mock server simulates a Zinit instance for testing purposes, allowing for reliable and reproducible tests.

