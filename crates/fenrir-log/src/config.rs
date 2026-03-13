//! Configuration types for Fenrir-Log.

use std::path::PathBuf;
use std::time::Duration;

use crate::error::LogError;

/// Log level configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Trace level - most verbose.
    Trace,
    /// Debug level - debugging information.
    Debug,
    /// Info level - general information.
    Info,
    /// Warn level - warnings.
    Warn,
    /// Error level - errors.
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

/// Configuration for rolling file logging.
#[derive(Debug, Clone)]
pub struct RollingConfig {
    /// Directory for log files.
    pub directory: PathBuf,
    /// Base filename.
    pub filename: String,
    /// Maximum file size in bytes before rotation.
    pub max_size_bytes: u64,
    /// Maximum number of files to keep.
    pub max_files: usize,
    /// Whether to compress rotated files.
    pub compress: bool,
}

impl Default for RollingConfig {
    fn default() -> Self {
        Self {
            directory: PathBuf::from("logs"),
            filename: "fenrir.log".to_string(),
            max_size_bytes: 10 * 1024 * 1024, // 10MB
            max_files: 7,
            compress: true,
        }
    }
}

/// Configuration for performance logging backend.
#[derive(Debug, Clone)]
pub struct PerfConfig {
    /// Whether performance logging is enabled.
    pub enabled: bool,
    /// Buffer size in bytes (power of two).
    pub buffer_size: usize,
    /// Backpressure mode.
    pub backpressure_mode: BackpressureMode,
    /// Archiver timeout (None for busy-spin).
    pub archiver_timeout: Option<Duration>,
    /// Log level for performance backend.
    pub level: LogLevel,
}

impl Default for PerfConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            buffer_size: 1 << 20, // 1MB
            backpressure_mode: BackpressureMode::Drop,
            archiver_timeout: Some(Duration::from_millis(5)),
            level: LogLevel::Debug,
        }
    }
}

/// Backpressure mode for performance logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureMode {
    /// Exponential backoff (guarantees delivery).
    Backoff,
    /// Drop messages when buffer is full (bounded latency).
    Drop,
}

/// Main configuration struct for Fenrir-Log.
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Whether tracing backend is enabled.
    pub enable_tracing: bool,
    /// Tracing log level.
    pub tracing_level: LogLevel,
    /// Whether to log to stderr.
    pub log_to_stderr: bool,
    /// Stderr log level (usually higher than file level).
    pub stderr_level: LogLevel,
    /// Whether to enable ANSI colors in stderr.
    pub ansi_colors: bool,
    /// Rolling file configuration.
    pub rolling_config: Option<RollingConfig>,
    /// Performance logging configuration.
    pub perf_config: PerfConfig,
    /// Additional tracing directives.
    pub tracing_directives: Vec<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            enable_tracing: true,
            tracing_level: LogLevel::Trace,
            log_to_stderr: true,
            stderr_level: LogLevel::Info,
            ansi_colors: true,
            rolling_config: None,
            perf_config: PerfConfig::default(),
            tracing_directives: vec![
                "fenrir=trace".to_string(),
                "fenrir_app=trace".to_string(),
                "fenrir_core=trace".to_string(),
                "fenrir_servo=trace".to_string(),
                "fenrir_network=trace".to_string(),
                "fenrir_ai=trace".to_string(),
                "servo=debug".to_string(),
            ],
        }
    }
}

impl LogConfig {
    /// Create a new builder with default configuration.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Enable tracing backend.
    pub fn with_tracing(mut self) -> Self {
        self.enable_tracing = true;
        self
    }
    
    /// Disable tracing backend.
    pub fn without_tracing(mut self) -> Self {
        self.enable_tracing = false;
        self
    }
    
    /// Set tracing log level.
    pub fn with_tracing_level(mut self, level: LogLevel) -> Self {
        self.tracing_level = level;
        self
    }
    
    /// Enable performance logging backend.
    pub fn with_perf_backend(mut self) -> Self {
        self.perf_config.enabled = true;
        self
    }
    
    /// Configure performance logging backend.
    pub fn with_perf_config(mut self, config: PerfConfig) -> Self {
        self.perf_config = config;
        self
    }
    
    /// Enable rolling file logging.
    pub fn with_rolling_file(
        mut self,
        directory: impl Into<PathBuf>,
        filename: impl Into<String>,
        max_size_mb: u64,
        max_files: usize,
    ) -> Self {
        self.rolling_config = Some(RollingConfig {
            directory: directory.into(),
            filename: filename.into(),
            max_size_bytes: max_size_mb * 1024 * 1024,
            max_files,
            compress: true,
        });
        self
    }
    
    /// Add a tracing directive.
    pub fn with_directive(mut self, directive: impl Into<String>) -> Self {
        self.tracing_directives.push(directive.into());
        self
    }
    
    /// Build the logger from configuration.
    pub fn build(self) -> Result<crate::FenrirLogger, LogError> {
        crate::FenrirLogger::from_config(self)
    }
}
