//! Performance logging backend using inqjet.

use std::io;
use std::time::Duration;

use crate::config::{BackpressureMode, LogLevel, PerfConfig};
use crate::error::LogError;

/// Performance logger using inqjet backend.
pub struct PerfLogger {
    #[allow(dead_code)]
    guard: Option<InqJetGuard>,
    config: PerfConfig,
}

/// Guard for inqjet backend that must be kept alive.
#[cfg(feature = "perf")]
pub struct InqJetGuard {
    // Inqjet doesn't expose a guard type, so we use a unit struct
    // In reality, we'd need to keep the builder or writer alive
}

impl PerfLogger {
    /// Initialize performance logging backend.
    pub fn new(config: PerfConfig) -> Result<Option<Self>, LogError> {
        if !config.enabled {
            return Ok(None);
        }
        
        #[cfg(not(feature = "perf"))]
        return Err(LogError::FeatureNotEnabled("perf"));
        
        #[cfg(feature = "perf")]
        {
            let guard = init_inqjet(&config)?;
            Ok(Some(Self {
                guard: Some(guard),
                config,
            }))
        }
    }
    
    /// Log a performance-critical message.
    pub fn log(&self, target: &str, message: impl std::fmt::Display) {
        #[cfg(feature = "perf")]
        {
            use tracing::Level;
            
            let level = match self.config.level {
                LogLevel::Trace => Level::Trace,
                LogLevel::Debug => Level::Debug,
                LogLevel::Info => Level::Info,
                LogLevel::Warn => Level::Warn,
                LogLevel::Error => Level::Error,
            };
            
            // Check if message should be logged based on level
            if self.should_log(level) {
                inqjet::log!(level, target: target, "{}", message);
            }
        }
        
        #[cfg(not(feature = "perf"))]
        let _ = (target, message);
    }
    
    #[cfg(feature = "perf")]
    fn should_log(&self, level: inqjet::Level) -> bool {
        use tracing::Level;
        
        let config_level = match self.config.level {
            LogLevel::Trace => Level::Trace,
            LogLevel::Debug => Level::Debug,
            LogLevel::Info => Level::Info,
            LogLevel::Warn => Level::Warn,
            LogLevel::Error => Level::Error,
        };
        
        level <= config_level
    }
    
    /// Get the current log level.
    pub fn level(&self) -> LogLevel {
        self.config.level
    }
    
    /// Set the log level at runtime.
    pub fn set_level(&mut self, level: LogLevel) {
        self.config.level = level;
        
        #[cfg(feature = "perf")]
        {
            use inqjet::{LevelFilter};
            use tracing::Level;
            
            let filter = match level {
                LogLevel::Trace => LevelFilter::Trace,
                LogLevel::Debug => LevelFilter::Debug,
                LogLevel::Info => LevelFilter::Info,
                LogLevel::Warn => LevelFilter::Warn,
                LogLevel::Error => LevelFilter::Error,
            };
            
            inqjet::set_level(filter);
        }
    }
}

#[cfg(feature = "perf")]
fn init_inqjet(config: &PerfConfig) -> Result<InqJetGuard, LogError> {
    use inqjet::{ColorMode, InqJetBuilder, LevelFilter};
    
    let level_filter = match config.level {
        LogLevel::Trace => LevelFilter::Trace,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Error => LevelFilter::Error,
    };
    
    let backpressure = match config.backpressure_mode {
        BackpressureMode::Backoff => inqjet::BackpressureMode::Backoff,
        BackpressureMode::Drop => inqjet::BackpressureMode::Drop,
    };
    
    let builder = InqJetBuilder::default()
        .with_writer(io::stdout()) // Default to stdout, can be overridden
        .with_log_level(level_filter)
        .with_buffer_size(config.buffer_size)
        .with_timeout(config.archiver_timeout)
        .with_color_mode(ColorMode::Auto)
        .with_backpressure(backpressure);
    
    let _guard = builder.build()?;
    
    Ok(InqJetGuard {})
}

#[cfg(not(feature = "perf"))]
fn init_inqjet(_config: &PerfConfig) -> Result<InqJetGuard, LogError> {
    Err(LogError::FeatureNotEnabled("perf"))
}

/// Pod trait for zero-cost logging of structured data.
///
/// This is a re-export of inqjet's Pod trait when the perf feature is enabled.
#[cfg(feature = "perf")]
pub use inqjet::Pod;

/// Derive macro for Pod trait.
///
/// This is a re-export of inqjet's Pod derive macro when the perf feature is enabled.
#[cfg(feature = "perf")]
pub use inqjet_macros::Pod;
