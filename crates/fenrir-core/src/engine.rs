//! FenrirEngine - Central browser engine orchestrator
//!
//! This module provides the main FenrirEngine struct that coordinates
//! all browser subsystems and manages the browser lifecycle.

use crate::error::{FenrirError, FenrirResult};
use crate::traits::FenrirModule;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use url::Url;
use uuid::Uuid;

/// Main browser engine that orchestrates all subsystems
pub struct FenrirEngine {
    /// Unique engine identifier
    id: Uuid,
    /// Engine configuration
    config: EngineConfig,
    /// Registered subsystems
    subsystems: HashMap<String, Arc<dyn FenrirModule>>,
    /// Active tabs
    tabs: RwLock<HashMap<Uuid, Tab>>,
    /// Engine state
    state: RwLock<EngineState>,
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Whether to enable hardware acceleration
    pub hardware_acceleration: bool,
    /// Maximum number of concurrent tabs
    pub max_tabs: usize,
    /// Default homepage URL
    pub homepage: Url,
    /// Whether to enable privacy features by default
    pub privacy_mode: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            hardware_acceleration: true,
            max_tabs: 100,
            homepage: Url::parse("https://www.duckduckgo.com").unwrap(),
            privacy_mode: true,
        }
    }
}

/// Tab representation
#[derive(Debug, Clone)]
pub struct Tab {
    /// Unique tab identifier
    pub id: Uuid,
    /// Current URL
    pub url: Url,
    /// Tab title
    pub title: String,
    /// Whether the tab is loading
    pub loading: bool,
    /// Whether the tab is active
    pub active: bool,
    /// Tab creation timestamp
    pub created_at: std::time::SystemTime,
}

/// Engine state
#[derive(Debug, Clone)]
pub enum EngineState {
    /// Engine is uninitialized
    Uninitialized,
    /// Engine is initializing subsystems
    Initializing,
    /// Engine is running
    Running,
    /// Engine is shutting down
    ShuttingDown,
    /// Engine has encountered an error
    Error(String),
}

impl FenrirEngine {
    /// Create a new FenrirEngine with default configuration
    pub fn new() -> Self {
        Self::with_config(EngineConfig::default())
    }
    
    /// Create a new FenrirEngine with custom configuration
    pub fn with_config(config: EngineConfig) -> Self {
        info!("Creating new FenrirEngine with ID: {}", Uuid::new_v4());
        
        Self {
            id: Uuid::new_v4(),
            config,
            subsystems: HashMap::new(),
            tabs: RwLock::new(HashMap::new()),
            state: RwLock::new(EngineState::Uninitialized),
        }
    }
    
    /// Initialize the engine and all subsystems
    pub async fn initialize(&mut self) -> FenrirResult<()> {
        info!("Initializing FenrirEngine {}", self.id);
        
        // Update state
        *self.state.write().await = EngineState::Initializing;
        
        // Initialize all registered subsystems
        for (name, subsystem) in &self.subsystems {
            info!("Initializing subsystem: {}", name);
            if let Err(e) = subsystem.start().await {
                error!("Failed to initialize subsystem {}: {}", name, e);
                *self.state.write().await = EngineState::Error(format!("Subsystem {} failed: {}", name, e));
                return Err(e);
            }
        }
        
        // Create initial tab with homepage
        let initial_tab = Tab {
            id: Uuid::new_v4(),
            url: self.config.homepage.clone(),
            title: "New Tab".to_string(),
            loading: false,
            active: true,
            created_at: std::time::SystemTime::now(),
        };
        
        self.tabs.write().await.insert(initial_tab.id, initial_tab);
        
        // Update state to running
        *self.state.write().await = EngineState::Running;
        
        info!("FenrirEngine {} initialized successfully", self.id);
        Ok(())
    }
    
    /// Register a subsystem with the engine
    pub fn register_subsystem(&mut self, name: String, subsystem: Arc<dyn FenrirModule>) {
        info!("Registering subsystem: {}", name);
        self.subsystems.insert(name, subsystem);
    }
    
    /// Load a URL in a new tab
    pub async fn load_url(&self, url: Url) -> FenrirResult<Uuid> {
        info!("Loading URL: {}", url);
        
        let state = self.state.read().await;
        match *state {
            EngineState::Running => {
                // Create new tab
                let tab = Tab {
                    id: Uuid::new_v4(),
                    url: url.clone(),
                    title: url.to_string(),
                    loading: true,
                    active: true,
                    created_at: std::time::SystemTime::now(),
                };
                
                let tab_id = tab.id;
                self.tabs.write().await.insert(tab_id, tab);
                
                // TODO: Actually load the URL through rendering subsystem
                info!("URL {} queued for loading in tab {}", url, tab_id);
                
                Ok(tab_id)
            }
            _ => Err(FenrirError::EngineNotRunning),
        }
    }
    
