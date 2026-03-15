//! Fenrir Rendering — Pure rendering engine abstraction for Fenrir Browser.
//!
//! This crate provides:
//! - Rendering context management (Window + Offscreen)
//! - Compositing engine (Servo content + UI)
//! - WebView management and delegation
//! - Frame rate control and rendering optimization
//! - Integration with Servo rendering engine
//!
//! Note: This crate does NOT contain metrics collection logic.
//! Metrics are handled by the separate `fenrir-metrics` crate.

pub mod compositing;
pub mod context;
pub mod frame_control;
pub mod webview;
pub mod renderer;

pub use compositing::CompositingEngine;
pub use context::RenderingContextManager;
pub use frame_control::{FrameRateController, RenderingState};
pub use webview::{FenrirWebViewDelegate, WebViewManager};
pub use renderer::{Renderer, DisplayList, WebRenderBackend};

/// Error types for rendering operations
#[derive(thiserror::Error, Debug)]
pub enum RenderingError {
    #[error("Engine initialization failed: {0}")]
    EngineInit(String),
    
    #[error("WebView creation failed: {0}")]
    WebViewCreation(String),
    
    #[error("Rendering context error: {0}")]
    Context(String),
    
    #[error("Compositing error: {0}")]
    Compositing(String),
    
    #[error("Paint scheduling error: {0}")]
    Scheduling(String),
    
    #[error("Frame callback error: {0}")]
    FrameCallback(String),
    
    #[error("Renderer error: {0}")]
    Renderer(String),
}

/// Result type for rendering operations
pub type RenderingResult<T> = std::result::Result<T, RenderingError>;

/// Re-export common types for convenience
pub mod prelude {
    pub use crate::{
        CompositingEngine, RenderingContextManager, RenderingError, RenderingResult,
        RenderingState, FrameRateController, FenrirWebViewDelegate, WebViewManager,
        Renderer, DisplayList, WebRenderBackend,
    };
}
