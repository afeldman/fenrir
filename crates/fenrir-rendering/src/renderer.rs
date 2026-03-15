//! Renderer abstraction layer for Fenrir Browser
//!
//! This module provides the core rendering abstraction that allows
//! different rendering backends (Servo, WebRender, custom Vulkan, etc.)
//! to be used interchangeably.

use crate::RenderingError;
use euclid::default::{Point2D, Rect, Size2D};
use std::any::Any;
use std::sync::Arc;

/// Main renderer trait that all rendering backends must implement
pub trait Renderer: Send {
    /// Initialize the renderer
    fn init(&mut self) -> Result<(), RenderingError>;
    
    /// Render a frame using the provided display list
    fn render_frame(&mut self, display_list: &DisplayList) -> Result<(), RenderingError>;
    
    /// Get the renderer's name for debugging
    fn name(&self) -> &'static str;
    
    /// Check if the renderer supports a specific feature
    fn supports_feature(&self, feature: &str) -> bool;
    
    /// Get the renderer's capabilities
    fn capabilities(&self) -> RendererCapabilities;
    
    /// Resize the rendering surface
    fn resize(&mut self, size: Size2D<u32>) -> Result<(), RenderingError>;
    
    /// Get the underlying renderer as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Display list abstraction for rendering commands
#[derive(Debug, Clone)]
pub struct DisplayList {
    /// List of rendering commands
    pub commands: Vec<RenderingCommand>,
    /// Bounding rectangle of the display list
    pub bounds: Rect<f32>,
    /// Whether the display list is empty
    pub is_empty: bool,
}

impl DisplayList {
    /// Create a new empty display list
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            bounds: Rect::new(Point2D::new(0.0, 0.0), Size2D::new(0.0, 0.0)),
            is_empty: true,
        }
    }
    
    /// Add a rendering command to the display list
    pub fn add_command(&mut self, command: RenderingCommand) {
        self.commands.push(command);
        self.is_empty = false;
        // Update bounds based on command
        // This is a simplified implementation
    }
    
    /// Clear the display list
    pub fn clear(&mut self) {
        self.commands.clear();
        self.is_empty = true;
        self.bounds = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(0.0, 0.0));
    }
}

impl Default for DisplayList {
    fn default() -> Self {
        Self::new()
    }
}

/// Rendering commands that can be added to a display list
#[derive(Debug, Clone)]
pub enum RenderingCommand {
    /// Draw a rectangle
    DrawRect {
        rect: Rect<f32>,
        color: [f32; 4],
    },
    /// Draw text
    DrawText {
        position: Point2D<f32>,
        text: String,
        color: [f32; 4],
        font_size: f32,
    },
    /// Draw an image
    DrawImage {
        rect: Rect<f32>,
        image_data: Vec<u8>,
        image_format: ImageFormat,
    },
    /// Apply a transformation
    Transform {
        matrix: [f32; 16],
    },
    /// Set clipping rectangle
    Clip {
        rect: Rect<f32>,
    },
}

/// Image formats supported by the renderer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// RGBA format (8 bits per channel)
    Rgba8,
    /// RGB format (8 bits per channel)
    Rgb8,
    /// Grayscale format (8 bits)
    Gray8,
}

/// Renderer capabilities
#[derive(Debug, Clone)]
pub struct RendererCapabilities {
    /// Whether the renderer supports hardware acceleration
    pub hardware_acceleration: bool,
    /// Maximum texture size
    pub max_texture_size: u32,
    /// Whether the renderer supports multithreaded rendering
    pub multithreaded: bool,
    /// Supported rendering APIs
    pub supported_apis: Vec<RendererApi>,
}

impl Default for RendererCapabilities {
    fn default() -> Self {
        Self {
            hardware_acceleration: true,
            max_texture_size: 8192, // Common maximum
            multithreaded: true,
            supported_apis: vec![RendererApi::WebRender],
        }
    }
}

/// Rendering APIs supported by Fenrir
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererApi {
    /// Servo's WebRender
    WebRender,
    /// Custom Vulkan renderer
    Vulkan,
    /// Custom Metal renderer (macOS/iOS)
    Metal,
    /// Custom DirectX 12 renderer (Windows)
    DirectX12,
    /// Software fallback renderer
    Software,
}

/// WebRender backend implementation
pub struct WebRenderBackend {
    /// Whether the backend is initialized
    initialized: bool,
    /// Current surface size
    size: Size2D<u32>,
    /// Renderer capabilities
    capabilities: RendererCapabilities,
}

impl WebRenderBackend {
    /// Create a new WebRender backend
    pub fn new() -> Self {
        Self {
            initialized: false,
            size: Size2D::new(0, 0),
            capabilities: RendererCapabilities {
                hardware_acceleration: true,
                max_texture_size: 16384, // WebRender can handle large textures
                multithreaded: true,
                supported_apis: vec![RendererApi::WebRender],
            },
        }
    }
    
