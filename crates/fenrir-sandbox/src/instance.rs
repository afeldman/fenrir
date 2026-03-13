//! Servo instance management

use crate::window::SandboxWindow;
use embedder_traits::PermissionRequest;
use fenrir_secure::Origin;
use fenrir_servo::FenrirHost;
use parking_lot::RwLock;
use servo::{LoadStatus, WebView, WebViewDelegate, Servo, ServoBuilder, WebViewBuilder};
use servo::glutin::surface::GlSurface;
use servo::glutin::display::Display;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tauri::Window;
use tracing::{info, warn, error};
use url::Url;

/// Configuration for a Servo instance
#[derive(Debug, Clone)]
pub struct ServoInstanceConfig {
    /// Whether to enable JavaScript
    pub enable_javascript: bool,
    /// Whether to enable WebGL
    pub enable_webgl: bool,
    /// Whether to enable WebGPU
    pub enable_webgpu: bool,
    /// User agent string
    pub user_agent: Option<String>,
    /// Initial window size
    pub window_size: (u32, u32),
    /// Whether to enable devtools
    pub enable_devtools: bool,
    /// Privacy settings
    pub private_browsing: bool,
}

impl Default for ServoInstanceConfig {
    fn default() -> Self {
        Self {
            enable_javascript: true,
            enable_webgl: true,
            enable_webgpu: false,
            user_agent: None,
            window_size: (1024, 768),
            enable_devtools: cfg!(debug_assertions),
            private_browsing: false,
        }
    }
}

/// A Servo instance running in a sandboxed window
pub struct ServoInstance {
    /// Unique identifier
    id: String,
    /// Origin of the instance
    origin: Origin,
    /// Current URL
    url: Url,
    /// Tauri window
    window: Window,
    /// Servo instance
    servo: Arc<Servo>,
    /// WebView for this instance
    webview: Option<WebView>,
    /// Configuration
    config: ServoInstanceConfig,
    /// Event handler for Servo events
    event_handler: Option<Box<dyn Fn(InstanceEvent) + Send + Sync>>,
}

impl ServoInstance {
    /// Create a new Servo instance
    pub async fn new(
        id: String,
        origin: Origin,
        url: Url,
        window: Window,
        event_handler: Option<Box<dyn Fn(InstanceEvent) + Send + Sync>>,
    ) -> Result<Self, crate::SandboxError> {
        info!("Initializing Servo instance {} for {}", id, url);
        
        // Create Servo instance with custom delegate
        let delegate = Rc::new(SandboxServoDelegate::new(id.clone()));
        
        let servo = ServoBuilder::default()
            .build()
            .map_err(|e| crate::SandboxError::ServoInit(e.to_string()))?;
        
        servo.set_delegate(delegate);
        
        let instance = Self {
            id,
            origin,
            url: url.clone(),
            window,
            servo: Arc::new(servo),
            webview: None,
            config: ServoInstanceConfig::default(),
            event_handler,
        };
        
        Ok(instance)
    }
    
    /// Create and initialize WebView for this instance
    pub fn create_webview(&mut self, display: &Display) -> Result<(), crate::SandboxError> {
        info!("Creating WebView for instance {}", self.id);
        
        // Create rendering context
        let rendering_context = self.create_rendering_context(display)?;
        
        // Create WebView delegate
        let webview_delegate = Rc::new(SandboxWebViewDelegate::new(
            self.id.clone(),
            self.event_handler.clone(),
        ));
        
        // Create WebView
        let webview = WebViewBuilder::new(&self.servo, rendering_context)
            .url(self.url.clone())
            .delegate(webview_delegate)
            .build()
            .map_err(|e| crate::SandboxError::ServoInit(e.to_string()))?;
        
        self.webview = Some(webview);
        
        info!("WebView created for instance {}", self.id);
        
        Ok(())
    }
    
    /// Navigate to a new URL
    pub fn navigate(&mut self, url: Url) -> Result<(), crate::SandboxError> {
        info!("Navigating instance {} to {}", self.id, url);
        
        self.url = url.clone();
        
        if let Some(webview) = &self.webview {
            webview.load(url);
        }
        
        Ok(())
    }
    
