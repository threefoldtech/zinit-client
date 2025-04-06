//! # Zinit Client
//!
//! A Rust client library for interacting with the [Zinit](https://github.com/threefoldtech/zinit) service manager.
//!
//! ## Features
//!
//! - Complete API coverage for all Zinit operations
//! - Robust error handling with custom error types
//! - Automatic reconnection on socket errors
//! - Retry mechanisms for transient failures
//! - Async/await support using Tokio
//! - Strongly typed service states and responses
//! - Efficient log streaming
//!
//! ## Usage
//!
//! ```rust,no_run
//! use zinit_client_rs::{ZinitClient, Result};
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create a client with default configuration
//!     let client = ZinitClient::new("/var/run/zinit.sock");
//!     
//!     // List all services
//!     let services = client.list().await?;
//!     println!("Services: {:?}", services);
//!     
//!     // Start a service
//!     client.start("nginx").await?;
//!     
//!     // Get service status
//!     let status = client.status("nginx").await?;
//!     println!("Nginx status: {:?}", status);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration
//!
//! You can customize the client behavior using `ClientConfig`:
//!
//! ```rust,no_run
//! use zinit_client_rs::{ZinitClient, ClientConfig};
//! use std::time::Duration;
//!
//! let config = ClientConfig {
//!     socket_path: "/var/run/zinit.sock".into(),
//!     connection_timeout: Duration::from_secs(5),
//!     operation_timeout: Duration::from_secs(30),
//!     max_retries: 3,
//!     retry_delay: Duration::from_millis(100),
//!     max_retry_delay: Duration::from_secs(5),
//!     retry_jitter: true,
//! };
//!
//! let client = ZinitClient::with_config(config);
//! ```

mod client;
mod connection;
mod error;
mod models;
mod protocol;
mod retry;

// Re-export public items
pub use client::{ClientConfig, ZinitClient};
pub use error::{Result, ZinitError};
pub use models::{LogEntry, LogStream, ServiceState, ServiceStatus, ServiceTarget};
pub use retry::RetryStrategy;

// Re-export futures for working with streams
pub use futures::StreamExt;
