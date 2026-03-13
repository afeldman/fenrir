//! fenrir-ui — Native user interface for Fenrir Browser built with Tauri
//!
//! This crate provides:
//! - Native UI components using Tauri and egui
//! - URL bar, tab management, and browser controls
//! - Integration with fenrir-sandbox for Servo instance management
//! - Event handling from Servo instances
//! - NO WebView for the UI itself – only native Rust/Tauri elements

mod components;
mod events;
mod state;
mod window;

pub use components::{UrlBar, TabBar, BrowserControls, SettingsPanel};
pub use events::{UiEvent, ServoEventHandler};
pub use state::{BrowserState, TabState, AppState};
pub use window::{MainWindow, WindowManager};

use crate::state::BrowserState;
use crate::window::WindowManager;
use fenrir_api::{ApiManager, UiEvent as ApiUiEvent};
use fenrir_sandbox::{SandboxManager, WindowConfig};
use fenrir_secure::{SecurityManager, Origin};
use std::sync::Arc;
use tauri::AppHandle;
use thiserror::Error;
use tracing::{info, error, warn};
use url::Url;

/// Main UI manager that coordinates all UI components
pub struct UiManager {
    app_handle: AppHandle,
    window_manager: WindowManager,
    sandbox_manager: Arc<SandboxManager>,
    api_manager: Arc<ApiManager>,
    security_manager: Arc<SecurityManager>,
    browser_state: BrowserState,
}

impl UiManager {
    /// Create a new UI manager
    pub fn new(
        app_handle: AppHandle,
        security_manager: Arc<SecurityManager>,
        sandbox_manager: Arc<SandboxManager>,
        api_manager: Arc<ApiManager>,
    ) -> Result<Self, UiError> {
        info!("Initializing UI manager");
        
        let window_manager = WindowManager::new(&app_handle)?;
        let browser_state = BrowserState::new();
        
        Ok(Self {
            app_handle,
            window_manager,
            sandbox_manager,
            api_manager,
            security_manager,
            browser_state,
        })
    }
    
    /// Initialize the main window and UI
    pub async fn initialize(&mut self) -> Result<(), UiError> {
        info!("Initializing main window and UI");
        
        // Create main window
        self.window_manager.create_main_window()?;
        
        // Initialize browser state
        self.browser_state.initialize().await?;
        
        // Set up event handlers
        self.setup_event_handlers()?;
        
        // Create system tray
        self.create_system_tray()?;
        
        info!("UI initialization complete");
        
        Ok(())
    }
    
    /// Handle a URL entered by the user
    pub async fn handle_url_entered(&mut self, url_str: String) -> Result<(), UiError> {
        info!("User entered URL: {}", url_str);
        
        // Parse URL
        let url = Url::parse(&url_str)
            .map_err(|e| UiError::InvalidUrl(url_str.clone(), e.to_string()))?;
        
        // Create new tab
        let tab_id = self.create_new_tab(&url).await?;
        
        // Update UI state
        self.browser_state.set_active_tab(tab_id.clone());
        
        // Update URL bar
        self.window_manager.update_url_bar(&url)?;
        
        info!("Created new tab {} for URL {}", tab_id, url);
        
        Ok(())
    }
    
    /// Create a new tab for a URL
    async fn create_new_tab(&self, url: &Url) -> Result<String, UiError> {
        // Create window config for this URL
        let window_config = WindowConfig::for_url(url);
        
        // Create Servo instance in sandboxed window
        let instance_id = self.sandbox_manager
            .create_instance(&self.app_handle, url.clone(), window_config)
            .await?;
        
        // Add tab to browser state
        let tab_id = self.browser_state.add_tab(
            url.clone(),
            instance_id.clone(),
            url.host_str().unwrap_or("New Tab").to_string(),
        );
        
        Ok(tab_id)
    }
    
    /// Handle "Mount Folder" action for current page
    pub async fn handle_mount_folder(&mut self, folder_path: String) -> Result<(), UiError> {
        info!("User requested to mount folder: {}", folder_path);
        
        // Get active tab
        let active_tab_id = self.browser_state.get_active_tab()
            .ok_or(UiError::NoActiveTab)?;
        
        let tab_state = self.browser_state.get_tab(&active_tab_id)
            .ok_or(UiError::TabNotFound(active_tab_id.clone()))?;
        
        // Get origin for current tab
        let origin = Origin::from_url(&tab_state.url)
            .ok_or_else(|| UiError::InvalidOrigin(tab_state.url.to_string()))?;
        
        // Mount directory using security manager
        self.security_manager.mount_directory(
            origin,
            std::path::PathBuf::from(folder_path),
        )?;
        
        // Show confirmation to user
        self.window_manager.show_notification(
            "Folder Mounted",
            &format!("Folder has been mounted for {}", origin),
        )?;
        
        info!("Folder mounted for origin {}", origin);
        
        Ok(())
    }
    
