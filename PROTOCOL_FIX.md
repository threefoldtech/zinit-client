# Zinit Client Protocol Fix

## Issue Investigation

### Problem
The service manager fails on Linux with error: "unknown command 'create' or wrong arguments count"

### Root Cause Analysis
1. **Base zinit-client** (for_augment/zinit-client/) sends **raw text commands**
2. **Zinit server versions have different protocols:**
   - **Old servers (v0.2.14)**: Only accept **raw commands**
   - **New servers (v0.2.25+)**: Only accept **JSON-RPC calls**
3. **Mock server** accepts both formats (misleading during development)

### Evidence from Testing

#### New Server (v0.2.25)
- Raw command: `"create service_name {\"json\":\"config\"}"` → `{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}`
- JSON-RPC: `{"jsonrpc":"2.0","method":"service_create","params":["test-service",{"exec":"echo hello","oneshot":true}],"id":1}` → `{"jsonrpc":"2.0","id":1,"result":"Service 'test-service' created successfully"}`

#### Old Server (v0.2.14)
- Raw command: `"list"` → `{"state":"ok","body":{"ssh-init":"Success","sshd":"Running"}}`
- JSON-RPC: `{"jsonrpc":"2.0","method":"service_list","id":1}` → `{"state":"error","body":"unknown command..."}`
- **No `create` command**: `"create test-service {...}"` → `"unknown command 'create' or wrong arguments count"`

### Key Discovery: Service Creation Workflow

#### New Server (v0.2.25+)
1. **`service_create`**: Creates YAML config file in `/etc/zinit/`
2. **`service_monitor`**: Loads and starts monitoring the config file
3. **`service_start`**: Actually starts the service

#### Old Server (v0.2.14)
- **No dynamic creation**: Services must be pre-configured via YAML files
- **Only `monitor`**: Loads existing config files
- **No `create` command**: Dynamic service creation not supported

### Server Version Comparison

#### New Server (v0.2.25) - JSON-RPC Only
**Available via JSON-RPC:**
- `service_list` - Lists all services
- `service_create` - Creates a new service (writes YAML config)
- `service_start` - Starts a service
- `service_stop` - Stops a service
- `service_status` - Gets service status
- `service_monitor` - Starts monitoring a service (loads config)
- `service_forget` - Stops monitoring a service
- `service_kill` - Sends signal to service
- `service_delete` - Deletes service configuration
- `service_get` - Gets service configuration
- `service_stats` - Gets service statistics
- `system_shutdown` - Shuts down system
- `system_reboot` - Reboots system
- `system_start_http_server` - Starts HTTP/RPC server
- `system_stop_http_server` - Stops HTTP/RPC server
- `stream_currentLogs` - Gets current logs
- `stream_subscribeLogs` - Subscribes to logs

**Response format:** JSON-RPC standard

#### Old Server (v0.2.14) - Raw Commands Only
**Available via raw commands:**
- `list` - Lists all services
- `status <service>` - Gets service status
- `start <service>` - Starts a service
- `stop <service>` - Stops a service
- `monitor <service>` - Starts monitoring a service (requires existing config)
- `forget <service>` - Stops monitoring a service
- `kill <service> <signal>` - Sends signal to service
- `restart <service>` - Restarts a service
- `shutdown` - Shuts down system
- `reboot` - Reboots system

**NOT available:**
- `create` - No dynamic service creation
- Any JSON-RPC methods

**Response format:** `{"state":"ok/error","body":...}`

## Fix Plan

### Universal Interface Strategy
**Transform the base zinit-client into a universal interface that works with both server versions:**

The base zinit-client will become a **protocol-agnostic abstraction layer** that:
1. **Auto-detects server capabilities** on first connection
2. **Transparently handles protocol differences** (JSON-RPC vs raw commands)
3. **Provides consistent API** regardless of server version
4. **Gracefully degrades** for unsupported features with clear error messages

### Benefits
- **Maximum ecosystem impact**: Benefits SAL, service manager, and all other users
- **Zero breaking changes**: Existing code continues to work unchanged
- **Future-proof**: Can easily adapt to new zinit server versions
- **Clean architecture**: Applications don't need to handle protocol differences

### Architecture Overview
```
Application Code (SAL, Service Manager, etc.)
    ↓
ZinitClient (Universal Interface)
    ↓
Protocol Detection & Routing
    ↓
┌─────────────────┬─────────────────┐
│   JSON-RPC      │   Raw Commands  │
│  (New Server)   │  (Old Server)   │
└─────────────────┴─────────────────┘
```

### Files to Modify
1. `src/client.rs` - Add protocol detection and transparent routing
2. `src/protocol.rs` - Add dual protocol support (JSON-RPC + raw commands)
3. `src/connection.rs` - Handle both response formats
4. `src/models.rs` - Add structures for both response types
5. `src/error.rs` - Add feature compatibility errors

