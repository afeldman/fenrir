//! Compositing Engine
//!
//! Handles the composition of Servo's offscreen rendering with UI elements
//! and presents the final frame to the screen.

use std::rc::Rc;
use std::sync::Arc;
use euclid::default::Rect;
use glow::HasContext;
use tracing::{debug, error, trace, warn};

use crate::RenderingError;

/// Type for blit callbacks from Servo
type BlitCallback = dyn Fn(&glow::Context, Rect<i32>) + Send + Sync;

/// Compositing engine that handles Servo content + UI composition
pub struct CompositingEngine {
    blit_callback: Option<Arc<BlitCallback>>,
    clear_color: [f32; 4],
}

impl CompositingEngine {
    /// Create a new compositing engine
    pub fn new() -> Self {
        Self {
            blit_callback: None,
            clear_color: [1.0, 1.0, 1.0, 1.0], // White background
        }
    }

    /// Set the blit callback from Servo's offscreen context
    pub fn set_blit_callback<F>(&mut self, callback: F)
    where
        F: Fn(&glow::Context, Rect<i32>) + Send + Sync + 'static,
    {
        self.blit_callback = Some(Arc::new(callback));
        trace!("Blit callback set");
    }

    /// Set the clear color for the offscreen buffer
    pub fn set_clear_color(&mut self, color: [f32; 4]) {
        self.clear_color = color;
    }

    /// Clear the offscreen buffer before Servo renders
    pub fn clear_offscreen_buffer(&self, gl: &glow::Context) -> Result<(), RenderingError> {
        trace!("Clearing offscreen buffer");

        unsafe {
            // Check current framebuffer binding
            let current_fbo = gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING);
            trace!("Current framebuffer before clear: {}", current_fbo);

            // Check framebuffer status
            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
            trace!("Framebuffer status before clear: {:?}", status);

            // Only clear if framebuffer is valid
            if status == glow::FRAMEBUFFER_COMPLETE {
                gl.clear_color(
                    self.clear_color[0],
                    self.clear_color[1],
                    self.clear_color[2],
                    self.clear_color[3],
                );
                gl.clear(glow::COLOR_BUFFER_BIT);
                trace!("Framebuffer successfully cleared");
            } else {
                warn!("Framebuffer is not complete! Status: {:?}", status);
                // Try to bind default framebuffer (0)
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                trace!("Default framebuffer bound");
            }

            let gl_error = gl.get_error();
            if gl_error != glow::NO_ERROR {
                error!("OpenGL error after framebuffer initialization: {:?}", gl_error);
                return Err(RenderingError::Context(format!(
                    "OpenGL error: {:?}",
                    gl_error
                )));
            }
        }

        Ok(())
    }

    /// Composite Servo content to the window
    pub fn composite_servo_content(
        &self,
        gl: &glow::Context,
        target_rect: Rect<i32>,
    ) -> Result<(), RenderingError> {
        trace!("Compositing Servo content to rect: {:?}", target_rect);

        if let Some(blit) = &self.blit_callback {
            // Apply scissor test to limit drawing to target area
            unsafe {
                gl.enable(glow::SCISSOR_TEST);
                gl.scissor(
                    target_rect.origin.x,
                    target_rect.origin.y,
                    target_rect.size.width,
                    target_rect.size.height,
                );

                // Clear target area with transparent color
                gl.clear_color(0.0, 0.0, 0.0, 0.0);
                gl.clear(glow::COLOR_BUFFER_BIT);

                gl.disable(glow::SCISSOR_TEST);
            }

            // Call Servo's blit callback
            blit(gl, target_rect);
            trace!("Servo content composited successfully");
        } else {
            trace!("No blit callback available - Servo content not rendered");
            // Draw a placeholder (e.g., white background) when no Servo content
            unsafe {
                gl.enable(glow::SCISSOR_TEST);
                gl.scissor(
                    target_rect.origin.x,
                    target_rect.origin.y,
                    target_rect.size.width,
                    target_rect.size.height,
                );

                gl.clear_color(1.0, 1.0, 1.0, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);

                gl.disable(glow::SCISSOR_TEST);
            }
        }

        // Check for OpenGL errors
        unsafe {
            let gl_error = gl.get_error();
            if gl_error != glow::NO_ERROR {
                warn!("OpenGL error after compositing: {:?}", gl_error);
                return Err(RenderingError::Context(format!(
                    "OpenGL compositing error: {:?}",
                    gl_error
                )));
            }
        }

        Ok(())
    }

    /// Check if a blit callback is available
    pub fn has_blit_callback(&self) -> bool {
        self.blit_callback.is_some()
    }
}

impl Default for CompositingEngine {
    fn default() -> Self {
        Self::new()
    }
}
