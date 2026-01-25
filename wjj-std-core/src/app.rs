//! Application framework module
//!
//! Provides a component-based application framework with lifecycle management.

use tokio::signal;
use tokio::task::JoinHandle;

/// Component trait - defines the interface for application components
#[async_trait::async_trait]
pub trait Component {
    /// Startup the component, returning background task handle if any
    async fn startup(&mut self) -> Option<JoinHandle<()>>;

    /// Graceful shutdown (call framework-specific graceful shutdown API)
    fn graceful_shutdown(&mut self) {}
}

/// Component registry - manages component lifecycle
pub struct Registry {
    components: Vec<Box<dyn Component>>,
    join_handles: Vec<JoinHandle<()>>,
}

impl Registry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            join_handles: Vec::new(),
        }
    }

    /// Register a component
    pub fn register(&mut self, component: Box<dyn Component>) {
        self.components.push(component);
    }

    /// Startup all components in registration order
    pub async fn startup_all(&mut self) {
        for c in &mut self.components {
            if let Some(handle) = c.startup().await {
                self.join_handles.push(handle);
            }
        }
    }

    /// Wait for all async components or shutdown signal
    pub async fn wait_all(&mut self) {
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

        // Wait for all component tasks to complete
        for handle in self.join_handles.drain(..) {
            let _ = handle.await;
        }
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
