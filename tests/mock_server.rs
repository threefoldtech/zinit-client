use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Mock service state
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MockServiceState {
    Unknown,
    Blocked,
    Spawned,
    Running,
    Success,
    Error,
    TestFailure,
}

impl ToString for MockServiceState {
    fn to_string(&self) -> String {
        match self {
            MockServiceState::Unknown => "Unknown".to_string(),
            MockServiceState::Blocked => "Blocked".to_string(),
            MockServiceState::Spawned => "Spawned".to_string(),
            MockServiceState::Running => "Running".to_string(),
            MockServiceState::Success => "Success".to_string(),
            MockServiceState::Error => "Error".to_string(),
            MockServiceState::TestFailure => "TestFailure".to_string(),
        }
    }
}

/// Mock service target
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockServiceTarget {
    Up,
    Down,
}

impl ToString for MockServiceTarget {
    fn to_string(&self) -> String {
        match self {
            MockServiceTarget::Up => "Up".to_string(),
            MockServiceTarget::Down => "Down".to_string(),
        }
    }
}

/// Mock service
#[derive(Debug, Clone)]
pub struct MockService {
    pub name: String,
    pub pid: u32,
    pub state: MockServiceState,
    pub target: MockServiceTarget,
    pub after: HashMap<String, String>,
}

/// Mock Zinit server for testing
pub struct MockZinitServer {
    socket_path: String,
    services: Arc<Mutex<HashMap<String, MockService>>>,
    server_handle: Option<JoinHandle<()>>,
    shutdown_sender: Option<mpsc::Sender<()>>,
}

impl MockZinitServer {
    /// Create a new mock Zinit server
    pub async fn new(socket_path: impl AsRef<Path>) -> Self {
        let socket_path = socket_path.as_ref().to_string_lossy().to_string();
        let services = Arc::new(Mutex::new(HashMap::new()));

        Self {
            socket_path,
            services,
            server_handle: None,
            shutdown_sender: None,
        }
    }

    /// Start the mock server
    pub async fn start(&mut self) -> std::io::Result<()> {
        // Remove the socket file if it exists
        let _ = std::fs::remove_file(&self.socket_path);

        // Create the listener
        let listener = UnixListener::bind(&self.socket_path)?;
        let services = Arc::clone(&self.services);

        // Create a channel for shutdown signaling
        let (tx, mut rx) = mpsc::channel::<()>(1);
        self.shutdown_sender = Some(tx);

        // Spawn the server task
        let socket_path = self.socket_path.clone();
        let handle = tokio::spawn(async move {
            println!("Mock Zinit server started on {}", socket_path);

            loop {
                tokio::select! {
                    Ok((stream, _)) = listener.accept() => {
                        let services = Arc::clone(&services);
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, services).await {
                                eprintln!("Error handling connection: {}", e);
                            }
                        });
                    }
                    _ = rx.recv() => {
                        println!("Mock Zinit server shutting down");
                        break;
                    }
                }
            }

