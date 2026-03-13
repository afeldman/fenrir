//! Window management

use tauri::{AppHandle, Window, WindowBuilder, WindowUrl};
use thiserror::Error;
use tracing::{info, error};

/// Manages all application windows
pub struct WindowManager {
    app_handle: AppHandle,
    main_window: Option<Window>,
    settings_window: Option<Window>,
    permissions_window: Option<Window>,
}

impl WindowManager {
    /// Create new window manager
    pub fn new(app_handle: &AppHandle) -> Result<Self, WindowError> {
        Ok(Self {
            app_handle: app_handle.clone(),
            main_window: None,
            settings_window: None,
            permissions_window: None,
        })
    }
    
    /// Create main browser window
    pub fn create_main_window(&mut self) -> Result<(), WindowError> {
        info!("Creating main browser window");
        
        if self.main_window.is_some() {
            return Err(WindowError::WindowAlreadyExists("main".to_string()));
        }
        
        let window = WindowBuilder::new(
            &self.app_handle,
            "main",
            WindowUrl::App("index.html".into()),
        )
        .title("Fenrir Browser")
        .inner_size(1200.0, 800.0)
        .min_inner_size(400.0, 300.0)
        .resizable(true)
        .decorations(true)
        .visible(true)
        .build()
        .map_err(|e| WindowError::Creation(e.to_string()))?;
        
        self.main_window = Some(window);
        
        info!("Main window created");
        
        Ok(())
    }
    
    /// Create settings window
    pub fn create_settings_window(&mut self) -> Result<(), WindowError> {
        info!("Creating settings window");
        
        if self.settings_window.is_some() {
            // Focus existing window
            if let Some(window) = &self.settings_window {
                let _ = window.set_focus();
            }
            return Ok(());
        }
        
        let window = WindowBuilder::new(
            &self.app_handle,
            "settings",
            WindowUrl::App("settings.html".into()),
        )
        .title("Fenrir Settings")
        .inner_size(800.0, 600.0)
        .min_inner_size(600.0, 400.0)
        .resizable(true)
        .decorations(true)
        .visible(true)
        .build()
        .map_err(|e| WindowError::Creation(e.to_string()))?;
        
        self.settings_window = Some(window);
        
        info!("Settings window created");
        
        Ok(())
    }
    
    /// Create permissions window
    pub fn create_permissions_window(&mut self) -> Result<(), WindowError> {
        info!("Creating permissions window");
        
        if self.permissions_window.is_some() {
            // Focus existing window
            if let Some(window) = &self.permissions_window {
                let _ = window.set_focus();
            }
            return Ok(());
        }
        
        let window = WindowBuilder::new(
            &self.app_handle,
            "permissions",
            WindowUrl::App("permissions.html".into()),
        )
        .title("Fenrir Permissions")
        .inner_size(600.0, 400.0)
        .min_inner_size(400.0, 300.0)
        .resizable(true)
        .decorations(true)
        .visible(true)
        .build()
        .map_err(|e| WindowError::Creation(e.to_string()))?;
        
        self.permissions_window = Some(window);
        
        info!("Permissions window created");
        
        Ok(())
    }
    
    /// Get main window
    pub fn main_window(&self) -> Option<&Window> {
        self.main_window.as_ref()
    }
    
    /// Update URL bar in main window
    pub fn update_url_bar(&self, url: &url::Url) -> Result<(), WindowError> {
        if let Some(window) = &self.main_window {
            // Send message to update URL bar
            // This would typically use Tauri's event system
            let _ = window.emit("url-updated", url.to_string());
        }
        
        Ok(())
    }
    
    /// Show notification
    pub fn show_notification(&self, title: &str, message: &str) -> Result<(), WindowError> {
        // Use Tauri's notification API
        tauri::api::notification::Notification::new(&self.app_handle.config().tauri.bundle.identifier)
            .title(title)
            .body(message)
            .show()
            .map_err(|e| WindowError::Notification(e.to_string()))?;
        
        Ok(())
    }
    
    /// Close settings window
    pub fn close_settings_window(&mut self) -> Result<(), WindowError> {
        if let Some(window) = self.settings_window.take() {
            window.close().map_err(|e| WindowError::Close(e.to_string()))?;
        }
        
        Ok(())
    }
    
    /// Close permissions window
    pub fn close_permissions_window(&mut self) -> Result<(), WindowError> {
        if let Some(window) = self.permissions_window.take() {
            window.close().map_err(|e| WindowError::Close(e.to_string()))?;
        }
        
        Ok(())
    }
    
    /// Close all windows
    pub fn close_all_windows(&mut self) -> Result<(), WindowError> {
        self.close_settings_window()?;
        self.close_permissions_window()?;
        
        Ok(())
    }
}

/// Main browser window wrapper
pub struct MainWindow {
    window: Window,
}

impl MainWindow {
    /// Create new main window wrapper
    pub fn new(window: Window) -> Self {
        Self { window }
    }
    
    /// Get underlying window
    pub fn inner(&self) -> &Window {
        &self.window
    }
    
    /// Set window title
    pub fn set_title(&self, title: &str) -> Result<(), WindowError> {
        self.window.set_title(title)
            .map_err(|e| WindowError::Update(e.to_string()))
    }
    
    /// Set window size
    pub fn set_size(&self, width: f64, height: f64) -> Result<(), WindowError> {
        self.window.set_size(tauri::PhysicalSize::new(width as u32, height as u32))
            .map_err(|e| WindowError::Update(e.to_string()))
    }
    
    /// Maximize window
    pub fn maximize(&self) -> Result<(), WindowError> {
        self.window.maximize()
            .map_err(|e| WindowError::Update(e.to_string()))
    }
    
    /// Unmaximize window
    pub fn unmaximize(&self) -> Result<(), WindowError> {
        self.window.unmaximize()
            .map_err(|e| WindowError::Update(e.to_string()))
    }
    
    /// Minimize window
    pub fn minimize(&self) -> Result<(), WindowError> {
        self.window.minimize()
            .map_err(|e| WindowError::Update(e.to_string()))
    }
    
    /// Show window
    pub fn show(&self) -> Result<(), WindowError> {
        self.window.show()
            .map_err(|e| WindowError::Update(e.to_string()))
    }
    
    /// Hide window
    pub fn hide(&self) -> Result<(), WindowError> {
        self.window.hide()
            .map_err(|e| WindowError::Update(e.to_string()))
    }
    
    /// Close window
    pub fn close(&self) -> Result<(), WindowError> {
        self.window.close()
            .map_err(|e| WindowError::Close(e.to_string()))
    }
}

/// Window errors
#[derive(Error, Debug)]
pub enum WindowError {
    #[error("Window creation failed: {0}")]
    Creation(String),
    
    #[error("Window already exists: {0}")]
    WindowAlreadyExists(String),
    
    #[error("Window update failed: {0}")]
    Update(String),
    
    #[error("Window close failed: {0}")]
    Close(String),
    
    #[error("Notification failed: {0}")]
    Notification(String),
    
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
}