    /// Create a WebRender backend with custom capabilities
    pub fn with_capabilities(capabilities: RendererCapabilities) -> Self {
        Self {
            initialized: false,
            size: Size2D::new(0, 0),
            capabilities,
        }
    }
}

impl Renderer for WebRenderBackend {
    fn init(&mut self) -> Result<(), RenderingError> {
        // In a real implementation, this would initialize WebRender
        // For now, this is a placeholder
        self.initialized = true;
        Ok(())
    }
    
    fn render_frame(&mut self, display_list: &DisplayList) -> Result<(), RenderingError> {
        if !self.initialized {
            return Err(RenderingError::Renderer("Renderer not initialized".to_string()));
        }
        
        if display_list.is_empty {
            // Nothing to render
            return Ok(());
        }
        
        // In a real implementation, this would convert the display list
        // to WebRender commands and submit them for rendering
        // For now, this is a placeholder
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "WebRender"
    }
    
    fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "hardware_acceleration" => self.capabilities.hardware_acceleration,
            "multithreaded" => self.capabilities.multithreaded,
            "webgl" => true, // WebRender supports WebGL
            "webrtc" => true, // WebRender supports WebRTC
            _ => false,
        }
    }
    
    fn capabilities(&self) -> RendererCapabilities {
        self.capabilities.clone()
    }
    
    fn resize(&mut self, size: Size2D<u32>) -> Result<(), RenderingError> {
        self.size = size;
        // In a real implementation, this would resize the WebRender surface
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Default for WebRenderBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Null renderer for testing
pub struct NullRenderer {
    initialized: bool,
}

impl NullRenderer {
    /// Create a new null renderer
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }
}

impl Renderer for NullRenderer {
    fn init(&mut self) -> Result<(), RenderingError> {
        self.initialized = true;
        Ok(())
    }
    
    fn render_frame(&mut self, _display_list: &DisplayList) -> Result<(), RenderingError> {
        if !self.initialized {
            return Err(RenderingError::Renderer("Renderer not initialized".to_string()));
        }
        // Do nothing - this is a null renderer
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "NullRenderer"
    }
    
    fn supports_feature(&self, _feature: &str) -> bool {
        false
    }
    
    fn capabilities(&self) -> RendererCapabilities {
        RendererCapabilities {
            hardware_acceleration: false,
            max_texture_size: 0,
            multithreaded: false,
            supported_apis: vec![],
        }
    }
    
    fn resize(&mut self, _size: Size2D<u32>) -> Result<(), RenderingError> {
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Default for NullRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_display_list() {
        let mut display_list = DisplayList::new();
        assert!(display_list.is_empty);
        
        display_list.add_command(RenderingCommand::DrawRect {
            rect: Rect::new(Point2D::new(0.0, 0.0), Size2D::new(100.0, 100.0)),
            color: [1.0, 0.0, 0.0, 1.0],
        });
        
        assert!(!display_list.is_empty);
        assert_eq!(display_list.commands.len(), 1);
        
        display_list.clear();
        assert!(display_list.is_empty);
        assert!(display_list.commands.is_empty());
    }
    
    #[test]
    fn test_webrender_backend() {
        let mut renderer = WebRenderBackend::new();
        assert_eq!(renderer.name(), "WebRender");
        
        let capabilities = renderer.capabilities();
        assert!(capabilities.hardware_acceleration);
        assert!(capabilities.multithreaded);
        assert!(capabilities.supported_apis.contains(&RendererApi::WebRender));
        
        assert!(renderer.supports_feature("hardware_acceleration"));
        assert!(renderer.supports_feature("multithreaded"));
        assert!(renderer.supports_feature("webgl"));
        assert!(!renderer.supports_feature("nonexistent"));
    }
    
    #[test]
    fn test_null_renderer() {
        let mut renderer = NullRenderer::new();
        assert_eq!(renderer.name(), "NullRenderer");
        
        let capabilities = renderer.capabilities();
        assert!(!capabilities.hardware_acceleration);
        assert!(!capabilities.multithreaded);
        assert!(capabilities.supported_apis.is_empty());
        
        assert!(!renderer.supports_feature("hardware_acceleration"));
    }
    
    #[test]
    fn test_renderer_trait() {
        let mut renderer = WebRenderBackend::new();
        assert!(renderer.init().is_ok());
        
        let display_list = DisplayList::new();
        assert!(renderer.render_frame(&display_list).is_ok());
        
        // Test downcasting
        let any_ref = renderer.as_any();
        assert!(any_ref.is::<WebRenderBackend>());
    }
}
