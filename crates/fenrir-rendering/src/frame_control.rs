//! Frame Rate Control and Rendering Optimization
//!
//! Provides frame rate limiting, rendering state tracking, and performance optimization.

use std::time::{Duration, Instant};
use tracing::{debug, trace};

/// Frame rate controller with configurable limits
pub struct FrameRateController {
    min_frame_time: Duration,
    last_frame_time: Instant,
    frame_count: u32,
    consecutive_static_frames: u32,
    max_consecutive_static_frames: u32,
    page_loaded: bool,
}

impl FrameRateController {
    /// Create a new frame rate controller
    pub fn new(target_fps: u32, max_static_frames: u32) -> Self {
        let min_frame_time = Duration::from_secs_f32(1.0 / target_fps as f32);
        
        Self {
            min_frame_time,
            last_frame_time: Instant::now(),
            frame_count: 0,
            consecutive_static_frames: 0,
            max_consecutive_static_frames: max_static_frames,
            page_loaded: false,
        }
    }

    /// Check if enough time has passed for a new frame
    pub fn should_render(&mut self) -> bool {
        let now = Instant::now();
        let time_since_last_frame = now.duration_since(self.last_frame_time);

        if time_since_last_frame >= self.min_frame_time {
            self.last_frame_time = now;
            self.frame_count += 1;
            
            // Reset static frame counter if we're rendering
            self.consecutive_static_frames = 0;
            
            trace!("Frame {} rendered after {:?}", self.frame_count, time_since_last_frame);
            true
        } else {
            // Too soon since last frame
            false
        }
    }

    /// Notify that a frame was requested but not rendered (static content)
    pub fn notify_static_frame(&mut self) {
        self.consecutive_static_frames += 1;
        
        if self.consecutive_static_frames >= self.max_consecutive_static_frames {
            trace!("Max consecutive static frames reached ({})", self.consecutive_static_frames);
            // In a real implementation, we might reduce rendering frequency here
        }
    }

    /// Set page loaded state (affects rendering optimization)
    pub fn set_page_loaded(&mut self, loaded: bool) {
        if self.page_loaded != loaded {
            self.page_loaded = loaded;
            if loaded {
                debug!("Page loaded - enabling rendering optimizations");
                // Reset counters when page loads
                self.consecutive_static_frames = 0;
            }
        }
    }

    /// Check if page is loaded
    pub fn is_page_loaded(&self) -> bool {
        self.page_loaded
    }

    /// Get current frame count
    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    /// Get consecutive static frames count
    pub fn consecutive_static_frames(&self) -> u32 {
        self.consecutive_static_frames
    }

    /// Reset all counters (e.g., on navigation)
    pub fn reset(&mut self) {
        self.frame_count = 0;
        self.consecutive_static_frames = 0;
        self.last_frame_time = Instant::now();
        trace!("Frame rate controller reset");
    }

    /// Get the target frame time
    pub fn target_frame_time(&self) -> Duration {
        self.min_frame_time
    }

    /// Get the current FPS based on recent frames
    pub fn current_fps(&self) -> f32 {
        if self.frame_count < 2 {
            return 0.0;
        }
        
        // Simple FPS calculation - in a real implementation,
        // you might want to track frame times over a window
        1.0 / self.min_frame_time.as_secs_f32()
    }
}

impl Default for FrameRateController {
    fn default() -> Self {
        // Default: 30 FPS target, max 10 consecutive static frames
        Self::new(30, 10)
    }
}

/// Rendering state tracker
pub struct RenderingState {
    pub frame_rate_controller: FrameRateController,
    pub last_redraw_time: Instant,
    pub rendering_enabled: bool,
}

impl RenderingState {
    /// Create new rendering state
    pub fn new() -> Self {
        Self {
            frame_rate_controller: FrameRateController::default(),
            last_redraw_time: Instant::now(),
            rendering_enabled: true,
        }
    }

    /// Check if rendering should occur (combines frame rate control and enabled state)
    pub fn should_render(&mut self) -> bool {
        if !self.rendering_enabled {
            return false;
        }
        
        self.frame_rate_controller.should_render()
    }

    /// Request a redraw (for user interactions, bypasses frame rate limiting)
    pub fn request_immediate_redraw(&mut self) {
        self.last_redraw_time = Instant::now();
        self.frame_rate_controller.reset();
        trace!("Immediate redraw requested");
    }

    /// Request a redraw with frame rate limiting (for automatic updates)
    pub fn request_limited_redraw(&mut self) -> bool {
        let now = Instant::now();
        let time_since_last_redraw = now.duration_since(self.last_redraw_time);
        
        if time_since_last_redraw >= self.frame_rate_controller.target_frame_time() {
            self.last_redraw_time = now;
            true
        } else {
            // Too soon, skip this redraw
            self.frame_rate_controller.notify_static_frame();
            false
        }
    }

    /// Enable or disable rendering
    pub fn set_rendering_enabled(&mut self, enabled: bool) {
        if self.rendering_enabled != enabled {
            self.rendering_enabled = enabled;
            if enabled {
                debug!("Rendering enabled");
                self.frame_rate_controller.reset();
            } else {
                debug!("Rendering disabled");
            }
        }
    }

    /// Get whether rendering is enabled
    pub fn is_rendering_enabled(&self) -> bool {
        self.rendering_enabled
    }

    /// Update page loaded state
    pub fn set_page_loaded(&mut self, loaded: bool) {
        self.frame_rate_controller.set_page_loaded(loaded);
    }

    /// Reset all state (e.g., on navigation)
    pub fn reset(&mut self) {
        self.frame_rate_controller.reset();
        self.last_redraw_time = Instant::now();
        self.rendering_enabled = true;
        trace!("Rendering state reset");
    }
}

impl Default for RenderingState {
    fn default() -> Self {
        Self::new()
    }
}