    /// Handle tab switching
    pub async fn handle_tab_switch(&mut self, tab_id: String) -> Result<(), UiError> {
        info!("Switching to tab: {}", tab_id);
        
        // Update browser state
        self.browser_state.set_active_tab(tab_id.clone());
        
        // Get tab state
        let tab_state = self.browser_state.get_tab(&tab_id)
            .ok_or(UiError::TabNotFound(tab_id.clone()))?;
        
        // Update URL bar
        self.window_manager.update_url_bar(&tab_state.url)?;
        
        // Focus the corresponding Servo window
        if let Some(instance) = self.sandbox_manager.get_instance(&tab_state.instance_id) {
            if let Err(e) = instance.window().set_focus() {
                warn!("Failed to focus window for instance {}: {}", tab_state.instance_id, e);
            }
        }
        
        Ok(())
    }
    
    /// Handle tab closing
    pub async fn handle_tab_close(&mut self, tab_id: String) -> Result<(), UiError> {
        info!("Closing tab: {}", tab_id);
        
        // Get tab state
        let tab_state = self.browser_state.get_tab(&tab_id)
            .ok_or(UiError::TabNotFound(tab_id.clone()))?;
        
        // Close Servo instance
        self.sandbox_manager.close_instance(&tab_state.instance_id)?;
        
        // Remove tab from state
        self.browser_state.remove_tab(&tab_id);
        
        // If this was the active tab, switch to another
        if self.browser_state.get_active_tab() == Some(&tab_id) {
            if let Some(new_active) = self.browser_state.tabs().keys().next() {
                self.handle_tab_switch(new_active.clone()).await?;
            } else {
                // No tabs left, create new empty tab
                self.handle_url_entered("about:blank".to_string()).await?;
            }
        }
        
        Ok(())
    }
    
    /// Handle browser navigation controls
    pub async fn handle_navigation(&mut self, action: NavigationAction) -> Result<(), UiError> {
        let active_tab_id = self.browser_state.get_active_tab()
            .ok_or(UiError::NoActiveTab)?;
        
        let tab_state = self.browser_state.get_tab(&active_tab_id)
            .ok_or(UiError::TabNotFound(active_tab_id.clone()))?;
        
        // Convert to API UI event
        let api_event = match action {
            NavigationAction::Back => ApiUiEvent::GoBack,
            NavigationAction::Forward => ApiUiEvent::GoForward,
            NavigationAction::Reload => ApiUiEvent::Reload,
            NavigationAction::Stop => ApiUiEvent::Stop,
        };
        
        // Send to API manager
        self.api_manager.handle_ui_event(&tab_state.instance_id, api_event).await?;
        
        Ok(())
    }
    
    /// Set up event handlers for Servo events
    fn setup_event_handlers(&self) -> Result<(), UiError> {
        // This would set up callbacks for Servo events
        // For now, it's a placeholder
        
        Ok(())
    }
    
    /// Create system tray
    fn create_system_tray(&self) -> Result<(), UiError> {
        // Create system tray with menu items
        // This is a simplified implementation
        
        Ok(())
    }
    
    /// Show settings dialog
    pub fn show_settings(&self) -> Result<(), UiError> {
        info!("Showing settings dialog");
        
        // Create and show settings window
        self.window_manager.create_settings_window()?;
        
        Ok(())
    }
    
    /// Show permissions manager
    pub fn show_permissions_manager(&self) -> Result<(), UiError> {
        info!("Showing permissions manager");
        
        // Create window to manage origin permissions
        self.window_manager.create_permissions_window()?;
        
        Ok(())
    }
}

/// Navigation actions
#[derive(Debug, Clone, Copy)]
pub enum NavigationAction {
    Back,
    Forward,
    Reload,
    Stop,
}

/// UI errors
#[derive(Error, Debug)]
pub enum UiError {
    #[error("Invalid URL '{0}': {1}")]
    InvalidUrl(String, String),
    
    #[error("Invalid origin for URL: {0}")]
    InvalidOrigin(String),
    
    #[error("No active tab")]
    NoActiveTab,
    
    #[error("Tab not found: {0}")]
    TabNotFound(String),
    
    #[error("Window error: {0}")]
    Window(String),
    
    #[error("Sandbox error: {0}")]
    Sandbox(#[from] fenrir_sandbox::SandboxError),
    
    #[error("API error: {0}")]
    Api(#[from] fenrir_api::ApiError),
    
    #[error("Security error: {0}")]
    Security(#[from] fenrir_secure::SecurityError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
}

/// Result type for UI operations
pub type UiResult<T> = Result<T, UiError>;
