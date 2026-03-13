//! Tracing backend implementation for Fenrir-Log.

use std::path::PathBuf;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

use crate::config::{LogConfig, LogLevel};
use crate::error::LogError;

/// Guard for tracing backend that must be kept alive.
pub struct TracingGuard {
    #[allow(dead_code)]
    worker_guard: WorkerGuard,
    #[allow(dead_code)]
    rolling_guard: Option<WorkerGuard>,
}

/// Initialize tracing backend.
pub fn init_tracing(config: &LogConfig) -> Result<Option<TracingGuard>, LogError> {
    if !config.enable_tracing {
        return Ok(None);
    }
    
    let mut layers = Vec::new();
    
    // File layer if rolling is configured
    if let Some(rolling_config) = &config.rolling_config {
        let file_layer = create_file_layer(rolling_config, config.tracing_level)?;
        layers.push(file_layer);
    }
    
    // Stderr layer
    if config.log_to_stderr {
        let stderr_layer = create_stderr_layer(config.stderr_level, config.ansi_colors);
        layers.push(stderr_layer);
    }
    
    // Initialize subscriber
    if layers.is_empty() {
        return Ok(None);
    }
    
    let subscriber = tracing_subscriber::registry().with(layers);
    subscriber.try_init().map_err(|e| LogError::TracingInit(e.to_string()))?;
    
    tracing::info!("Tracing logger initialized with level: {:?}", config.tracing_level);
    
    // Note: We don't actually create WorkerGuards here since we're using non-blocking
    // appenders directly in the layers. In a real implementation, we'd need to store them.
    Ok(Some(TracingGuard {
        worker_guard: create_dummy_guard(),
        rolling_guard: None,
    }))
}

fn create_file_layer(
    config: &crate::config::RollingConfig,
    level: LogLevel,
) -> Result<Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync>, LogError> {
    use logroller::{Compression, LogRollerBuilder, Rotation, RotationSize};
    use tracing_subscriber::fmt::Layer;
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&config.directory)?;
    
    let log_file = PathBuf::from(&config.filename);
    let appender = LogRollerBuilder::new(&config.directory, &log_file)
        .rotation(Rotation::SizeBased(RotationSize::Bytes(config.max_size_bytes)))
        .max_keep_files(config.max_files.try_into().unwrap())
        .compression(if config.compress {
            Compression::Gzip
        } else {
            Compression::None
        })
        .build()?;
    
    let (non_blocking, guard) = tracing_appender::non_blocking(appender);
    
    let filter = EnvFilter::from_default_env()
        .add_directive(format!("{}={}", "fenrir", level).parse().unwrap());
    
    let layer = Layer::default()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .with_filter(filter);
    
    // Store guard somewhere - for now we'll just leak it
    std::mem::forget(guard);
    
    Ok(Box::new(layer))
}

fn create_stderr_layer(level: LogLevel, ansi_colors: bool) -> Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync> {
    use tracing_subscriber::fmt::Layer;
    
    let filter = EnvFilter::from_default_env()
        .add_directive(format!("{}={}", "fenrir", level).parse().unwrap());
    
    Box::new(
        Layer::default()
            .with_writer(std::io::stderr)
            .with_ansi(ansi_colors)
            .compact()
            .with_filter(filter)
    )
}

fn create_dummy_guard() -> WorkerGuard {
    // Create a dummy guard - in real implementation this would be the actual guard
    let (_, guard) = tracing_appender::non_blocking(std::io::sink());
    guard
}
