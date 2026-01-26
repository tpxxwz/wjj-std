//! Application framework example
//!
//! Run with:
//! ```bash
//! cargo run --example app_framework --features app
//! ```

use tokio::time::{Duration, interval};
use wjj_std::{Component, Registry, async_trait};
use wjj_std::anyhow::Result;

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
    async fn startup(
        &mut self,
        _shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<Option<tokio::task::JoinHandle<()>>, wjj_std::Error> {
        log::info!("📦 Database connecting to: {}", self.db_url);
        // Simulate database connection (synchronous, no background task needed)
        tokio::time::sleep(Duration::from_millis(100)).await;
        log::info!("✅ Database connected");
        Ok(None) // No background task for this component
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
    async fn startup(
        &mut self,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<Option<tokio::task::JoinHandle<()>>, wjj_std::Error> {
        let port = self.port;

        log::info!("🌐 HTTP server listening on port {}", port);

        // Spawn background task with graceful shutdown support
        Ok(Some(tokio::spawn(async move {
            log::info!("✅ HTTP server task running");

            let mut ticker = interval(Duration::from_secs(2));
            let mut request_count = 0;

            // Server loop - monitor both shutdown signal and work
            loop {
                tokio::select! {
                    // Check for shutdown signal
                    _ = shutdown_rx.recv() => {
                        log::info!("⏳ HTTP server received shutdown signal");
                        log::info!("🔄 Cleaning up... (processed {} requests)", request_count);
                        break;
                    }
                    // Simulate handling requests
                    _ = ticker.tick() => {
                        request_count += 1;
                        log::info!("📨 Processing request #{} on port {}", request_count, port);
                    }
                }
            }

            log::info!("🛑 HTTP server task stopped gracefully");
        }))
    }
}

pub struct HealthCheckComponent;

#[async_trait]
impl Component for HealthCheckComponent {
    async fn startup(
        &mut self,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<Option<tokio::task::JoinHandle<()>>, wjj_std::Error> {
        log::info!("💚 Health check service starting");

        Ok(Some(tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(5));

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        log::info!("💚 Health check service shutting down");
                        break;
                    }
                    _ = ticker.tick() => {
                        log::info!("💚 All systems operational");
                    }
                }
            }
        }))
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
    registry.register(Box::new(HealthCheckComponent));

    registry
}

// ========== Main ==========

#[tokio::main]
async fn main() {
    // Initialize logger for example
    env_logger::init();

    println!("╔════════════════════════════════════════════╗");
    println!("║  WJJ-STD Application Framework Example     ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Create configuration
    let config = MyConfig {
        db_url: "postgres://localhost/mydb".to_string(),
        http_port: 8080,
    };

    // Register all components (generic!)
    let mut registry = register_all(config);

    // Startup all components
    if let Err(e) = registry.startup_all().await {
        eprintln!("Failed to start components: {}", e);
        std::process::exit(1);
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  All components started. Press Ctrl+C to shutdown");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Wait for shutdown signal
    registry.wait_for_shutdown().await;

    println!("\n✅ Shutdown complete! Goodbye!");
}
