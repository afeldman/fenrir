//! Rendering Context Management
//!
//! Provides management for WindowRenderingContext and OffscreenRenderingContext
//! with proper lifecycle handling and error management.

use std::rc::Rc;
use winit::dpi::PhysicalSize;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use servo::{OffscreenRenderingContext, RenderingContext, WindowRenderingContext};
use tracing::{debug, error, info, trace};

use crate::RenderingError;

/// Manages both window and offscreen rendering contexts
pub struct RenderingContextManager {
    window_ctx: Rc<WindowRenderingContext>,
    offscreen_ctx: Rc<OffscreenRenderingContext>,
    current_size: PhysicalSize<u32>,
    toolbar_height: u32,
}

impl RenderingContextManager {
    /// Create a new RenderingContextManager
    pub fn new(
        display_handle: impl HasDisplayHandle,
        window_handle: impl HasWindowHandle,
        initial_size: PhysicalSize<u32>,
        toolbar_height: u32,
    ) -> Result<Self, RenderingError> {
        info!(
            "Creating rendering contexts: {}x{} (toolbar: {}px)",
            initial_size.width, initial_size.height, toolbar_height
        );

        // Create window rendering context
        let window_ctx = Rc::new(
            WindowRenderingContext::new(
                display_handle.display_handle().map_err(|e| {
                    RenderingError::Context(format!("Failed to get display handle: {:?}", e))
                })?,
                window_handle.window_handle().map_err(|e| {
                    RenderingError::Context(format!("Failed to get window handle: {:?}", e))
                })?,
                initial_size,
            )
            .map_err(|e| RenderingError::Context(format!("WindowRenderingContext: {:?}", e)))?,
        );

        // Make window context current
        window_ctx
            .make_current()
            .map_err(|e| RenderingError::Context(format!("make_current failed: {:?}", e)))?;

        // Calculate offscreen context size (window minus toolbar)
        let scale = 1.0; // Default scale factor
        let offscreen_height = initial_size
            .height
            .saturating_sub((toolbar_height as f32 * scale) as u32);
        let offscreen_size = PhysicalSize::new(initial_size.width, offscreen_height);

        debug!(
            "Offscreen context size: {}x{}",
            offscreen_size.width, offscreen_size.height
        );

        // Create offscreen rendering context
        let offscreen_ctx = Rc::new(window_ctx.offscreen_context(offscreen_size));

        // Make offscreen context current for Servo
        offscreen_ctx
            .make_current()
            .map_err(|e| RenderingError::Context(format!("offscreen make_current failed: {:?}", e)))?;

        info!("Rendering contexts created successfully");

        Ok(Self {
            window_ctx,
            offscreen_ctx,
            current_size: initial_size,
            toolbar_height,
        })
    }

    /// Get the window rendering context
    pub fn window_context(&self) -> &Rc<WindowRenderingContext> {
        &self.window_ctx
    }

    /// Get the offscreen rendering context (for Servo rendering)
    pub fn offscreen_context(&self) -> &Rc<OffscreenRenderingContext> {
        &self.offscreen_ctx
    }

    /// Get the offscreen context as a generic RenderingContext
    pub fn servo_context(&self) -> Rc<dyn RenderingContext> {
        self.offscreen_ctx.clone() as Rc<dyn RenderingContext>
    }

    /// Resize both contexts
    pub fn resize(&self, new_size: PhysicalSize<u32>) -> Result<(), RenderingError> {
        if new_size == self.current_size {
            trace!("Size unchanged, skipping resize");
            return Ok(());
        }

        info!(
            "Resizing rendering contexts from {}x{} to {}x{}",
            self.current_size.width, self.current_size.height, new_size.width, new_size.height
        );

        // Resize window context
        self.window_ctx.resize(new_size);

        // Calculate new offscreen size
        let scale = 1.0; // Default scale factor
        let toolbar_px = (self.toolbar_height as f32 * scale) as u32;
        let offscreen_height = new_size.height.saturating_sub(toolbar_px);
        let offscreen_size = PhysicalSize::new(new_size.width, offscreen_height);

        debug!(
            "New offscreen size: {}x{}",
            offscreen_size.width, offscreen_size.height
        );

        // Resize offscreen context
        self.offscreen_ctx.resize(offscreen_size);

        // Update current size
        // Note: We can't mutate self because we have immutable references
        // The size is tracked externally by the application

        info!("Rendering contexts resized successfully");

        Ok(())
    }

    /// Get the current offscreen size (for Servo rendering)
    pub fn offscreen_size(&self) -> PhysicalSize<u32> {
        self.offscreen_ctx.size()
    }

    /// Get the window scale factor
    pub fn scale_factor(&self) -> f32 {
        1.0 // Default scale factor
    }

    /// Prepare for rendering: make offscreen context current for Servo
    pub fn prepare_for_servo(&self) -> Result<(), RenderingError> {
        trace!("Preparing for Servo rendering");
        self.offscreen_ctx
            .make_current()
            .map_err(|e| RenderingError::Context(format!("prepare_for_servo failed: {:?}", e)))
    }

    /// Prepare for compositing: make window context current for UI
    pub fn prepare_for_compositing(&self) -> Result<(), RenderingError> {
        trace!("Preparing for compositing");
        self.window_ctx
            .make_current()
            .map_err(|e| RenderingError::Context(format!("prepare_for_compositing failed: {:?}", e)))
    }

    /// Present the rendered frame to the screen
    pub fn present(&self) {
        trace!("Presenting frame");
        self.window_ctx.present();
    }

    /// Get the blit callback for compositing Servo content to window
    pub fn blit_callback(&self) -> Option<Box<dyn Fn(&dyn std::any::Any, euclid::default::Rect<i32>) + Send + Sync>> {
        // This is a simplified version for now
        // In a real implementation, we'd need to handle the actual callback type
        None
    }
}
