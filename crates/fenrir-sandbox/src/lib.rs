//! fenrir-sandbox — Sandbox management for Servo instances in Tauri windows
//!
//! This crate provides:
//! - Management of Servo instances in isolated Tauri windows
//! - Integration with Servo's embedding APIs
//! - Lifecycle management for Servo processes
//! - Coordination between Servo's internal processes and Tauri main UI

mod instance;
mod window;

pub use instance::{ServoInstance, ServoInstanceConfig, InstanceEvent, InstanceManager};
pub use window::{SandboxWindow, WindowConfig};

use crate::instance::InstanceManager;
use fenrir_secure::{SecurityManager, Origin};
use std::sync::Arc;
use tauri::Window;
use thiserror::Error;
use tracing::{info, error};
use url::Url;

/// Main sandbox manager that coordinates Servo instances in Tauri windows
pub struct SandboxManager {
    instance_manager: InstanceManager,
    security_manager: Arc<SecurityManager>,
}

impl SandboxManager {
    /// Create a new sandbox manager
    pub fn new(security_manager: Arc<SecurityManager>) -> Self {
        Self {
            instance_manager: InstanceManager::new(),
            security_manager,
        }
    }

    /// Create a new Servo instance in a sandboxed Tauri window
    pub async fn create_instance(
        &self,
        app_handle: &tauri::AppHandle,
        url: Url,
        window_config: WindowConfig,
        event_handler: Option<Box<dyn Fn(InstanceEvent) + Send + Sync>>,
    ) -> Result<String, SandboxError> {
        // Extract origin from URL
        let origin = Origin::from_url(&url)
            .ok_or_else(|| SandboxError::InvalidUrl(url.to_string()))?;
        
        // Generate unique instance ID
        let instance_id = format!("servo-{}", uuid::Uuid::new_v4());
        
        info!("Creating new Servo instance {} for {}", instance_id, url);
        
        // Create Tauri window for this instance
        let window = self.create_sandbox_window(app_handle, &instance_id, window_config)?;
        
        // Create Servo instance
        let mut instance = ServoInstance::new(
            instance_id.clone(),
            origin.clone(),
            url,
            window,
            event_handler,
        ).await?;
        
        // TODO: Create WebView once we have a display context
        // This requires integration with the windowing system
        
        // Register instance with security manager
        self.security_manager.register_servo_instance(
            instance_id.clone(),
            origin,
            instance.servo().clone(),
        );
        
        // Register with instance manager
        self.instance_manager.register_instance(instance);
        
        info!("Servo instance {} created successfully", instance_id);
        
        Ok(instance_id)
    }

    /// Close a Servo instance and its window
    pub fn close_instance(&self, instance_id: &str) -> Result<(), SandboxError> {
        info!("Closing Servo instance {}", instance_id);
        
        // Unregister from security manager
        self.security_manager.unregister_servo_instance(instance_id);
        
        // Close instance
        self.instance_manager.close_instance(instance_id)?;
        
        info!("Servo instance {} closed", instance_id);
        
        Ok(())
    }

    /// Get a Servo instance by ID
    pub fn get_instance(&self, instance_id: &str) -> Option<ServoInstance> {
        self.instance_manager.get_instance(instance_id)
    }

    /// Get all active instances
    pub fn get_all_instances(&self) -> Vec<ServoInstance> {
        self.instance_manager.get_all_instances()
    }

    /// Navigate an instance to a new URL
    pub fn navigate_instance(&self, instance_id: &str, url: Url) -> Result<(), SandboxError> {
        let mut instance = self.instance_manager
            .get_instance_mut(instance_id)
            .ok_or(SandboxError::InstanceNotFound(instance_id.to_string()))?;
        
        // Update origin if needed
        if let Some(new_origin) = Origin::from_url(&url) {
            // Update security manager with new origin
            self.security_manager.register_servo_instance(
                instance_id.to_string(),
                new_origin,
                instance.servo().clone(),
            );
        }
        
        instance.navigate(url)
    }

    /// Reload an instance
    pub fn reload_instance(&self, instance_id: &str) -> Result<(), SandboxError> {
        let instance = self.instance_manager
            .get_instance_mut(instance_id)
            .ok_or(SandboxError::InstanceNotFound(instance_id.to_string()))?;
        
        instance.reload()
    }

    /// Go back in instance history
    pub fn go_back_instance(&self, instance_id: &str) -> Result<(), SandboxError> {
        let instance = self.instance_manager
            .get_instance_mut(instance_id)
            .ok_or(SandboxError::InstanceNotFound(instance_id.to_string()))?;
        
        instance.go_back()
    }

