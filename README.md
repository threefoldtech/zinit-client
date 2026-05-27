# ZOS Init Client

[![Crates.io](https://img.shields.io/crates/v/zinit-client.svg)](https://crates.io/crates/zinit-client)
[![Documentation](https://docs.rs/zinit-client/badge.svg)](https://docs.rs/zinit-client)

A Rust client library for interacting with the [ZOS Init](https://github.com/threefoldtech/zinit) service manager. It provides programmatic control over service lifecycle, status queries, and management from Rust applications.

## What this is

ZOS Init Client is a Rust library that enables external tools to monitor and control services supervised by ZOS Init. It communicates with ZOS Init over a Unix domain socket and exposes a typed, async API for service operations. The library automatically detects server version and protocol, making it suitable for both new and legacy ZOS Init installations.

## What this repository contains

- **Typed Rust client** for the ZOS Init Unix socket protocol
- **Service lifecycle operations**: list, start, stop, restart, status, create, delete
- **Async/await API** built on Tokio
- **Strongly typed service states and responses**
- **Comprehensive error types** with helpful messages
- **Examples** demonstrating common usage patterns

## Role in the stack

ZOS Init Client is used by tools and services that need to manage or observe processes supervised by ZOS Init. ZOS Init itself is a lightweight PID 1 replacement and service manager used within ZOS / Zero-OS and container environments. This client library abstracts the wire protocol so Rust applications can integrate with ZOS Init without dealing with raw socket communication.

## ZOS / Zero-OS

ZOS, also known as Zero-OS, is the operating system layer used to run and manage nodes. It provides the low-level runtime environment for workloads, networking, storage, and automation. ZOS Init is the init system used by ZOS, and this client library enables ZOS components and external tooling to interact with it programmatically.

## Relation to ThreeFold

This technology is used within the ThreeFold ecosystem and was first deployed on the ThreeFold Grid. The component itself is designed as reusable infrastructure technology and should be understood by its technical function first, independent of any specific deployment.

## Ownership

This repository is owned and maintained by TF-Tech NV, a Belgian company responsible for the development and maintenance of this technology.

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

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
Copyright (c) TF-Tech NV.
