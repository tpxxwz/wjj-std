//! Application framework module

pub use async_trait::async_trait;
use tokio::task::JoinHandle;

/// Component trait - defines the interface for application components
#[async_trait::async_trait]
pub trait Component: Send {
    /// Startup the component
    ///
    /// Do all initialization that might fail here (e.g., bind port, connect to DB).
    /// If successful, spawn background task and return the JoinHandle.
    /// Return None if no background task is needed.
    async fn startup(&mut self) -> Option<JoinHandle<()>>;

    /// Graceful shutdown (send shutdown signal to background task)
    fn graceful_shutdown(&mut self) {}
}

/// Component registry - manages component lifecycle
pub struct Registry {
    components: Vec<Box<dyn Component>>,
    handles: Vec<JoinHandle<()>>,
}

impl Registry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            handles: Vec::new(),
        }
    }

    /// Register a component
    pub fn register(&mut self, component: Box<dyn Component>) {
        self.components.push(component);
    }

    /// Startup all components in registration order
    /// Collects JoinHandles from components that have background tasks
    pub async fn startup_all(&mut self) {
        for c in &mut self.components {
            if let Some(handle) = c.startup().await {
                self.handles.push(handle);
            }
        }
    }

    /// Wait for shutdown signal, then gracefully shutdown all components
    pub async fn wait_for_shutdown(&mut self) {
        // Wait for shutdown signal
        wait_for_signal().await;

        // Trigger graceful shutdown
        println!("Starting graceful shutdown...");
        self.shutdown_all().await;
    }

    /// Shutdown all components in reverse registration order
    /// Then wait for all background tasks to complete
    pub async fn shutdown_all(&mut self) {
        // Shutdown in reverse order (last registered, first shutdown)
        for c in self.components.iter_mut().rev() {
            c.graceful_shutdown();
        }

        // Wait for all handles to complete (graceful shutdown)
        for handle in self.handles.drain(..) {
            let _ = handle.await;
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Wait for Ctrl+C or SIGTERM
async fn wait_for_signal() {
    #[cfg(unix)]
    {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => println!("Received Ctrl+C"),
            _ = unix_signal_shutdown() => println!("Received SIGTERM"),
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        println!("Received Ctrl+C");
    }
}

/// Listen for SIGTERM signal (Unix/Linux/macOS only)
#[cfg(unix)]
async fn unix_signal_shutdown() {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("Failed to install SIGTERM handler")
        .recv()
        .await;
}
