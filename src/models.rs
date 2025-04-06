use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::pin::Pin;

use crate::error::Result;

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
