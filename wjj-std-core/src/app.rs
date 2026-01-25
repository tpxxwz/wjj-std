//! Application framework module
//!
//! Provides a component-based application framework with lifecycle management.

use tokio::signal;

/// Component trait - defines the interface for application components
#[async_trait::async_trait]
pub trait Component {
    /// Startup the component
    async fn startup(&mut self);

    /// Graceful shutdown (call framework-specific graceful shutdown API)
    fn graceful_shutdown(&mut self) {}
}

/// Component registry - manages component lifecycle
pub struct Registry {
    components: Vec<Box<dyn Component>>,
}

impl Registry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    /// Register a component
    pub fn register(&mut self, component: Box<dyn Component>) {
        self.components.push(component);
    }

    /// Startup all components in registration order
    pub async fn startup_all(&mut self) {
        for c in &mut self.components {
            c.startup().await;
        }
    }

    /// Wait for shutdown signal
    pub async fn wait_for_shutdown(&mut self) {
        // Wait for shutdown signal
        #[cfg(unix)]
        {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => println!("Received Ctrl+C"),
                _ = unix_signal_shutdown() => println!("Received SIGTERM"),
            }
        }

        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c().await;
            println!("Received Ctrl+C");
        }

        // Trigger graceful shutdown
        println!("Starting graceful shutdown...");
        self.shutdown_all();
    }

    /// Shutdown all components in reverse registration order
    pub fn shutdown_all(&mut self) {
        // Shutdown in reverse order (last registered, first shutdown)
        for c in self.components.iter_mut().rev() {
            c.graceful_shutdown();
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Listen for SIGTERM signal (Unix/Linux/macOS only)
#[cfg(unix)]
async fn unix_signal_shutdown() {
    signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("Failed to install SIGTERM handler")
        .recv()
        .await;
}