    /// Reload the current page
    pub fn reload(&self) -> Result<(), crate::SandboxError> {
        info!("Reloading instance {}", self.id);
        
        if let Some(webview) = &self.webview {
            webview.reload();
        }
        
        Ok(())
    }
    
    /// Go back in history
    pub fn go_back(&self) -> Result<(), crate::SandboxError> {
        info!("Going back in instance {}", self.id);
        
        if let Some(webview) = &self.webview {
            webview.go_back();
        }
        
        Ok(())
    }
    
    /// Go forward in history
    pub fn go_forward(&self) -> Result<(), crate::SandboxError> {
        info!("Going forward in instance {}", self.id);
        
        if let Some(webview) = &self.webview {
            webview.go_forward();
        }
        
        Ok(())
    }
    
    /// Stop loading
    pub fn stop(&self) -> Result<(), crate::SandboxError> {
        info!("Stopping load in instance {}", self.id);
        
        // Note: stop() method no longer exists in Servo API
        // We could implement alternative stopping mechanism if needed
        warn!("stop() method is not available in current Servo API");
        
        Ok(())
    }
    
    /// Execute JavaScript in the page
    pub fn execute_script(&self, script: &str) -> Result<(), crate::SandboxError> {
        info!("Executing script in instance {}", self.id);
        
        if let Some(webview) = &self.webview {
            webview.evaluate_javascript(script, |result| {
                match result {
                    Ok(value) => info!("JavaScript executed successfully: {:?}", value),
                    Err(err) => error!("JavaScript execution failed: {:?}", err),
                }
            });
        }
        
        Ok(())
    }
    
    /// Get the instance ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Get the origin
    pub fn origin(&self) -> &Origin {
        &self.origin
    }
    
    /// Get the current URL
    pub fn url(&self) -> &Url {
        &self.url
    }
    
    /// Get the Tauri window
    pub fn window(&self) -> &Window {
        &self.window
    }
    
    /// Get the Servo instance
    pub fn servo(&self) -> Arc<Servo> {
        Arc::clone(&self.servo)
    }
    
    /// Get the WebView
    pub fn webview(&self) -> Option<&WebView> {
        self.webview.as_ref()
    }
    
    /// Get the configuration
    pub fn config(&self) -> &ServoInstanceConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn update_config(&mut self, config: ServoInstanceConfig) {
        self.config = config;
    }
    
    /// Set event handler
    pub fn set_event_handler(&mut self, handler: Box<dyn Fn(InstanceEvent) + Send + Sync>) {
        self.event_handler = Some(handler);
    }
    
    /// Handle Servo events
    pub fn handle_event(&self, event: InstanceEvent) {
        if let Some(handler) = &self.event_handler {
            handler(event);
        }
    }
    
    /// Create rendering context for Servo
    fn create_rendering_context(&self, display: &Display) -> Result<Rc<dyn servo::RenderingContext>, crate::SandboxError> {
        // This is a simplified implementation
        // In a real implementation, we would create a proper GL context
        
        // For now, return a dummy context
        // TODO: Implement proper GL context creation
        Err(crate::SandboxError::ServoInit("Rendering context not implemented".to_string()))
    }
    
    /// Process Servo events (should be called regularly from main loop)
    pub fn process_events(&self) {
        self.servo.spin_event_loop();
    }
}

/// Servo delegate for sandboxed instances
struct SandboxServoDelegate {
    instance_id: String,
}

impl SandboxServoDelegate {
    fn new(instance_id: String) -> Self {
        Self { instance_id }
    }
}

impl servo::ServoDelegate for SandboxServoDelegate {
    fn notify_error(&self, err: servo::ServoError) {
        error!("Servo error in instance {}: {:?}", self.instance_id, err);
    }
    
    fn load_web_resource(&self, load: servo::WebResourceLoad) {
        info!("Servo instance {} loading resource: {}", self.instance_id, load.request().url);
        // Let Servo handle the resource loading normally
    }
    
    fn show_console_message(&self, level: servo::ConsoleLogLevel, message: String) {
        match level {
            servo::ConsoleLogLevel::Error => error!(target: "servo::console", "[{}] {}", self.instance_id, message),
            servo::ConsoleLogLevel::Warn => warn!(target: "servo::console", "[{}] {}", self.instance_id, message),
            _ => info!(target: "servo::console", "[{}] {}", self.instance_id, message),
        }
    }
}

