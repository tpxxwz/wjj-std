//! Application framework example
//!
//! Run with:
//! ```bash
//! cargo run --example app_framework --features app
//! ```

use wjj_std::{Registry, Component};
use async_trait::async_trait;
use tokio::task::JoinHandle;

// ========== Generic Configuration Trait ==========

/// Trait for application configuration
pub trait AppConfig: Clone + Send + Sync + 'static {
    /// Get database connection string
    fn db_url(&self) -> &str;

    /// Get HTTP server port
    fn http_port(&self) -> u16;
}

// ========== Example Configuration ==========

#[derive(Clone, Debug)]
pub struct MyConfig {
    pub db_url: String,
    pub http_port: u16,
}

impl AppConfig for MyConfig {
    fn db_url(&self) -> &str {
        &self.db_url
    }

    fn http_port(&self) -> u16 {
        self.http_port
    }
}

// ========== Example Components ==========

pub struct DatabaseComponent {
    db_url: String,
}

impl DatabaseComponent {
    pub fn new<C>(config: &C) -> Self
    where
        C: AppConfig,
    {
        Self {
            db_url: config.db_url().to_string(),
        }
    }
}

#[async_trait]
impl Component for DatabaseComponent {
    async fn startup(&mut self) -> Option<JoinHandle<()>> {
        println!("Database connecting to: {}", self.db_url);
        // Simulate database connection
        Some(tokio::spawn(async {
            println!("Database task running");
            tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
        }))
    }

    fn graceful_shutdown(&mut self) {
        println!("Database disconnecting");
    }
}

pub struct HttpServerComponent {
    port: u16,
}

impl HttpServerComponent {
    pub fn new<C>(config: &C) -> Self
    where
        C: AppConfig,
    {
        Self {
            port: config.http_port(),
        }
    }
}

#[async_trait]
impl Component for HttpServerComponent {
    async fn startup(&mut self) -> Option<JoinHandle<()>> {
        println!("HTTP server listening on port {}", self.port);
        Some(tokio::spawn(async {
            println!("HTTP server task running");
            tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
        }))
    }

    fn graceful_shutdown(&mut self) {
        println!("HTTP server shutting down");
    }
}

// ========== Generic Register Function ==========

/// Generic function to register all components
pub fn register_all<C>(config: C) -> Registry
where
    C: AppConfig,
{
    let mut registry = Registry::new();

    // Register components using generic config
    registry.register(Box::new(DatabaseComponent::new(&config)));
    registry.register(Box::new(HttpServerComponent::new(&config)));

    registry
}

// ========== Main ==========

#[tokio::main]
async fn main() {
    println!("=== WJJ-STD Application Framework Example ===\n");

    // Create configuration
    let config = MyConfig {
        db_url: "postgres://localhost/mydb".to_string(),
        http_port: 8080,
    };

    // Register all components (generic!)
    let mut registry = register_all(config);

    // Startup all components
    registry.startup_all().await;

    println!("\nAll components started. Press Ctrl+C to shutdown...\n");

    // Wait for shutdown signal
    registry.wait_all().await;

    println!("\nShutdown complete!");
}
