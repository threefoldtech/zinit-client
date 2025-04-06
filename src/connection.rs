use crate::error::{Result, ZinitError};
use crate::protocol::ProtocolHandler;
use crate::retry::RetryStrategy;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::net::UnixStream;
use tokio::time::timeout;
use tracing::{debug, error, trace};

/// Manages connections to the Zinit Unix socket
#[derive(Debug, Clone)]
pub struct ConnectionManager {
    /// Path to the Zinit Unix socket
    socket_path: PathBuf,
    /// Timeout for connection attempts
    connection_timeout: Duration,
    /// Timeout for operations
    operation_timeout: Duration,
    /// Retry strategy for failed operations
    retry_strategy: RetryStrategy,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(
        socket_path: impl AsRef<Path>,
        connection_timeout: Duration,
        operation_timeout: Duration,
        retry_strategy: RetryStrategy,
    ) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            connection_timeout,
            operation_timeout,
            retry_strategy,
        }
    }

    /// Connect to the Zinit Unix socket
    async fn connect(&self) -> Result<UnixStream> {
        debug!("Connecting to Zinit socket at {:?}", self.socket_path);

        let connect_future = UnixStream::connect(&self.socket_path);
        match timeout(self.connection_timeout, connect_future).await {
            Ok(result) => {
                let stream = result.map_err(|e| {
                    error!("Failed to connect to Zinit socket: {}", e);
                    ZinitError::ConnectionError(e)
                })?;
                debug!("Connected to Zinit socket");
                Ok(stream)
            }
            Err(_) => {
                error!("Connection timeout after {:?}", self.connection_timeout);
                Err(ZinitError::TimeoutError(self.connection_timeout))
            }
        }
    }

    /// Send a command to Zinit and get the response
    pub async fn send_command(&self, command: &str) -> Result<String> {
        debug!("Sending command: {}", command);

        self.retry_strategy
            .retry(|| async {
                let stream = self.connect().await?;
                let mut buf_stream = BufStream::new(stream);

                // Send the command
                trace!("Writing command to socket");
                buf_stream.write_all(command.as_bytes()).await?;
                buf_stream.write_all(b"\n").await?;
                buf_stream.flush().await?;

                // Read the response
                trace!("Reading response from socket");
                let mut response = String::new();

                // Use timeout for reading the response
                match timeout(
                    self.operation_timeout,
                    buf_stream.read_to_string(&mut response),
                )
                .await
                {
                    Ok(result) => {
                        result?;
                        trace!("Response received: {} bytes", response.len());
                        Ok(response)
                    }
                    Err(_) => {
                        error!("Operation timeout after {:?}", self.operation_timeout);
                        Err(ZinitError::TimeoutError(self.operation_timeout))
                    }
                }
            })
            .await
    }

    /// Send a command to Zinit and parse the response
    pub async fn execute_command(&self, command: &str) -> Result<serde_json::Value> {
        let response = self.send_command(command).await?;
        ProtocolHandler::parse_response(&response)
    }

    /// Stream logs from Zinit
    pub async fn stream_logs(&self, command: &str) -> Result<UnixStream> {
        debug!("Streaming logs with command: {}", command);

        let stream = self.connect().await?;
        let mut buf_stream = BufStream::new(stream);

        // Send the command
        buf_stream.write_all(command.as_bytes()).await?;
        buf_stream.write_all(b"\n").await?;
        buf_stream.flush().await?;

        // Return the stream for continuous reading
        Ok(buf_stream.into_inner())
    }
}