    /// Go forward in instance history
    pub fn go_forward_instance(&self, instance_id: &str) -> Result<(), SandboxError> {
        let instance = self.instance_manager
            .get_instance_mut(instance_id)
            .ok_or(SandboxError::InstanceNotFound(instance_id.to_string()))?;
        
        instance.go_forward()
    }

    /// Stop loading in instance
    pub fn stop_instance(&self, instance_id: &str) -> Result<(), SandboxError> {
        let instance = self.instance_manager
            .get_instance_mut(instance_id)
            .ok_or(SandboxError::InstanceNotFound(instance_id.to_string()))?;
        
        instance.stop()
    }

    /// Execute JavaScript in instance
    pub fn execute_script_instance(&self, instance_id: &str, script: &str) -> Result<(), SandboxError> {
        let instance = self.instance_manager
            .get_instance_mut(instance_id)
            .ok_or(SandboxError::InstanceNotFound(instance_id.to_string()))?;
        
        instance.execute_script(script)
    }

    /// Process events for all instances (should be called from main loop)
    pub fn process_events(&self) {
        for instance in self.instance_manager.get_all_instances() {
            instance.process_events();
        }
    }

    /// Create a sandboxed Tauri window for a Servo instance
    fn create_sandbox_window(
        &self,
        app_handle: &tauri::AppHandle,
        instance_id: &str,
        config: WindowConfig,
    ) -> Result<Window, SandboxError> {
        let window_label = format!("servo-{}", instance_id);
        
        // Create window with sandboxing features
        let window = tauri::WindowBuilder::new(
            app_handle,
            window_label,
            tauri::WindowUrl::App("index.html".into()),
        )
        .title(&config.title)
        .inner_size(config.width, config.height)
        .min_inner_size(config.min_width, config.min_height)
        .resizable(config.resizable)
        .decorations(config.decorations)
        .always_on_top(config.always_on_top)
        .visible(false) // Start hidden, show after Servo is initialized
        .build()
        .map_err(|e| SandboxError::WindowCreation(e.to_string()))?;
        
        // Configure window for sandboxing
        // Disable devtools for security in release builds
        #[cfg(debug_assertions)]
        {
            window.open_devtools();
        }
        
        Ok(window)
    }

    /// Handle events from Servo instances
    pub fn handle_instance_event(&self, instance_id: &str, event: InstanceEvent) {
        match event {
            InstanceEvent::TitleChanged(title) => {
                info!("Instance {} title changed to: {:?}", instance_id, title);
                // Update window title
                if let Some(instance) = self.instance_manager.get_instance(instance_id) {
                    if let Err(e) = instance.window().set_title(&title.unwrap_or_default()) {
                        error!("Failed to update window title: {}", e);
                    }
                }
            }
            InstanceEvent::UrlChanged(url) => {
                info!("Instance {} URL changed to: {}", instance_id, url);
                // Update security manager with new origin if needed
                if let Some(origin) = Origin::from_url(&url) {
                    if let Some(instance) = self.instance_manager.get_instance(instance_id) {
                        self.security_manager.register_servo_instance(
                            instance_id.to_string(),
                            origin,
                            instance.servo().clone(),
                        );
                    }
                }
            }
            InstanceEvent::LoadStatusChanged(status) => {
                info!("Instance {} load status: {:?}", instance_id, status);
                // Show window when loading starts
                if let Some(instance) = self.instance_manager.get_instance(instance_id) {
                    if matches!(status, servo::LoadStatus::Started) {
                        if let Err(e) = instance.window().show() {
                            error!("Failed to show window: {}", e);
                        }
                    }
                }
            }
            InstanceEvent::PermissionRequest(request) => {
                info!("Instance {} permission request", instance_id);
                // Forward to security manager
                let grant = self.security_manager
                    .handle_servo_permission_request(request, instance_id);
                info!("Permission grant result: {:?}", grant);
            }
            InstanceEvent::NavigationRequest(_request) => {
                info!("Instance {} navigation request", instance_id);
                // Handle navigation requests
                // This could be used to implement custom navigation handling
            }
            InstanceEvent::Custom(name, data) => {
                info!("Instance {} custom event: {} = {:?}", instance_id, name, data);
                // Handle custom events
            }
        }
    }
}

/// Sandbox errors
#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("Instance not found: {0}")]
    InstanceNotFound(String),
    
    #[error("Window creation failed: {0}")]
    WindowCreation(String),
    
    #[error("Servo initialization failed: {0}")]
    ServoInit(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Security error: {0}")]
    Security(#[from] fenrir_secure::SecurityError),
}

/// Result type for sandbox operations
pub type SandboxResult<T> = Result<T, SandboxError>;