/// WebView delegate for sandboxed instances
struct SandboxWebViewDelegate {
    instance_id: String,
    event_handler: Option<Box<dyn Fn(InstanceEvent) + Send + Sync>>,
}

impl SandboxWebViewDelegate {
    fn new(
        instance_id: String,
        event_handler: Option<Box<dyn Fn(InstanceEvent) + Send + Sync>>,
    ) -> Self {
        Self {
            instance_id,
            event_handler,
        }
    }
    
    fn emit_event(&self, event: InstanceEvent) {
        if let Some(handler) = &self.event_handler {
            handler(event);
        }
    }
}

impl WebViewDelegate for SandboxWebViewDelegate {
    fn notify_new_frame_ready(&self, _webview: WebView) {
        info!("New frame ready for instance {}", self.instance_id);
        // In a real implementation, we would trigger a redraw here
    }
    
    fn request_navigation(&self, _webview: WebView, request: servo::NavigationRequest) {
        info!("Navigation request for instance {}", self.instance_id);
        self.emit_event(InstanceEvent::NavigationRequest(request));
    }
    
    fn request_permission(&self, _webview: WebView, request: PermissionRequest) {
        info!("Permission request for instance {}", self.instance_id);
        self.emit_event(InstanceEvent::PermissionRequest(request));
    }
    
    fn notify_page_title_changed(&self, _webview: WebView, title: Option<String>) {
        info!("Title changed for instance {}: {:?}", self.instance_id, title);
        self.emit_event(InstanceEvent::TitleChanged(title));
    }
    
    fn notify_url_changed(&self, _webview: WebView, url: Url) {
        info!("URL changed for instance {}: {}", self.instance_id, url);
        self.emit_event(InstanceEvent::UrlChanged(url));
    }
    
    fn notify_load_status_changed(&self, _webview: WebView, status: LoadStatus) {
        info!("Load status changed for instance {}: {:?}", self.instance_id, status);
        self.emit_event(InstanceEvent::LoadStatusChanged(status));
    }
}

/// Events from Servo instances
#[derive(Debug)]
pub enum InstanceEvent {
    /// Page title changed
    TitleChanged(Option<String>),
    /// URL changed
    UrlChanged(Url),
    /// Load status changed
    LoadStatusChanged(LoadStatus),
    /// Navigation requested
    NavigationRequest(servo::NavigationRequest),
    /// Permission requested
    PermissionRequest(PermissionRequest),
    /// Other custom events
    Custom(String, serde_json::Value),
}

/// Manager for multiple Servo instances
pub struct InstanceManager {
    instances: RwLock<HashMap<String, ServoInstance>>,
}

impl InstanceManager {
    /// Create a new instance manager
    pub fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new instance
    pub fn register_instance(&self, instance: ServoInstance) {
        let id = instance.id().to_string();
        self.instances.write().insert(id, instance);
    }
    
    /// Get an instance by ID
    pub fn get_instance(&self, id: &str) -> Option<ServoInstance> {
        self.instances.read().get(id).cloned()
    }
    
    /// Get a mutable reference to an instance
    pub fn get_instance_mut(&self, id: &str) -> Option<ServoInstance> {
        // Note: This requires interior mutability patterns
        // For simplicity, we'll handle mutation through methods
        self.instances.read().get(id).cloned()
    }
    
    /// Get all instances
    pub fn get_all_instances(&self) -> Vec<ServoInstance> {
        self.instances.read().values().cloned().collect()
    }
    
    /// Close an instance
    pub fn close_instance(&self, id: &str) -> Result<(), crate::SandboxError> {
        if let Some(instance) = self.instances.write().remove(id) {
            // Close the window
            if let Err(e) = instance.window().close() {
                warn!("Failed to close window for instance {}: {}", id, e);
            }
            
            info!("Instance {} closed", id);
        }
        
        Ok(())
    }
    
    /// Check if an instance exists
    pub fn has_instance(&self, id: &str) -> bool {
        self.instances.read().contains_key(id)
    }
    
    /// Get the number of active instances
    pub fn instance_count(&self) -> usize {
        self.instances.read().len()
    }
}
