//! Application framework module
//!
//! Provides a component-based application framework with lifecycle management.

pub use anyhow::Error;
pub use async_trait::async_trait;
pub use log;

use tokio::sync::broadcast;
use tokio::task::JoinHandle;

/// Component trait - defines the interface for application components
#[async_trait::async_trait]
pub trait Component: Send {
    /// Startup the component
    ///
    /// Do all initialization that might fail here (e.g., bind port, connect to DB).
    /// If successful, spawn background task and return the JoinHandle.
    /// Return None if no background task is needed.
    ///
    /// The shutdown_rx parameter receives the shutdown signal from the registry.
    /// Components should monitor this channel and gracefully shutdown when a signal is received.
    ///
    /// Returns an error if startup fails - this will trigger shutdown of all previously started components.
    async fn startup(
        &mut self,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<Option<JoinHandle<()>>, Error>;
}

/// Component registry - manages component lifecycle
pub struct Registry {
    components: Vec<Box<dyn Component>>,
    handles: Vec<JoinHandle<()>>,
    shutdown_tx: broadcast::Sender<()>,
}

impl Registry {
    /// Create a new registry
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            components: Vec::new(),
            handles: Vec::new(),
            shutdown_tx,
        }
    }

    /// Register a component
    pub fn register(&mut self, component: Box<dyn Component>) {
        self.components.push(component);
    }

    /// Startup all components in registration order (STRICT MODE)
    ///
    /// In strict mode, if any component fails to start or panics:
    /// 1. All previously started components are gracefully shut down
    /// 2. Returns an error
    ///
    /// This ensures that dependencies are properly satisfied and resources are not leaked.
    pub async fn startup_all(&mut self) -> Result<(), Error> {
        let mut started_count = 0;

        for (idx, c) in self.components.iter_mut().enumerate() {
            let shutdown_rx = self.shutdown_tx.subscribe();

            // Startup component
            match c.startup(shutdown_rx).await {
                Ok(Some(handle)) => {
                    log::info!("Component {} started with background task", idx);
                    self.handles.push(handle);
                    started_count += 1;
                }
                Ok(None) => {
                    log::info!("Component {} started (no background task)", idx);
                    started_count += 1;
                }
                Err(e) => {
                    self.shutdown_all().await;
                    log::error!("Component {} failed to start: {}", idx, e);
                    return Err(e);
                }
            }
        }

        log::info!("All {} components started successfully", started_count);
        Ok(())
    }

    /// Wait for shutdown signal, then gracefully shutdown all components
    pub async fn wait_for_shutdown(&mut self) {
        // Wait for shutdown signal
        wait_for_signal().await;

        // Trigger graceful shutdown
        log::info!("Starting graceful shutdown...");
        self.shutdown_all().await;
    }

    /// Shutdown all components by broadcasting shutdown signal
    /// Then wait for all background tasks to complete (in reverse order)
    pub async fn shutdown_all(&mut self) {
        // Send shutdown signal to all components
        let _ = self.shutdown_tx.send(());

        // Wait for all handles to complete (graceful shutdown)
        // Shutdown in reverse registration order
        for handle in self.handles.drain(..).rev() {
            if let Err(e) = handle.await {
                let msg = if e.is_panic() {
                    "Task panicked"
                } else {
                    "Task cancelled"
                };
                log::warn!("{}: {:?}", msg, e);
            }
        }
    }
}

/// Wait for Ctrl+C or SIGTERM
async fn wait_for_signal() {
    #[cfg(unix)]
    {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                log::info!("Received Ctrl+C");
            }
            _ = unix_signal_shutdown() => {
                log::info!("Received SIGTERM");
            }
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        log::info!("Received Ctrl+C");
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
