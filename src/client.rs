use crate::connection::ConnectionManager;
use crate::error::{Result, ZinitError};
use crate::models::{LogEntry, LogStream, ServiceState, ServiceStatus, ServiceTarget};
use crate::protocol::ProtocolHandler;
use crate::retry::RetryStrategy;
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{debug, trace};

/// Configuration for the Zinit client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Path to the Zinit Unix socket
    pub socket_path: PathBuf,
    /// Timeout for connection attempts
    pub connection_timeout: Duration,
    /// Timeout for operations
    pub operation_timeout: Duration,
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Base delay between retries
    pub retry_delay: Duration,
    /// Maximum delay between retries
    pub max_retry_delay: Duration,
    /// Whether to add jitter to retry delays
    pub retry_jitter: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from("/var/run/zinit.sock"),
            connection_timeout: Duration::from_secs(5),
            operation_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(5),
            retry_jitter: true,
        }
    }
}

/// Client for interacting with Zinit
#[derive(Debug, Clone)]
pub struct ZinitClient {
    /// Connection manager
    connection_manager: ConnectionManager,
    /// Client configuration
    #[allow(dead_code)]
    config: ClientConfig,
}

impl ZinitClient {
    /// Create a new Zinit client with the default configuration
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self::with_config(ClientConfig {
            socket_path: socket_path.as_ref().to_path_buf(),
            ..Default::default()
        })
    }

    /// Create a new Zinit client with a custom configuration
    pub fn with_config(config: ClientConfig) -> Self {
        let retry_strategy = RetryStrategy::new(
            config.max_retries,
            config.retry_delay,
            config.max_retry_delay,
            config.retry_jitter,
        );

        let connection_manager = ConnectionManager::new(
            &config.socket_path,
            config.connection_timeout,
            config.operation_timeout,
            retry_strategy,
        );

        Self {
            connection_manager,
            config,
        }
    }

    /// List all services and their states
    pub async fn list(&self) -> Result<HashMap<String, ServiceState>> {
        debug!("Listing all services");
        let response = self.connection_manager.execute_command("list").await?;

        let map: HashMap<String, String> = serde_json::from_value(response)?;
        let result = map
            .into_iter()
            .map(|(name, state_str)| {
                let state = match state_str.as_str() {
                    "Unknown" => ServiceState::Unknown,
                    "Blocked" => ServiceState::Blocked,
                    "Spawned" => ServiceState::Spawned,
                    "Running" => ServiceState::Running,
                    "Success" => ServiceState::Success,
                    "Error" => ServiceState::Error,
                    "TestFailure" => ServiceState::TestFailure,
                    _ => ServiceState::Unknown,
                };
                (name, state)
            })
            .collect();

        Ok(result)
    }

    /// Get the status of a service
    pub async fn status(&self, service: impl AsRef<str>) -> Result<ServiceStatus> {
        let service_name = service.as_ref();
        debug!("Getting status for service: {}", service_name);

        let command = ProtocolHandler::format_command("status", &[service_name]);
        let response = self.connection_manager.execute_command(&command).await?;

        let mut status: ServiceStatus = serde_json::from_value(response)?;

        // Convert state string to enum
        status.state = match status.state.to_string().as_str() {
            "Unknown" => ServiceState::Unknown,
            "Blocked" => ServiceState::Blocked,
            "Spawned" => ServiceState::Spawned,
            "Running" => ServiceState::Running,
            "Success" => ServiceState::Success,
            "Error" => ServiceState::Error,
            "TestFailure" => ServiceState::TestFailure,
            _ => ServiceState::Unknown,
        };

        // Convert target string to enum
        status.target = match status.target.to_string().as_str() {
            "Up" => ServiceTarget::Up,
            "Down" => ServiceTarget::Down,
            _ => ServiceTarget::Down,
        };

        Ok(status)
    }

    /// Start a service
    pub async fn start(&self, service: impl AsRef<str>) -> Result<()> {
        let service_name = service.as_ref();
        debug!("Starting service: {}", service_name);

        let command = ProtocolHandler::format_command("start", &[service_name]);
        self.connection_manager.execute_command(&command).await?;

        Ok(())
    }

    /// Stop a service
    pub async fn stop(&self, service: impl AsRef<str>) -> Result<()> {
        let service_name = service.as_ref();
        debug!("Stopping service: {}", service_name);

        let command = ProtocolHandler::format_command("stop", &[service_name]);
        self.connection_manager.execute_command(&command).await?;

        Ok(())
    }

    /// Restart a service
    pub async fn restart(&self, service: impl AsRef<str>) -> Result<()> {
        let service_name = service.as_ref();
        debug!("Restarting service: {}", service_name);

        // First stop the service
        self.stop(service_name).await?;

        // Wait for the service to stop
        let mut attempts = 0;
        let max_attempts = 20;

        while attempts < max_attempts {
            let status = self.status(service_name).await?;
            if status.pid == 0 && status.target == ServiceTarget::Down {
                // Service is stopped, now start it
                return self.start(service_name).await;
            }

            attempts += 1;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        // Service didn't stop gracefully, try to kill it
        self.kill(service_name, "SIGKILL").await?;
        self.start(service_name).await
    }

    /// Monitor a service
    pub async fn monitor(&self, service: impl AsRef<str>) -> Result<()> {
        let service_name = service.as_ref();
        debug!("Monitoring service: {}", service_name);

        let command = ProtocolHandler::format_command("monitor", &[service_name]);
        self.connection_manager.execute_command(&command).await?;

        Ok(())
    }

    /// Forget a service
    pub async fn forget(&self, service: impl AsRef<str>) -> Result<()> {
        let service_name = service.as_ref();
        debug!("Forgetting service: {}", service_name);

        let command = ProtocolHandler::format_command("forget", &[service_name]);
        self.connection_manager.execute_command(&command).await?;

        Ok(())
    }

    /// Send a signal to a service
    pub async fn kill(&self, service: impl AsRef<str>, signal: impl AsRef<str>) -> Result<()> {
        let service_name = service.as_ref();
        let signal_name = signal.as_ref();
        debug!(
            "Sending signal {} to service: {}",
            signal_name, service_name
        );

        let command = ProtocolHandler::format_command("kill", &[service_name, signal_name]);
        self.connection_manager.execute_command(&command).await?;

        Ok(())
    }

    /// Stream logs from services
    pub async fn logs(&self, follow: bool, filter: Option<impl AsRef<str>>) -> Result<LogStream> {
        let command = if follow {
            "log".to_string()
        } else {
            "log snapshot".to_string()
        };

        debug!("Streaming logs with command: {}", command);
        let stream = self.connection_manager.stream_logs(&command).await?;
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        // Create a stream of log entries
        let filter_str = filter.as_ref().map(|f| f.as_ref().to_string());

        let log_stream = async_stream::stream! {
            while let Some(line_result) = lines.next_line().await.transpose() {
                match line_result {
                    Ok(line) => {
                        trace!("Received log line: {}", line);

                        // Parse the log line
                        if let Some(entry) = parse_log_line(&line, &filter_str) {
                            yield Ok(entry);
                        }
                    }
                    Err(e) => {
                        yield Err(ZinitError::ConnectionError(e));
                        break;
                    }
                }
            }
        };

        Ok(LogStream {
            inner: Box::pin(log_stream),
        })
    }

    /// Shutdown the system
    pub async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down the system");
        self.connection_manager.execute_command("shutdown").await?;
        Ok(())
    }

    /// Reboot the system
    pub async fn reboot(&self) -> Result<()> {
        debug!("Rebooting the system");
        self.connection_manager.execute_command("reboot").await?;
        Ok(())
    }

    /// Get raw service information
    pub async fn get_service(&self, service: impl AsRef<str>) -> Result<serde_json::Value> {
        let service_name = service.as_ref();
        debug!("Getting raw service info for: {}", service_name);

        let command = ProtocolHandler::format_command("status", &[service_name]);
        let response = self.connection_manager.execute_command(&command).await?;

        Ok(response)
    }

    /// Create a new service
    pub async fn create_service(
        &self,
        name: impl AsRef<str>,
        config: serde_json::Value,
    ) -> Result<()> {
        let service_name = name.as_ref();
        debug!("Creating service: {}", service_name);

        // Convert the config to a string
        let config_str = serde_json::to_string(&config)?;

        // Format the command with the service name and config
        let command = ProtocolHandler::format_command("create", &[service_name, &config_str]);
        self.connection_manager.execute_command(&command).await?;

        Ok(())
    }

    /// Delete a service
    pub async fn delete_service(&self, name: impl AsRef<str>) -> Result<()> {
        let service_name = name.as_ref();
        debug!("Deleting service: {}", service_name);

        // First ensure the service is stopped
        let status = self.status(service_name).await?;
        if status.state == ServiceState::Running || status.target == ServiceTarget::Up {
            // Stop the service first
            self.stop(service_name).await?;

            // Wait for the service to stop
            let mut attempts = 0;
            let max_attempts = 10;

            while attempts < max_attempts {
                let status = self.status(service_name).await?;
                if status.pid == 0 && status.target == ServiceTarget::Down {
                    break;
                }

                attempts += 1;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        // Now forget the service
        self.forget(service_name).await?;

        Ok(())
    }
}

/// Parse a log line into a LogEntry
fn parse_log_line(line: &str, filter: &Option<String>) -> Option<LogEntry> {
    // Example log line: "zinit: INFO (service) message"
    let parts: Vec<&str> = line.splitn(4, ' ').collect();

    if parts.len() < 4 || !parts[0].starts_with("zinit:") {
        return None;
    }

    let level = parts[1];
    let service = parts[2].trim_start_matches('(').trim_end_matches(')');

    // Apply filter if provided
    if let Some(filter_str) = filter {
        if service != filter_str {
            return None;
        }
    }

    let message = parts[3];
    let timestamp = Utc::now(); // Zinit doesn't include timestamps, so we use current time

    Some(LogEntry {
        timestamp,
        service: service.to_string(),
        message: format!("[{}] {}", level, message),
    })
}
