use crate::error::{Result, ZinitError};
use crate::models::{JsonRpcRequest, JsonRpcResponse, Protocol, Response, ResponseState};
use serde_json::Value;
use tracing::debug;

/// Handler for the Zinit protocol
#[derive(Debug, Clone)]
pub struct ProtocolHandler;

impl ProtocolHandler {
    /// Format a command for sending to Zinit
    pub fn format_command(command: &str, args: &[&str]) -> String {
        let mut cmd = command.to_string();
        for arg in args {
            cmd.push(' ');
            cmd.push_str(arg);
        }
        cmd
    }

    /// Parse a response from Zinit
    pub fn parse_response(response: &str) -> Result<Value> {
        debug!("Parsing response: {}", response);

        // For log command, return raw text
        if response.starts_with("zinit:") {
            return Ok(Value::String(response.to_string()));
        }

        // Parse JSON response
        let response: Response = serde_json::from_str(response)?;

        match response.state {
            ResponseState::Ok => Ok(response.body),
            ResponseState::Error => {
                let error_msg = match response.body {
                    Value::String(msg) => msg,
                    _ => serde_json::to_string(&response.body)
                        .unwrap_or_else(|_| "Unknown error".to_string()),
                };

                // Map error message to specific error types
                let error = if error_msg.contains("unknown") && error_msg.contains("service") {
                    let service = extract_service_name(&error_msg);
                    ZinitError::UnknownService(service)
                } else if error_msg.contains("already monitored") {
                    let service = extract_service_name(&error_msg);
                    ZinitError::ServiceAlreadyMonitored(service)
                } else if error_msg.contains("is up") {
                    let service = extract_service_name(&error_msg);
                    ZinitError::ServiceIsUp(service)
                } else if error_msg.contains("is down") || error_msg.contains("not running") {
                    let service = extract_service_name(&error_msg);
                    ZinitError::ServiceIsDown(service)
                } else if error_msg.contains("signal") {
                    let signal = error_msg
                        .split("signal")
                        .nth(1)
                        .unwrap_or("unknown")
                        .trim()
                        .to_string();
                    ZinitError::InvalidSignal(signal)
                } else if error_msg.contains("shutting down") {
                    ZinitError::ShuttingDown
                } else {
                    ZinitError::ServiceError(error_msg)
                };

                Err(error)
            }
        }
    }
}

/// Extract a service name from an error message
fn extract_service_name(error_msg: &str) -> String {
    // Try to extract service name from quotes
    if let Some(quoted) = error_msg.split('"').nth(1) {
        return quoted.to_string();
    }

    // Fallback to extracting from the message
    error_msg
        .split_whitespace()
        .find(|word| !word.starts_with('"') && !word.contains("service"))
        .unwrap_or("unknown")
        .to_string()
}

impl ProtocolHandler {
    /// Format a JSON-RPC request for new servers
    pub fn format_json_rpc_request(method: &str, params: Value, id: u64) -> Result<String> {
        let request = JsonRpcRequest::new(method, params, id);
        serde_json::to_string(&request).map_err(ZinitError::from)
    }

    /// Parse a JSON-RPC response from new servers
    pub fn parse_json_rpc_response(response: &str) -> Result<Value> {
        debug!("Parsing JSON-RPC response: {}", response);

        let rpc_response: JsonRpcResponse = serde_json::from_str(response)?;

        if let Some(error) = rpc_response.error {
            return Err(ZinitError::JsonRpcError {
                code: error.code,
                message: error.message,
                data: error.data,
            });
        }

        Ok(rpc_response.result.unwrap_or(Value::Null))
    }

    /// Format a raw command for old servers
    pub fn format_raw_command(command: &str, args: &[&str]) -> String {
        Self::format_command(command, args)
    }

    /// Parse a raw response from old servers
    pub fn parse_raw_response(response: &str) -> Result<Value> {
        Self::parse_response(response)
    }

    /// Route request formatting based on protocol
    pub fn format_request(
        protocol: Protocol,
        method: &str,
        args: &[&str],
        params: Option<Value>,
        id: u64,
    ) -> Result<String> {
        match protocol {
            Protocol::JsonRpc => {
                let params = params.unwrap_or(Value::Array(
                    args.iter().map(|s| Value::String(s.to_string())).collect(),
                ));
                Self::format_json_rpc_request(method, params, id)
            }
            Protocol::RawCommands => Ok(Self::format_raw_command(method, args)),
        }
    }

    /// Route response parsing based on protocol
    pub fn parse_response_by_protocol(protocol: Protocol, response: &str) -> Result<Value> {
        match protocol {
            Protocol::JsonRpc => Self::parse_json_rpc_response(response),
            Protocol::RawCommands => Self::parse_raw_response(response),
        }
    }
}
