   # Changelog

   ## 0.3.0 (2025-01-15)

   ### Added
   - **Service CRUD Operations**: Three new methods for comprehensive service management:
     - `create_service(name, config)` - Create new services with JSON configuration
     - `get_service(name)` - Retrieve service configuration and detailed status information
     - `delete_service(name)` - Safely delete services (stops them first if running)
   - **Enhanced Mock Server**: Added support for `create`, `get`, and `delete` commands in test mock server
   - **Comprehensive Unit Tests**: 8 new integration tests covering all CRUD operations and edge cases
   - **Interactive Service Management**: Enhanced `service_management.rs` example with full CRUD operations menu
   - **Improved Documentation**: Updated README.md with service CRUD examples and usage patterns

   ### Changed
   - **Consolidated Examples**: Merged `service_crud_operations.rs` into `service_management.rs` for better organization
   - **Enhanced Error Handling**: Better error messages and handling for service CRUD operations
   - **Improved Mock Server**: More robust command parsing to handle JSON configurations properly

   ### Technical Details
   - All new methods follow async/await patterns consistent with existing API
   - Service configurations use flexible JSON format for maximum compatibility
   - PIDs are automatically assigned by the system when services start (not user-configured)
   - Comprehensive error handling with specific error types for different scenarios
   - Full backward compatibility maintained - no breaking changes

   ## 0.2.0 (2023-06-15)

   ### Added
   - Renamed the client to zinit-client

   ### Changed
   - Changed the package name to zinit-client

   ## 0.1.0 (2023-06-15)

   ### Added
   - Initial release
   - Complete API coverage for all Zinit operations
   - Robust error handling with custom error types
   - Automatic reconnection on socket errors
   - Retry mechanisms for transient failures
   - Async/await support using Tokio
   - Strongly typed service states and responses
   - Efficient log streaming
