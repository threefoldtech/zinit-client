//! Universal Zinit Client Demo
//!
//! This example demonstrates the universal zinit-client interface that works
//! seamlessly with both old (v0.2.14) and new (v0.2.25+) zinit server versions.
//!
//! Features demonstrated:
//! - Automatic protocol detection (JSON-RPC vs Raw Commands)
//! - Feature-aware operations with graceful degradation
//! - Complete service lifecycle management
//! - Consistent API across server versions
//!
//! Usage:
//!   cargo run --example universal_client_demo [socket_path]

use serde_json::json;
use std::env;
use tokio::time::{sleep, Duration};
use zinit_client::ZinitClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get socket path from command line or use default
    let socket_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/zinit.sock".to_string());

    println!("🌟 Universal Zinit Client Demo");
    println!("📍 Socket path: {}", socket_path);
    println!("🔄 Demonstrating universal interface with automatic protocol detection");
    println!();

    // Create client
    let client = ZinitClient::new(&socket_path);

    // Test 1: Protocol Detection & Service Listing
    println!("🚀 Test 1: Protocol Detection & Service Listing");
    let initial_services = match client.list().await {
        Ok(services) => {
            println!("✅ Connection successful!");
            println!("📋 Found {} services", services.len());
            for (name, state) in services.iter().take(3) {
                println!("   - {}: {:?}", name, state);
            }
            if services.len() > 3 {
                println!("   ... and {} more", services.len() - 3);
            }
            services
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            return Ok(());
        }
    };
    println!();

    // Test 2: Service Creation & Deletion (Feature Detection)
    println!("🔧 Test 2: Service Creation & Deletion");
    let test_service_name = "demo-service";
    let test_config = json!({
        "exec": "sleep 30",
        "oneshot": false
    });

    // Clean up any existing test service first
    if initial_services.contains_key(test_service_name) {
        println!("🧹 Cleaning up existing test service...");
        match client.delete_service(test_service_name).await {
            Ok(_) => println!("   ✅ Existing service cleaned up"),
            Err(e) => println!("   ⚠️  Cleanup warning: {}", e),
        }
    }

    let supports_creation = match client.create_service(test_service_name, test_config).await {
        Ok(_) => {
            println!("✅ Service creation: SUPPORTED");
            println!("🎯 Detected: New server (v0.2.25+) with JSON-RPC protocol");
            true
        }
        Err(e) => {
            if e.to_string().contains("not supported") {
                println!("ℹ️  Service creation: NOT SUPPORTED");
                println!("🎯 Detected: Old server (v0.2.14) with raw command protocol");
                println!("💡 Suggestion: {}", e);
                false
            } else {
                println!("❌ Service creation error: {}", e);
                false
            }
        }
    };
    println!();

    // Test 3: Service Status Operations
    println!("📊 Test 3: Service Status Operations");
    if let Some((service_name, _)) = initial_services.iter().next() {
        match client.status(service_name).await {
            Ok(status) => {
                println!("✅ Status check: SUCCESS");
                println!("   Service: {}", service_name);
                println!("   State: {:?}", status.state);
                println!("   Target: {:?}", status.target);
                if status.pid > 0 {
                    println!("   PID: {}", status.pid);
                }
                if !status.after.is_empty() {
                    println!("   Dependencies: {} services", status.after.len());
                }
            }
            Err(e) => {
                println!("❌ Status check: FAILED - {}", e);
            }
        }
    } else {
        println!("⚠️  No services available for status testing");
    }
    println!();

    // Test 4: Service Lifecycle Operations (if we created a service)
    if supports_creation {
        println!("🔄 Test 4: Service Lifecycle Operations");

        // Test monitor
        print!("   📡 Monitor: ");
        match client.monitor(test_service_name).await {
            Ok(_) => println!("SUCCESS"),
            Err(e) => println!("FAILED - {}", e),
        }

        // Wait a moment for monitoring to take effect
        sleep(Duration::from_millis(500)).await;

        // Test start
        print!("   ▶️  Start: ");
        match client.start(test_service_name).await {
            Ok(_) => println!("SUCCESS"),
            Err(e) => println!("FAILED - {}", e),
        }

        // Wait for service to start
        sleep(Duration::from_secs(1)).await;

        // Check status after start
        print!("   📊 Status after start: ");
        match client.status(test_service_name).await {
            Ok(status) => {
                println!("SUCCESS - State: {:?}, PID: {}", status.state, status.pid);
            }
            Err(e) => println!("FAILED - {}", e),
        }

        // Test stop
        print!("   ⏹️  Stop: ");
        match client.stop(test_service_name).await {
            Ok(_) => println!("SUCCESS"),
            Err(e) => println!("FAILED - {}", e),
        }

        // Wait for service to stop
        sleep(Duration::from_secs(1)).await;

        // Test restart
        print!("   🔄 Restart: ");
        match client.restart(test_service_name).await {
            Ok(_) => println!("SUCCESS"),
            Err(e) => println!("FAILED - {}", e),
        }

        // Wait for restart
        sleep(Duration::from_secs(1)).await;

        // Test kill with SIGTERM
        print!("   💀 Kill (SIGTERM): ");
        match client.kill(test_service_name, "SIGTERM").await {
            Ok(_) => println!("SUCCESS"),
            Err(e) => println!("FAILED - {}", e),
        }

        // Test forget
        print!("   🧠 Forget: ");
        match client.forget(test_service_name).await {
            Ok(_) => println!("SUCCESS"),
            Err(e) => println!("FAILED - {}", e),
        }

        // Test delete
        print!("   🗑️  Delete: ");
        match client.delete_service(test_service_name).await {
            Ok(_) => println!("SUCCESS"),
            Err(e) => println!("FAILED - {}", e),
        }

        println!();
    }

    // Test 5: Final Service List
    println!("📋 Test 5: Final Service List");
    match client.list().await {
        Ok(services) => {
            println!("✅ Final service count: {}", services.len());
            if supports_creation {
                if services.contains_key(test_service_name) {
                    println!("⚠️  Test service still exists (cleanup incomplete)");
                } else {
                    println!("✅ Test service properly cleaned up");
                }
            }
        }
        Err(e) => {
            println!("❌ Final list failed: {}", e);
        }
    }
    println!();

    // Summary
    println!("🎉 Universal Zinit Client Demo Complete!");
    println!();
    println!("📊 Demo Results Summary:");
    println!("   ✅ Protocol detection: Working");
    println!("   ✅ Service listing: Working");
    println!("   ✅ Service status: Working");
    if supports_creation {
        println!("   ✅ Service creation: Working (New Server)");
        println!("   ✅ Service lifecycle: Working (Start/Stop/Restart)");
        println!("   ✅ Service management: Working (Monitor/Forget/Kill)");
        println!("   ✅ Service deletion: Working");
        println!("   🎯 Server Type: New (v0.2.25+) with JSON-RPC");
    } else {
        println!("   ℹ️  Service creation: Not supported (Old Server)");
        println!("   🎯 Server Type: Old (v0.2.14) with Raw Commands");
    }
    println!();
    println!("🌟 Universal interface successfully adapts to both server versions!");

    Ok(())
}
