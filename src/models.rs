use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::pin::Pin;

use crate::error::Result;

/// Protocol type used by the zinit server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// JSON-RPC protocol (new servers v0.2.25+)
    JsonRpc,
    /// Raw command protocol (old servers v0.2.14)
    RawCommands,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::JsonRpc => write!(f, "JSON-RPC"),
            Protocol::RawCommands => write!(f, "Raw Commands"),
        }
    }
}

/// Server capabilities based on version and protocol
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCapabilities {
    /// Protocol used by the server
    pub protocol: Protocol,
    /// Whether the server supports dynamic service creation
    pub supports_create: bool,
    /// Whether the server supports service deletion
    pub supports_delete: bool,
    /// Whether the server supports getting service configuration
    pub supports_get_config: bool,
    /// Whether the server supports service statistics
    pub supports_stats: bool,
    /// Whether the server supports log streaming
    pub supports_log_streaming: bool,
    /// Whether the server supports HTTP/RPC server management
    pub supports_http_server: bool,
}

impl ServerCapabilities {
    /// Full capabilities for new servers (JSON-RPC)
    pub fn full() -> Self {
        Self {
            protocol: Protocol::JsonRpc,
            supports_create: true,
            supports_delete: true,
            supports_get_config: true,
            supports_stats: true,
            supports_log_streaming: true,
            supports_http_server: true,
        }
    }

    /// Legacy capabilities for old servers (raw commands)
    pub fn legacy() -> Self {
        Self {
            protocol: Protocol::RawCommands,
            supports_create: false,
            supports_delete: false,
            supports_get_config: false,
            supports_stats: false,
            supports_log_streaming: true, // Basic log support
            supports_http_server: false,
        }
    }
}

/// Service state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ServiceState {
    /// Service state is unknown
    Unknown,
    /// Service is blocked by dependencies
    Blocked,
    /// Service is spawned but not yet running
    Spawned,
    /// Service is running
    Running,
    /// Service completed successfully (for oneshot services)
    Success,
    /// Service exited with an error
    Error,
    /// Service test command failed
    TestFailure,
}

impl fmt::Display for ServiceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceState::Unknown => write!(f, "Unknown"),
            ServiceState::Blocked => write!(f, "Blocked"),
            ServiceState::Spawned => write!(f, "Spawned"),
            ServiceState::Running => write!(f, "Running"),
            ServiceState::Success => write!(f, "Success"),
            ServiceState::Error => write!(f, "Error"),
            ServiceState::TestFailure => write!(f, "TestFailure"),
        }
    }
}

/// Service target state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ServiceTarget {
    /// Service should be running
    Up,
    /// Service should be stopped
    Down,
}

impl fmt::Display for ServiceTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceTarget::Up => write!(f, "Up"),
            ServiceTarget::Down => write!(f, "Down"),
        }
    }
}

/// Service status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,
    /// Process ID (0 if not running)
    pub pid: u32,
    /// Current state
    pub state: ServiceState,
    /// Target state
    pub target: ServiceTarget,
    /// Dependencies and their states
    pub after: HashMap<String, String>,
}

/// Log entry from a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp of the log entry
    pub timestamp: DateTime<Utc>,
    /// Service that generated the log
    pub service: String,
    /// Log message
    pub message: String,
}

/// Stream of log entries
pub struct LogStream {
    /// Inner stream of log entries
    pub(crate) inner: Pin<Box<dyn Stream<Item = Result<LogEntry>> + Send>>,
}

impl Stream for LogStream {
    type Item = Result<LogEntry>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Response from the Zinit API
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Response {
    /// Response state (ok or error)
    pub state: ResponseState,
    /// Response body
    pub body: serde_json::Value,
}

/// Response state
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ResponseState {
    /// Success response
    Ok,
    /// Error response
    Error,
}

/// JSON-RPC request structure
#[derive(Debug, Clone, Serialize)]
pub(crate) struct JsonRpcRequest {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Method name
    pub method: String,
    /// Parameters
    pub params: serde_json::Value,
    /// Request ID
    pub id: u64,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(method: &str, params: serde_json::Value, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        }
    }
}

/// JSON-RPC response structure
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct JsonRpcResponse {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID
    pub id: Option<u64>,
    /// Result (if successful)
    pub result: Option<serde_json::Value>,
    /// Error (if failed)
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    pub data: Option<serde_json::Value>,
}
