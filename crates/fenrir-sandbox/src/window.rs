//! Window configuration and management

use serde::{Deserialize, Serialize};
use tauri::{PhysicalSize, Window};

/// Configuration for a sandbox window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window title
    pub title: String,
    /// Window width
    pub width: f64,
    /// Window height
    pub height: f64,
    /// Minimum width
    pub min_width: f64,
    /// Minimum height
    pub min_height: f64,
    /// Whether the window is resizable
    pub resizable: bool,
    /// Whether to show window decorations
    pub decorations: bool,
    /// Whether the window is always on top
    pub always_on_top: bool,
    /// Whether to enable transparency
    pub transparent: bool,
    /// Initial position (None for center)
    pub position: Option<(i32, i32)>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Fenrir Browser".to_string(),
            width: 1024.0,
            height: 768.0,
            min_width: 400.0,
            min_height: 300.0,
            resizable: true,
            decorations: true,
            always_on_top: false,
            transparent: false,
            position: None,
        }
    }
}

impl WindowConfig {
    /// Create a window config for a specific URL
    pub fn for_url(url: &url::Url) -> Self {
        let title = if let Some(host) = url.host_str() {
            format!("Fenrir - {}", host)
        } else {
            "Fenrir Browser".to_string()
        };
        
        Self {
            title,
            ..Default::default()
        }
    }
    
    /// Create a minimal window config (for popups, etc.)
    pub fn minimal() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            min_width: 300.0,
            min_height: 200.0,
            decorations: false,
            ..Default::default()
        }
    }
    
    /// Create a fullscreen window config
    pub fn fullscreen() -> Self {
        Self {
            width: 1920.0,
            height: 1080.0,
            min_width: 1920.0,
            min_height: 1080.0,
            resizable: false,
            decorations: false,
            ..Default::default()
        }
    }
}

/// A sandbox window wrapper with additional functionality
pub struct SandboxWindow {
    window: Window,
    config: WindowConfig,
}

impl SandboxWindow {
    /// Create a new sandbox window
    pub fn new(window: Window, config: WindowConfig) -> Self {
        Self { window, config }
    }
    
    /// Get the underlying Tauri window
    pub fn inner(&self) -> &Window {
        &self.window
    }
    
    /// Get the window configuration
    pub fn config(&self) -> &WindowConfig {
        &self.config
    }
    
    /// Update the window configuration
    pub fn update_config(&mut self, config: WindowConfig) {
        self.config = config;
        
        // Apply configuration changes to the window
        let _ = self.window.set_title(&self.config.title);
        let _ = self.window.set_size(PhysicalSize::new(
            self.config.width as u32,
            self.config.height as u32,
        ));
        let _ = self.window.set_min_size(Some(PhysicalSize::new(
            self.config.min_width as u32,
            self.config.min_height as u32,
        )));
        let _ = self.window.set_resizable(self.config.resizable);
        let _ = self.window.set_decorations(self.config.decorations);
        let _ = self.window.set_always_on_top(self.config.always_on_top);
    }
    
    /// Show the window
    pub fn show(&self) -> tauri::Result<()> {
        self.window.show()
    }
    
    /// Hide the window
    pub fn hide(&self) -> tauri::Result<()> {
        self.window.hide()
    }
    
    /// Close the window
    pub fn close(&self) -> tauri::Result<()> {
        self.window.close()
    }
    
    /// Set the window title
    pub fn set_title(&self, title: &str) -> tauri::Result<()> {
        self.window.set_title(title)
    }
    
    /// Set the window size
    pub fn set_size(&self, width: f64, height: f64) -> tauri::Result<()> {
        self.window.set_size(PhysicalSize::new(width as u32, height as u32))
    }
    
    /// Set whether the window is resizable
    pub fn set_resizable(&self, resizable: bool) -> tauri::Result<()> {
        self.window.set_resizable(resizable)
    }
    
    /// Set whether to show window decorations
    pub fn set_decorations(&self, decorations: bool) -> tauri::Result<()> {
        self.window.set_decorations(decorations)
    }
    
    /// Set whether the window is always on top
    pub fn set_always_on_top(&self, always_on_top: bool) -> tauri::Result<()> {
        self.window.set_always_on_top(always_on_top)
    }
}