            // Clean up the socket file
            let _ = std::fs::remove_file(&socket_path);
        });

        self.server_handle = Some(handle);
        Ok(())
    }

    /// Stop the mock server
    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown_sender.take() {
            let _ = tx.send(()).await;
        }

        if let Some(handle) = self.server_handle.take() {
            let _ = handle.await;
        }
    }

    /// Add a mock service
    pub fn add_service(&self, service: MockService) {
        let mut services = self.services.lock().unwrap();
        services.insert(service.name.clone(), service);
    }

    /// Remove a mock service
    #[allow(dead_code)]
    pub fn remove_service(&self, name: &str) {
        let mut services = self.services.lock().unwrap();
        services.remove(name);
    }

    /// Get the socket path
    #[allow(dead_code)]
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }

    /// Handle a client connection
    async fn handle_connection(
        stream: UnixStream,
        services: Arc<Mutex<HashMap<String, MockService>>>,
    ) -> std::io::Result<()> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        // Read the command
        reader.read_line(&mut line).await?;
        let command = line.trim();

        // Process the command
        let response = Self::process_command(command, &services);

        // Send the response
        let mut stream = reader.into_inner();
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Process a command and generate a response
    fn process_command(
        command: &str,
        services: &Arc<Mutex<HashMap<String, MockService>>>,
    ) -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            return r#"{"state":"error","body":"unknown command"}"#.to_string();
        }

        match parts[0] {
            "list" => {
                let services_lock = services.lock().unwrap();
                let mut map = HashMap::new();

                for (name, service) in services_lock.iter() {
                    map.insert(name.clone(), service.state.to_string());
                }

                let body = serde_json::to_value(map).unwrap();
                format!(r#"{{"state":"ok","body":{}}}"#, body)
            }
            "status" if parts.len() == 2 => {
                let service_name = parts[1];
                let services_lock = services.lock().unwrap();

                match services_lock.get(service_name) {
                    Some(service) => {
                        let status = serde_json::json!({
                            "name": service.name,
                            "pid": service.pid,
                            "state": service.state.to_string(),
                            "target": service.target.to_string(),
                            "after": service.after
                        });

                        format!(r#"{{"state":"ok","body":{}}}"#, status)
                    }
                    None => {
                        format!(
                            r#"{{"state":"error","body":"service name \"{}\" unknown"}}"#,
                            service_name
                        )
                    }
                }
            }
            "start" if parts.len() == 2 => {
                let service_name = parts[1];
                let mut services_lock = services.lock().unwrap();

                match services_lock.get_mut(service_name) {
                    Some(service) => {
                        service.target = MockServiceTarget::Up;
                        if service.state != MockServiceState::Running {
                            service.state = MockServiceState::Running;
                            service.pid = 1000 + rand::random::<u32>() % 9000;
                        }
                        r#"{"state":"ok","body":null}"#.to_string()
                    }
                    None => {
                        format!(
                            r#"{{"state":"error","body":"service name \"{}\" unknown"}}"#,
                            service_name
                        )
                    }
                }
            }
            "stop" if parts.len() == 2 => {
                let service_name = parts[1];
                let mut services_lock = services.lock().unwrap();

                match services_lock.get_mut(service_name) {
                    Some(service) => {
                        service.target = MockServiceTarget::Down;
                        if service.state == MockServiceState::Running {
                            service.state = MockServiceState::Success;
                            service.pid = 0;
                        }
                        r#"{"state":"ok","body":null}"#.to_string()
                    }
                    None => {
                        format!(
                            r#"{{"state":"error","body":"service name \"{}\" unknown"}}"#,
                            service_name
                        )
                    }
                }
            }
            "monitor" if parts.len() == 2 => {
                let service_name = parts[1];
                let services_lock = services.lock().unwrap();

                if services_lock.contains_key(service_name) {
                    format!(
                        r#"{{"state":"error","body":"service \"{}\" already monitored"}}"#,
                        service_name
                    )
                } else {
                    drop(services_lock);
                    let mut services_lock = services.lock().unwrap();

                    let service = MockService {
                        name: service_name.to_string(),
                        pid: 0,
                        state: MockServiceState::Unknown,
                        target: MockServiceTarget::Up,
                        after: HashMap::new(),
                    };

                    services_lock.insert(service_name.to_string(), service);
                    r#"{"state":"ok","body":null}"#.to_string()
                }
            }
            "forget" if parts.len() == 2 => {
                let service_name = parts[1];
                let mut services_lock = services.lock().unwrap();

                match services_lock.get(service_name) {
                    Some(service) => {
                        if service.target == MockServiceTarget::Up || service.pid != 0 {
                            format!(
                                r#"{{"state":"error","body":"service \"{}\" is up"}}"#,
                                service_name
                            )
                        } else {
                            services_lock.remove(service_name);
                            r#"{"state":"ok","body":null}"#.to_string()
                        }
                    }
                    None => {
                        format!(
                            r#"{{"state":"error","body":"service name \"{}\" unknown"}}"#,
                            service_name
                        )
                    }
                }
            }
            "kill" if parts.len() == 3 => {
                let service_name = parts[1];
                let signal = parts[2];
                let mut services_lock = services.lock().unwrap();

                match services_lock.get_mut(service_name) {
                    Some(service) => {
                        if service.pid == 0 {
                            format!(
                                r#"{{"state":"error","body":"service \"{}\" is down"}}"#,
                                service_name
                            )
                        } else {
                            // Simulate the effect of the signal
                            if signal == "SIGKILL" || signal == "SIGTERM" {
                                service.state = MockServiceState::Success;
                                service.pid = 0;
                            }
                            r#"{"state":"ok","body":null}"#.to_string()
                        }
                    }
                    None => {
                        format!(
                            r#"{{"state":"error","body":"service name \"{}\" unknown"}}"#,
                            service_name
                        )
                    }
                }
            }
            "log" => {
                // For simplicity, just return a few mock log lines
                "zinit: INFO (service1) This is a mock log message\nzinit: ERROR (service2) This is an error message\n".to_string()
            }
            _ => {
                format!(
                    r#"{{"state":"error","body":"unknown command '{}' or wrong arguments count"}}"#,
                    parts[0]
                )
            }
        }
    }
}

impl Drop for MockZinitServer {
    fn drop(&mut self) {
        // Clean up the socket file if it exists
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::time::sleep;
    use zinit_client::ZinitClient;

    #[tokio::test]
    async fn test_mock_server() {
        // Create a temporary directory for the socket
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let socket_path = temp_dir.path().join("mock-zinit.sock");

        // Create and start the mock server
        let mut server = MockZinitServer::new(&socket_path).await;
        server.start().await.expect("Failed to start mock server");

        // Add some mock services
        server.add_service(MockService {
            name: "service1".to_string(),
            pid: 1001,
            state: MockServiceState::Running,
            target: MockServiceTarget::Up,
            after: HashMap::new(),
        });

        server.add_service(MockService {
            name: "service2".to_string(),
            pid: 0,
            state: MockServiceState::Success,
            target: MockServiceTarget::Down,
            after: HashMap::new(),
        });

        // Create a client to connect to the mock server
        let client = ZinitClient::new(&socket_path);

        // Wait a moment for the server to be ready
        sleep(Duration::from_millis(100)).await;

        // Test listing services
        let services = client.list().await.expect("Failed to list services");
        assert_eq!(services.len(), 2);

        // Test getting service status
        let status = client
            .status("service1")
            .await
            .expect("Failed to get status");
        assert_eq!(status.name, "service1");
        assert_eq!(status.pid, 1001);

        // Test starting a service
        client
            .start("service2")
            .await
            .expect("Failed to start service");
        let status = client
            .status("service2")
            .await
            .expect("Failed to get status");
        assert!(status.pid > 0);

        // Test stopping a service
        client
            .stop("service1")
            .await
            .expect("Failed to stop service");
        let status = client
            .status("service1")
            .await
            .expect("Failed to get status");
        assert_eq!(status.pid, 0);

        // Stop the mock server
        server.stop().await;
    }
}