### Protocol Detection Logic
```rust
pub struct ZinitClient {
    connection_manager: ConnectionManager,
    protocol: OnceCell<Protocol>,
    capabilities: OnceCell<ServerCapabilities>,
}

impl ZinitClient {
    async fn detect_protocol(&self) -> Result<Protocol> {
        // Try JSON-RPC first (new servers)
        if self.try_json_rpc_call("service_list", vec![]).await.is_ok() {
            Ok(Protocol::JsonRpc)
        } else {
            // Fallback to raw commands (old servers)
            Ok(Protocol::RawCommands)
        }
    }

    async fn detect_capabilities(&self) -> Result<ServerCapabilities> {
        match self.get_protocol().await? {
            Protocol::JsonRpc => {
                // New servers support all features
                Ok(ServerCapabilities::full())
            }
            Protocol::RawCommands => {
                // Old servers don't support dynamic creation
                Ok(ServerCapabilities::legacy())
            }
        }
    }
}
```

### Method Mapping

#### New Server (JSON-RPC)
- `"create"` → `"service_create"` (creates config + monitors)
- `"start"` → `"service_start"`
- `"stop"` → `"service_stop"`
- `"status"` → `"service_status"`
- `"list"` → `"service_list"`
- `"monitor"` → `"service_monitor"`
- `"forget"` → `"service_forget"`
- `"kill"` → `"service_kill"`
- `"shutdown"` → `"system_shutdown"`
- `"reboot"` → `"system_reboot"`

#### Old Server (Raw Commands)
- `"create"` → **Error**: Feature not supported
- `"start"` → `"start <service>"`
- `"stop"` → `"stop <service>"`
- `"status"` → `"status <service>"`
- `"list"` → `"list"`
- `"monitor"` → `"monitor <service>"` (requires existing config)
- `"forget"` → `"forget <service>"`
- `"kill"` → `"kill <service> <signal>"`
- `"shutdown"` → `"shutdown"`
- `"reboot"` → `"reboot"`

### Implementation Steps

#### Phase 1: Core Infrastructure
1. **Add protocol detection mechanism** - Auto-detect server capabilities
2. **Update models.rs** - Add structures for both response formats
3. **Update error.rs** - Add feature compatibility error types
4. **Update protocol.rs** - Add dual protocol support

#### Phase 2: Protocol Handlers
5. **Implement JSON-RPC handler** - For new servers (v0.2.25+)
6. **Implement raw command handler** - For old servers (v0.2.14)
7. **Update connection.rs** - Route to appropriate protocol handler

#### Phase 3: Client Integration
8. **Update client.rs** - Add transparent protocol routing
9. **Implement capability detection** - Feature availability per server
10. **Add graceful degradation** - Clear errors for unsupported features

#### Phase 4: Testing & Validation
11. **Test with both server versions** - Ensure compatibility
12. **Update integration tests** - Cover both protocol paths
13. **Verify SAL integration** - Ensure service manager works
14. **Update mock server** - Match real server behavior

### Testing Strategy
- Test protocol detection with both server versions
- Test each method individually with both servers
- Verify feature detection works correctly
- Test graceful degradation for unsupported features
- Ensure existing integration tests pass with both protocols
- Test service manager integration with both server versions

### Response Format Examples

#### JSON-RPC (New Server)
```json
// Request
{"jsonrpc":"2.0","method":"service_create","params":["service_name",{"exec":"command","oneshot":true}],"id":1}

// Success Response
{"jsonrpc":"2.0","id":1,"result":"Service 'service_name' created successfully"}

// Error Response
{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"Service not found"}}
```

#### Raw Commands (Old Server)
```bash
# Request
list

# Success Response
{"state":"ok","body":{"ssh-init":"Success","sshd":"Running"}}

# Error Response
{"state":"error","body":"service name \"unknown\" unknown"}
```

### Universal Interface Benefits

#### For Application Developers
```rust
// Same code works with both server versions
let client = ZinitClient::new("/tmp/zinit.sock");

// Works on both old and new servers
let services = client.list().await?;

// Graceful degradation for unsupported features
match client.create_service("test", config).await {
    Ok(_) => println!("Service created"),
    Err(ZinitError::FeatureNotSupported(msg)) => {
        println!("Feature not available: {}", msg);
        // Fallback to manual config file creation
    }
    Err(e) => return Err(e),
}
```

#### For SAL Integration
- **Zero code changes**: SAL's zinit-client wrapper continues to work
- **Automatic compatibility**: Service manager works with both server versions
- **Future-proof**: New zinit server versions handled transparently

### Compatibility Matrix
| Operation | Old Server (v0.2.14) | New Server (v0.2.25+) | Client Behavior |
|-----------|---------------------|----------------------|-----------------|
| `list()` | ✅ Raw command | ✅ JSON-RPC | Auto-detects protocol |
| `start()` | ✅ Raw command | ✅ JSON-RPC | Works transparently |
| `stop()` | ✅ Raw command | ✅ JSON-RPC | Works transparently |
| `status()` | ✅ Raw command | ✅ JSON-RPC | Works transparently |
| `create_service()` | ❌ Not supported | ✅ JSON-RPC | Graceful error on old |
| `monitor()` | ✅ Raw command | ✅ JSON-RPC | Works transparently |

## Implementation Status
- [x] Document issue and fix plan ✅
- [x] Investigate both server versions ✅
- [x] Understand service creation workflow ✅
- [x] Design universal interface architecture ✅
- [ ] **Phase 1**: Core infrastructure (protocol detection, models, errors)
- [ ] **Phase 2**: Protocol handlers (JSON-RPC + raw commands)
- [ ] **Phase 3**: Client integration (transparent routing)
- [ ] **Phase 4**: Testing & validation (both server versions)
- [ ] Verify SAL service manager works with both versions
