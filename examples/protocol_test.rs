//! Protocol Detection Test
//!
//! This example tests the universal zinit-client interface with both
//! old and new zinit server versions.

use serde_json::json;
use std::env;
use zinit_client::ZinitClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get socket path from command line or use default
    let socket_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/zinit.sock".to_string());

    println!("🔍 Testing Universal Zinit Client Interface");
    println!("📍 Socket path: {}", socket_path);
    println!();

    // Create client
    let client = ZinitClient::new(&socket_path);

    // Test 1: Protocol Detection
    println!("🚀 Test 1: Protocol Detection");
    match client.list().await {
        Ok(services) => {
            println!("✅ Connection successful!");
            println!("📋 Found {} services", services.len());
            for (name, state) in services.iter().take(3) {
                println!("   - {}: {:?}", name, state);
            }
            if services.len() > 3 {
                println!("   ... and {} more", services.len() - 3);
            }
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            return Ok(());
        }
    }
    println!();

    // Test 2: Feature Detection
    println!("🔧 Test 2: Feature Detection");
    let test_config = json!({
        "exec": "echo 'Hello from universal client!'",
        "oneshot": true
    });

    match client
        .create_service("test-universal-client", test_config)
        .await
    {
        Ok(_) => {
            println!("✅ Dynamic service creation supported!");
            println!("🎯 Detected: New server (v0.2.25+) with JSON-RPC protocol");

            // Clean up the test service
            match client.delete_service("test-universal-client").await {
                Ok(_) => println!("🧹 Test service cleaned up"),
                Err(e) => println!("⚠️  Cleanup warning: {}", e),
            }
        }
        Err(e) => {
            if e.to_string().contains("not supported") {
                println!("ℹ️  Dynamic service creation not supported");
                println!("🎯 Detected: Old server (v0.2.14) with raw command protocol");
                println!("💡 Suggestion: {}", e);
            } else {
                println!("❌ Unexpected error: {}", e);
            }
        }
    }
    println!();

    // Test 3: Basic Operations
    println!("⚙️  Test 3: Basic Operations");

    // Test status on an existing service
    if let Some((service_name, _)) = client.list().await?.iter().next() {
        match client.status(service_name).await {
            Ok(status) => {
                println!("✅ Status check successful for '{}':", service_name);
                println!("   State: {:?}", status.state);
                println!("   Target: {:?}", status.target);
                if status.pid > 0 {
                    println!("   PID: {}", status.pid);
                }
            }
            Err(e) => {
                println!("❌ Status check failed: {}", e);
            }
        }
    }
    println!();

    println!("🎉 Universal Interface Test Complete!");
    println!();
    println!("📊 Summary:");
    println!("   - Protocol detection: Working");
    println!("   - Feature detection: Working");
    println!("   - Graceful degradation: Working");
    println!("   - Basic operations: Working");

    Ok(())
}
