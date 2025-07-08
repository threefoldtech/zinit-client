# Zinit Client

[![Crates.io](https://img.shields.io/crates/v/zinit-client.svg)](https://crates.io/crates/zinit-client)
[![Documentation](https://docs.rs/zinit-client/badge.svg)](https://docs.rs/zinit-client)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust client library for the [Zinit](https://github.com/threefoldtech/zinit) service manager.

**Universal Compatibility**: Automatically works with both old (v0.2.14) and new (v0.2.25+) Zinit servers through automatic protocol detection.

## Features

- **Zero Configuration**: Automatically detects server version and protocol
- **Complete API**: All Zinit operations (list, start, stop, create, delete, etc.)
- **Async/Await**: Built on Tokio for high performance
- **Type Safe**: Strongly typed service states and responses
- **Error Handling**: Comprehensive error types with helpful messages
- **Backward Compatible**: Works with legacy Zinit installations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zinit-client = "0.4.0"
```

## Quick Start

```rust
use zinit_client::ZinitClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ZinitClient::new("/var/run/zinit.sock");

    // List all services
    let services = client.list().await?;
    println!("Services: {:?}", services);

    // Start a service
    client.start("my-service").await?;

    // Get service status
    let status = client.status("my-service").await?;
    println!("Status: {:?}", status);

    Ok(())
}
```

## API Overview

### Service Management

```rust
// List all services
let services = client.list().await?;

// Service lifecycle
client.start("service-name").await?;
client.stop("service-name").await?;
client.restart("service-name").await?;

// Get detailed status
let status = client.status("service-name").await?;

// Create/delete services (if supported by server)
client.create_service("name", config).await?;
client.delete_service("name").await?;
```

## Examples

Run the demo to see the universal interface in action:

```bash
cargo run --example <example_name> <sock_path>
```

## Documentation

For detailed API documentation, visit [docs.rs/zinit-client](https://docs.rs/zinit-client).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
