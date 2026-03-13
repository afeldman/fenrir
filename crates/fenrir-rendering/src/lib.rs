//! Fenrir Rendering — Pure rendering engine abstraction for Fenrir Browser.
//!
//! This crate provides:
//! - WebView management and delegation
//! - Paint scheduling and frame timing
//! - Rendering context management
//! - Integration with Servo rendering engine
//!
//! Note: This crate does NOT contain metrics collection logic.
//! Metrics are handled by the separate `fenrir-metrics` crate.

pub mod webview;

pub use webview::WebViewManager;

/// Error types for rendering operations
#[derive(thiserror::Error, Debug)]
pub enum RenderingError {
    #[error("Engine initialization failed: {0}")]
    EngineInit(String),
    
    #[error("WebView creation failed: {0}")]
    WebViewCreation(String),
    
    #[error("Rendering context error: {0}")]
    Context(String),
    
    #[error("Paint scheduling error: {0}")]
    Scheduling(String),
    
    #[error("Frame callback error: {0}")]
    FrameCallback(String),
}

/// Result type for rendering operations
pub type RenderingResult<T> = std::result::Result<T, RenderingError>;

/// Re-export common types for convenience
pub mod prelude {
    pub use crate::{
        RenderingError, RenderingResult,
        WebViewManager,
    };
}
