use std::io;
use std::time::Duration;
use thiserror::Error;

/// Custom error types for the Zinit client
#[derive(Debug, Error)]
pub enum ZinitError {
    /// Error connecting to the Zinit socket
    #[error("Connection error: {0}")]
    ConnectionError(#[from] io::Error),

    /// Error in the Zinit protocol
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Error from the Zinit service
    #[error("Service error: {0}")]
    ServiceError(String),

    /// Operation timed out
    #[error("Timeout error: operation took longer than {0:?}")]
    TimeoutError(Duration),

    /// Retry limit reached
    #[error("Retry limit reached after {0} attempts")]
    RetryLimitReached(usize),

    /// Error parsing JSON response
    #[error("Parse error: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Unknown service
    #[error("Unknown service: {0}")]
    UnknownService(String),

    /// Service already monitored
    #[error("Service already monitored: {0}")]
    ServiceAlreadyMonitored(String),

    /// Service is up (can't forget)
    #[error("Service is up: {0}")]
    ServiceIsUp(String),

    /// Service is down (can't kill)
    #[error("Service is down: {0}")]
    ServiceIsDown(String),

    /// Invalid signal name
    #[error("Invalid signal name: {0}")]
    InvalidSignal(String),

    /// System is shutting down
    #[error("System is shutting down")]
    ShuttingDown,

    /// Feature not supported by this server version
    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),

    /// Protocol detection failed
    #[error("Protocol detection failed: {0}")]
    ProtocolDetectionFailed(String),

    /// JSON-RPC error from server
    #[error("JSON-RPC error (code {code}): {message}")]
    JsonRpcError {
        /// Error code
        code: i32,
        /// Error message
        message: String,
        /// Additional error data
        data: Option<serde_json::Value>,
    },
}

/// Result type for Zinit client operations
pub type Result<T> = std::result::Result<T, ZinitError>;
