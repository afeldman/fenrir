//! Error types for Fenrir-Log.

use thiserror::Error;

/// Errors that can occur during logger initialization or operation.
#[derive(Error, Debug)]
pub enum LogError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Tracing initialization error.
    #[cfg(feature = "tracing")]
    #[error("Tracing initialization error: {0}")]
    TracingInit(String),
    
    /// Inqjet initialization error.
    #[cfg(feature = "perf")]
    #[error("Inqjet initialization error: {0}")]
    InqjetInit(#[from] inqjet::Error),
    
    /// Logroller initialization error.
    #[cfg(feature = "rolling")]
    #[error("Logroller initialization error: {0}")]
    LogrollerInit(String),
    
    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    /// Logger already initialized.
    #[error("Logger already initialized")]
    AlreadyInitialized,
    
    /// Feature not enabled.
    #[error("Feature '{0}' not enabled")]
    FeatureNotEnabled(&'static str),
}

impl LogError {
    /// Create a configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
}
