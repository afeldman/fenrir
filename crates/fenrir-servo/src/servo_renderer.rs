//! ServoRenderer - Servo-based implementation of the Renderer trait

use fenrir_core::error::FenrirError;
use fenrir_rendering::renderer::{Renderer, DisplayList, RendererCapabilities, RendererApi};
use fenrir_rendering::RenderingError;
use servo::{Servo, ServoBuilder, WebView, WebViewBuilder};
use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{info, error};
use url::Url;
use euclid::default::Size2D;

/// Servo-based renderer that implements the Renderer trait
pub struct ServoRenderer {
    /// Underlying Servo instance
    servo: Servo,
    /// Waker for event loop
    waker_notify: Arc<Notify>,
    /// Whether the renderer is initialized
    initialized: bool,
    /// Current size
    size: Size2D<u32>,
    /// Renderer capabilities
    capabilities: RendererCapabilities,
}

impl ServoRenderer {
    /// Create a new ServoRenderer
    pub fn new() -> Result<Self, FenrirError> {
        let waker_notify = Arc::new(Notify::new());
        let waker = crate::FenrirEventLoopWaker::new(Arc::clone(&waker_notify));
        
        info!("Creating ServoRenderer");
        
        let servo = ServoBuilder::default()
            .event_loop_waker(Box::new(waker))
            .build();
        
        // Set a default delegate (simplified for now)
        // In a real implementation, we'd use FenrirServoDelegate
        
        Ok(Self {
            servo,
            waker_notify,
            initialized: false,
            size: Size2D::new(0, 0),
            capabilities: RendererCapabilities {
                hardware_acceleration: true,
                max_texture_size: 16384,
                multithreaded: true,
                supported_apis: vec![RendererApi::WebRender],
            },
        })
    }
    
    /// Load a URL in the renderer
    pub fn load_url(&mut self, url: Url, rendering_context: Rc<dyn servo::RenderingContext>) -> Result<(), FenrirError> {
        info!("Loading URL in ServoRenderer: {}", url);
        
        if !self.initialized {
            return Err(FenrirError::ServoInit("Renderer not initialized".to_string()));
        }
        
        // Create a new WebView for the URL
        let _webview = WebViewBuilder::new(&self.servo, rendering_context)
            .url(url)
            .build();
        
        info!("URL loaded successfully");
        
        Ok(())
    }
    
    /// Get a reference to the underlying Servo instance
    pub fn servo(&self) -> &Servo {
        &self.servo
    }
    
    /// Get a mutable reference to the underlying Servo instance
    pub fn servo_mut(&mut self) -> &mut Servo {
        &mut self.servo
    }
    
    /// Process Servo events (should be called from event loop)
    pub fn spin_event_loop(&self) {
        self.servo.spin_event_loop();
    }
    
    /// Wait for Servo to signal work
    pub async fn wait_for_work(&self) {
        self.waker_notify.notified().await;
    }
}

impl Renderer for ServoRenderer {
    fn init(&mut self) -> Result<(), RenderingError> {
        info!("Initializing ServoRenderer");
        
        // Note: Servo initialization is done in new()
        // Additional initialization would go here
        
        self.initialized = true;
        info!("ServoRenderer initialized successfully");
        
        Ok(())
    }
    
    fn render_frame(&mut self, _display_list: &DisplayList) -> Result<(), RenderingError> {
        if !self.initialized {
            return Err(RenderingError::Renderer("Renderer not initialized".to_string()));
        }
        
        // Servo handles its own rendering internally
        // The display list is not used directly by Servo
        // Instead, Servo renders HTML/CSS content
        
        // Process any pending events
        self.spin_event_loop();
        
        // In a real implementation, we'd check if a new frame is ready
        // and composite it with the display list
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "ServoRenderer"
    }
    
    fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "html" => true,
            "css" => true,
            "javascript" => true,
            "webgl" => true,
            "webrtc" => true,
            "hardware_acceleration" => self.capabilities.hardware_acceleration,
            "multithreaded" => self.capabilities.multithreaded,
            _ => false,
        }
    }
    
    fn capabilities(&self) -> RendererCapabilities {
        self.capabilities.clone()
    }
    
    fn resize(&mut self, size: Size2D<u32>) -> Result<(), RenderingError> {
        self.size = size;
        
        // Note: WebView doesn't have a resize method in the public API
        // In a real implementation, we'd need to handle this differently
        info!("Would resize to {}x{}", size.width, size.height);
        
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Default for ServoRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create ServoRenderer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fenrir_rendering::renderer::Renderer;
    
    #[test]
    fn test_servo_renderer_creation() {
        let renderer = ServoRenderer::new();
        assert!(renderer.is_ok());
        
        let mut renderer = renderer.unwrap();
        assert_eq!(renderer.name(), "ServoRenderer");
        
        let capabilities = renderer.capabilities();
        assert!(capabilities.hardware_acceleration);
        assert!(capabilities.multithreaded);
        assert!(capabilities.supported_apis.contains(&RendererApi::WebRender));
    }
    
    #[test]
    fn test_servo_renderer_features() {
        let renderer = ServoRenderer::new().unwrap();
        
        assert!(renderer.supports_feature("html"));
        assert!(renderer.supports_feature("css"));
        assert!(renderer.supports_feature("javascript"));
        assert!(renderer.supports_feature("webgl"));
        assert!(renderer.supports_feature("hardware_acceleration"));
        assert!(!renderer.supports_feature("nonexistent"));
    }
    
    #[test]
    fn test_renderer_trait_implementation() {
        let mut renderer = ServoRenderer::new().unwrap();
        
        // Test initialization
        let init_result = renderer.init();
        assert!(init_result.is_ok());
        
        // Test rendering (should succeed even without a WebView)
        let display_list = DisplayList::new();
        let render_result = renderer.render_frame(&display_list);
        assert!(render_result.is_ok());
        
        // Test downcasting
        let any_ref = renderer.as_any();
        assert!(any_ref.is::<ServoRenderer>());
    }
}
