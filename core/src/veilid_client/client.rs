//! Veilid client for anonymous networking

use super::config::VeilidConfig;
use crate::Result;
use std::sync::Arc;
use veilid_core::{VeilidAPI, VeilidUpdate, RoutingContext};

/// State of the Veilid network connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VeilidState {
    /// Not initialized
    Uninitialized,

    /// Starting up
    Starting,

    /// Connected to network
    Connected,

    /// Temporarily disconnected
    Disconnected,

    /// Shutting down
    Stopping,

    /// Stopped
    Stopped,
}

/// Veilid client wrapper
pub struct VeilidClient {
    /// Client configuration
    config: VeilidConfig,

    /// Current state
    state: VeilidState,

    /// Veilid API instance (when initialized)
    api: Option<VeilidAPI>,
}

impl VeilidClient {
    /// Create a new Veilid client
    pub fn new(config: VeilidConfig) -> Self {
        Self {
            config,
            state: VeilidState::Uninitialized,
            api: None,
        }
    }

    /// Initialize and start Veilid node
    pub async fn start(&mut self) -> Result<()> {
        if self.state != VeilidState::Uninitialized {
            return Err(crate::Error::Invalid(
                "Client already initialized".to_string()
            ));
        }

        self.state = VeilidState::Starting;

        // In test mode, don't actually start Veilid
        #[cfg(test)]
        {
            self.state = VeilidState::Connected;
            tracing::info!("Veilid client started (test mode)");
            return Ok(());
        }

        #[cfg(not(test))]
        {
            // Create Veilid update callback handler
            let update_callback = Arc::new(|update: VeilidUpdate| {
                match update {
                    VeilidUpdate::Log(log) => {
                        tracing::debug!("Veilid log: {:?}", log);
                    }
                    VeilidUpdate::AppMessage(msg) => {
                        tracing::info!("Veilid app message: {:?}", msg);
                    }
                    VeilidUpdate::AppCall(call) => {
                        tracing::info!("Veilid app call: {:?}", call);
                    }
                    VeilidUpdate::Attachment(attachment) => {
                        tracing::info!("Veilid attachment state: {:?}", attachment);
                    }
                    VeilidUpdate::Network(network) => {
                        tracing::debug!("Veilid network state: {:?}", network);
                    }
                    VeilidUpdate::Config(config) => {
                        tracing::debug!("Veilid config update: {:?}", config);
                    }
                    VeilidUpdate::RouteChange(route) => {
                        tracing::info!("Veilid route change: {:?}", route);
                    }
                    VeilidUpdate::ValueChange(value) => {
                        tracing::info!("Veilid value change: {:?}", value);
                    }
                    VeilidUpdate::Shutdown => {
                        tracing::info!("Veilid shutdown");
                    }
                }
            });

            // Convert our config to Veilid config
            let veilid_config = self.config.to_veilid_config()?;

            // Initialize Veilid API with config
            let api = veilid_core::api_startup_config(update_callback, veilid_config).await?;

            // Attach to the network
            api.attach().await?;

            self.api = Some(api);
            self.state = VeilidState::Connected;

            tracing::info!("Veilid client started and attached to network");

            Ok(())
        }
    }

    /// Stop Veilid node
    pub async fn stop(&mut self) -> Result<()> {
        if self.state == VeilidState::Stopped {
            return Ok(());
        }

        self.state = VeilidState::Stopping;

        // In test mode, no actual shutdown needed
        #[cfg(not(test))]
        {
            // Shutdown actual Veilid node
            if let Some(api) = self.api.take() {
                // Detach from network first
                api.detach().await?;

                // Shutdown the API
                api.shutdown().await;
            }
        }

        self.state = VeilidState::Stopped;

        tracing::info!("Veilid client stopped");

        Ok(())
    }

    /// Get current state
    pub fn state(&self) -> VeilidState {
        self.state
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state == VeilidState::Connected
    }