    /// Get engine ID
    pub fn id(&self) -> Uuid {
        self.id
    }
    
    /// Get engine configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }
    
    /// Get engine state
    pub async fn state(&self) -> EngineState {
        self.state.read().await.clone()
    }
    
    /// Get all tabs
    pub async fn tabs(&self) -> Vec<Tab> {
        self.tabs.read().await.values().cloned().collect()
    }
    
    /// Get active tab
    pub async fn active_tab(&self) -> Option<Tab> {
        self.tabs.read().await.values().find(|tab| tab.active).cloned()
    }
    
    /// Close a tab
    pub async fn close_tab(&self, tab_id: Uuid) -> FenrirResult<()> {
        info!("Closing tab: {}", tab_id);
        
        let mut tabs = self.tabs.write().await;
        if tabs.remove(&tab_id).is_some() {
            info!("Tab {} closed", tab_id);
            Ok(())
        } else {
            Err(FenrirError::TabNotFound(tab_id))
        }
    }
    
    /// Shutdown the engine and all subsystems
    pub async fn shutdown(&mut self) -> FenrirResult<()> {
        info!("Shutting down FenrirEngine {}", self.id);
        
        // Update state
        *self.state.write().await = EngineState::ShuttingDown;
        
        // Shutdown all subsystems
        for (name, subsystem) in &self.subsystems {
            info!("Shutting down subsystem: {}", name);
            subsystem.stop().await;
        }
        
        // Clear tabs
        self.tabs.write().await.clear();
        
        info!("FenrirEngine {} shutdown complete", self.id);
        Ok(())
    }
}

impl Drop for FenrirEngine {
    fn drop(&mut self) {
        // Try to shutdown gracefully if still running
        let runtime = tokio::runtime::Runtime::new();
        if let Ok(rt) = runtime {
            let _ = rt.block_on(self.shutdown());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::FenrirModule;
    use std::sync::atomic::{AtomicBool, Ordering};
    
    struct MockSubsystem {
        running: AtomicBool,
    }
    
    impl MockSubsystem {
        fn new() -> Self {
            Self {
                running: AtomicBool::new(false),
            }
        }
    }
    
    impl FenrirModule for MockSubsystem {
        fn name(&self) -> &'static str {
            "MockSubsystem"
        }
        
        fn start(&self) -> Pin<Box<dyn Future<Output = FenrirResult<()>> + Send>> {
            let running = self.running.clone();
            Box::pin(async move {
                running.store(true, Ordering::SeqCst);
                Ok(())
            })
        }
        
        fn stop(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
            let running = self.running.clone();
            Box::pin(async move {
                running.store(false, Ordering::SeqCst);
            })
        }
        
        fn is_running(&self) -> bool {
            self.running.load(Ordering::SeqCst)
        }
    }
    
    #[tokio::test]
    async fn test_engine_creation() {
        let engine = FenrirEngine::new();
        assert_eq!(engine.id().get_version_num(), 4); // UUID v4
    }
    
    #[tokio::test]
    async fn test_engine_initialization() {
        let mut engine = FenrirEngine::new();
        let subsystem = Arc::new(MockSubsystem::new());
        
        engine.register_subsystem("mock".to_string(), subsystem.clone());
        
        let result = engine.initialize().await;
        assert!(result.is_ok());
        
        let state = engine.state().await;
        match state {
            EngineState::Running => (),
            _ => panic!("Engine should be in Running state"),
        }
        
        assert!(subsystem.is_running());
    }
    
    #[tokio::test]
    async fn test_load_url() {
        let mut engine = FenrirEngine::new();
        engine.initialize().await.unwrap();
        
        let url = Url::parse("https://example.com").unwrap();
        let result = engine.load_url(url).await;
        
        assert!(result.is_ok());
        let tab_id = result.unwrap();
        
        let tabs = engine.tabs().await;
        assert_eq!(tabs.len(), 2); // Homepage tab + new tab
        assert!(tabs.iter().any(|t| t.id == tab_id));
    }
    
    #[tokio::test]
    async fn test_close_tab() {
        let mut engine = FenrirEngine::new();
        engine.initialize().await.unwrap();
        
        let url = Url::parse("https://example.com").unwrap();
        let tab_id = engine.load_url(url).await.unwrap();
        
        let tabs_before = engine.tabs().await;
        assert_eq!(tabs_before.len(), 2);
        
        let result = engine.close_tab(tab_id).await;
        assert!(result.is_ok());
        
        let tabs_after = engine.tabs().await;
        assert_eq!(tabs_after.len(), 1);
    }
}
