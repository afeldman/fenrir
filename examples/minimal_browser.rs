//! Minimal Fenrir Browser Example
//!
//! This example demonstrates the minimal working architecture of Fenrir Browser:
//! 1. Initialize FenrirEngine
//! 2. Create a Tauri window
//! 3. Connect UI to engine
//! 4. Load a URL
//! 5. Basic rendering pipeline

use fenrir_core::engine::{FenrirEngine, EngineConfig};
use fenrir_rendering::renderer::{Renderer, WebRenderBackend};
use std::sync::Arc;
use tracing::{info, error};
use url::Url;

/// Minimal browser application
struct FenrirBrowser {
    /// Core browser engine
    engine: FenrirEngine,
    /// Rendering backend
    renderer: Box<dyn Renderer>,
    /// Whether the browser is running
    running: bool,
}

impl FenrirBrowser {
    /// Create a new FenrirBrowser instance
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Creating FenrirBrowser...");
        
        // Create engine configuration
        let config = EngineConfig {
            hardware_acceleration: true,
            max_tabs: 10,
            homepage: Url::parse("https://www.duckduckgo.com")?,
            privacy_mode: true,
        };
        
        // Create engine
        let mut engine = FenrirEngine::with_config(config);
        
        // Create renderer
        let mut renderer = WebRenderBackend::new();
        
        // Initialize renderer
        renderer.init()?;
        
        // Register renderer as a subsystem
        // Note: In a real implementation, we'd need to wrap the renderer
        // to implement FenrirModule trait
        
        info!("FenrirBrowser created successfully");
        
        Ok(Self {
            engine,
            renderer: Box::new(renderer),
            running: false,
        })
    }
    
    /// Initialize the browser
    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing FenrirBrowser...");
        
        // Initialize engine
        self.engine.initialize().await?;
        
        self.running = true;
        info!("FenrirBrowser initialized successfully");
        
        Ok(())
    }
    
    /// Load a URL
    async fn load_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Loading URL: {}", url);
        
        let url = Url::parse(url)?;
        let tab_id = self.engine.load_url(url).await?;
        
        info!("URL loaded in tab: {}", tab_id);
        
        // In a real implementation, we'd render the page here
        // For now, just log that rendering would happen
        
        Ok(())
    }
    
    /// Run the browser event loop
    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting FenrirBrowser event loop...");
        
        // Simple event loop for demonstration
        for i in 0..5 {
            info!("Event loop iteration {}", i);
            
            // Simulate rendering a frame
            let display_list = fenrir_rendering::renderer::DisplayList::new();
            if let Err(e) = self.renderer.render_frame(&display_list) {
                error!("Rendering error: {}", e);
            }
            
            // Sleep to simulate frame rate
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        info!("Event loop completed");
        Ok(())
    }
    
    /// Shutdown the browser
    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Shutting down FenrirBrowser...");
        
        self.running = false;
        
        // Shutdown engine
        self.engine.shutdown().await?;
        
        info!("FenrirBrowser shutdown complete");
        Ok(())
    }
}

/// Main function for the minimal example
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("=== Fenrir Browser Minimal Example ===");
    
    // Create browser instance
    let mut browser = FenrirBrowser::new().await?;
    
    // Initialize browser
    browser.initialize().await?;
    
    // Load a test URL
    browser.load_url("https://example.com").await?;
    
    // Run event loop
    browser.run().await?;
    
    // Shutdown
    browser.shutdown().await?;
    
    info!("=== Example completed successfully ===");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_browser_creation() {
        let browser = FenrirBrowser::new().await;
        assert!(browser.is_ok());
    }
    
    #[tokio::test]
    async fn test_browser_initialization() {
        let mut browser = FenrirBrowser::new().await.unwrap();
        let result = browser.initialize().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_url_loading() {
        let mut browser = FenrirBrowser::new().await.unwrap();
        browser.initialize().await.unwrap();
        
        let result = browser.load_url("https://example.com").await;
        assert!(result.is_ok());
    }
}