    /// Get configuration
    pub fn config(&self) -> &VeilidConfig {
        &self.config
    }

    /// Create a new routing context for private routes
    pub async fn new_routing_context(&self) -> Result<RoutingContext> {
        if !self.is_connected() {
            return Err(crate::Error::Network("Not connected".to_string()));
        }

        let api = self.api.as_ref()
            .ok_or_else(|| crate::Error::Network("API not initialized".to_string()))?;

        // Create routing context with default safety (privacy) enabled
        let routing_context = api.routing_context()?
            .with_default_safety()?;

        Ok(routing_context)
    }

    /// Get the VeilidAPI instance (for advanced operations)
    pub fn api(&self) -> Option<&VeilidAPI> {
        self.api.as_ref()
    }

    /// Send data via Veilid (anonymous, encrypted)
    ///
    /// Uses app_call which expects a response
    pub async fn send_message(
        &self,
        target: &str,
        data: &[u8],
    ) -> Result<Vec<u8>> {
        if !self.is_connected() {
            return Err(crate::Error::Network("Not connected".to_string()));
        }

        let api = self.api.as_ref()
            .ok_or_else(|| crate::Error::Network("API not initialized".to_string()))?;

        // Create routing context with privacy
        let routing_context = api.routing_context()?
            .with_default_safety()?;

        // Parse target (could be a route ID or node address)
        let target = api.parse_as_target(target)?;

        // Send message and wait for response
        let response = routing_context.app_call(target, data.to_vec()).await?;

        Ok(response)
    }

    /// Send one-way message (no response expected)
    pub async fn send_message_oneway(
        &self,
        target: &str,
        data: &[u8],
    ) -> Result<()> {
        if !self.is_connected() {
            return Err(crate::Error::Network("Not connected".to_string()));
        }

        let api = self.api.as_ref()
            .ok_or_else(|| crate::Error::Network("API not initialized".to_string()))?;

        // Create routing context with privacy
        let routing_context = api.routing_context()?
            .with_default_safety()?;

        // Parse target
        let target = api.parse_as_target(target)?;

        // Send one-way message
        routing_context.app_message(target, data.to_vec()).await?;

        Ok(())
    }

    /// Receive data from Veilid
    ///
    /// TODO: Implement actual Veilid app_message handler
    pub async fn receive_message(&self) -> Result<Option<Vec<u8>>> {
        if !self.is_connected() {
            return Err(crate::Error::Network("Not connected".to_string()));
        }

        // TODO: Implement actual message receiving
        // This will be event-driven in reality

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_client_creation() {
        let tmp = TempDir::new().unwrap();
        let config = VeilidConfig::for_testing(tmp.path().to_path_buf());
        let client = VeilidClient::new(config);

        assert_eq!(client.state(), VeilidState::Uninitialized);
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_client_start_stop() {
        let tmp = TempDir::new().unwrap();
        let config = VeilidConfig::for_testing(tmp.path().to_path_buf());
        let mut client = VeilidClient::new(config);

        // Start
        client.start().await.unwrap();
        assert_eq!(client.state(), VeilidState::Connected);
        assert!(client.is_connected());

        // Stop
        client.stop().await.unwrap();
        assert_eq!(client.state(), VeilidState::Stopped);
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_cannot_start_twice() {
        let tmp = TempDir::new().unwrap();
        let config = VeilidConfig::for_testing(tmp.path().to_path_buf());
        let mut client = VeilidClient::new(config);

        client.start().await.unwrap();

        // Second start should fail
        let result = client.start().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_operations_require_connection() {
        let tmp = TempDir::new().unwrap();
        let config = VeilidConfig::for_testing(tmp.path().to_path_buf());
        let client = VeilidClient::new(config);

        // Should fail when not connected
        let result = client.send_message("test", b"data").await;
        assert!(result.is_err());

        let result = client.new_routing_context().await;
        assert!(result.is_err());
    }
}
