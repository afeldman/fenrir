//! Fenrir-Log: Hybrid logging library combining tracing observability with inqjet performance.
//!
//! # Features
//!
//! - **Hybrid Architecture**: Use `tracing` for structured logs and `inqjet` for performance-critical paths
//! - **Feature Flags**: Enable only what you need (`tracing`, `rolling`, `perf`)
//! - **Zero-Cost Abstraction**: `Pod` type support for efficient structured logging
//! - **Runtime Configuration**: Adjust log levels and backends at runtime
//! - **Rolling File Support**: Built-in log rotation with compression
//!
//! # Quick Start
//!
//! ```rust
//! use fenrir_log::{FenrirLogger, LogConfig};
//!
//! // Basic setup with tracing only
//! let logger = FenrirLogger::builder()
//!     .with_tracing()
//!     .build()?;
//!
//! // Full setup with both backends
//! let logger = FenrirLogger::builder()
//!     .with_tracing()
//!     .with_perf_backend()  // Enable inqjet for performance logs
//!     .with_rolling_file("logs/fenrir.log", 10 * 1024 * 1024, 7) // 10MB files, keep 7
//!     .build()?;
//!
//! // Performance-critical logging (uses inqjet if enabled)
//! logger.perf_log("network::request", format!("Request to {}", url));
//!
//! // Standard logging (uses tracing)
//! tracing::info!("User logged in: {}", username);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod config;
pub mod error;

#[cfg(feature = "perf")]
pub mod perf;

#[cfg(feature = "tracing")]
pub mod tracing;

pub use config::LogConfig;
pub use error::LogError;

/// Re-export common logging types for convenience
pub mod prelude {
    pub use super::LogConfig;
    pub use super::LogError;
    
    #[cfg(feature = "perf")]
    pub use super::perf::PerfLogger;
    
    #[cfg(feature = "tracing")]
    pub use super::tracing::TracingGuard;
}

/// Main logger struct that manages both tracing and performance backends.
pub struct FenrirLogger {
    #[cfg(feature = "tracing")]
    tracing_guard: Option<crate::tracing::TracingGuard>,
    
    #[cfg(feature = "perf")]
    perf_logger: Option<crate::perf::PerfLogger>,
    
    config: LogConfig,
}

impl FenrirLogger {
    /// Creates a new builder for configuring the logger.
    pub fn builder() -> LogConfig {
        LogConfig::default()
    }
    
    /// Create logger from configuration.
    pub fn from_config(config: LogConfig) -> Result<Self, LogError> {
        #[cfg(feature = "tracing")]
        let tracing_guard = crate::tracing::init_tracing(&config)?;
        
        #[cfg(not(feature = "tracing"))]
        let tracing_guard = None;
        
        #[cfg(feature = "perf")]
        let perf_logger = crate::perf::PerfLogger::new(config.perf_config.clone())?;
        
        #[cfg(not(feature = "perf"))]
        let perf_logger = None;
        
        Ok(Self {
            #[cfg(feature = "tracing")]
            tracing_guard,
            #[cfg(feature = "perf")]
            perf_logger,
            config,
        })
    }
    
    /// Log a performance-critical message.
    ///
    /// This uses the inqjet backend if enabled, otherwise falls back to tracing.
    pub fn perf_log(&self, target: &str, message: impl std::fmt::Display) {
        #[cfg(feature = "perf")]
        if let Some(ref perf_logger) = self.perf_logger {
            perf_logger.log(target, message);
            return;
        }
        
        #[cfg(feature = "tracing")]
        if self.config.enable_tracing {
            tracing::debug!(target: target, "{}", message);
        }
    }
    
    /// Log a performance-critical message with Pod type.
    ///
    /// This is more efficient than regular logging for structured data.
    #[cfg(feature = "perf")]
    pub fn perf_log_pod<T: crate::perf::Pod + std::fmt::Debug>(
        &self,
        target: &str,
        data: &T,
        message: &str,
    ) {
        if let Some(ref perf_logger) = self.perf_logger {
            use inqjet::Level;
            
            let level = match self.config.perf_config.level {
                config::LogLevel::Trace => Level::Trace,
                config::LogLevel::Debug => Level::Debug,
                config::LogLevel::Info => Level::Info,
                config::LogLevel::Warn => Level::Warn,
                config::LogLevel::Error => Level::Error,
            };
            
            inqjet::log!(level, target: target, "{}: {:?}", message, data);
        }
    }
    
    /// Check if performance logging is enabled.
    pub fn has_perf_backend(&self) -> bool {
        #[cfg(feature = "perf")]
        return self.perf_logger.is_some();
        
        #[cfg(not(feature = "perf"))]
        false
    }
    
    /// Check if tracing is enabled.
    pub fn has_tracing(&self) -> bool {
        #[cfg(feature = "tracing")]
        return self.tracing_guard.is_some();
        
        #[cfg(not(feature = "tracing"))]
        false
    }
    
    /// Get the configuration.
    pub fn config(&self) -> &LogConfig {
        &self.config
    }
}

/// Initialize the global logger with default configuration.
///
/// # Panics
///
/// Panics if logger initialization fails.
pub fn init_default() -> FenrirLogger {
    FenrirLogger::builder()
        .with_tracing()
        .build()
        .expect("Failed to initialize default logger")
}

/// Example Pod type for structured logging.
#[cfg(feature = "perf")]
#[derive(Debug, crate::perf::Pod)]
pub struct NetworkRequest {
    pub url: &'static str,
    pub method: &'static str,
    pub duration_ms: u64,
    pub status_code: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builder() {
        let config = FenrirLogger::builder()
            .with_tracing()
            .with_perf_backend();
        
        assert!(config.enable_tracing);
        assert!(config.perf_config.enabled);
    }
    
    #[test]
    #[cfg(feature = "perf")]
    fn test_pod_type() {
        let request = NetworkRequest {
            url: "https://example.com",
            method: "GET",
            duration_ms: 42,
            status_code: 200,
        };
        
        // This should compile, proving Pod trait works
        let _ = &request as &dyn crate::perf::Pod;
    }
}
