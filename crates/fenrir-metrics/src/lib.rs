//! Fenrir Metrics — Performance monitoring and metrics collection for Fenrir Browser.
//!
//! This crate provides:
//! - Render performance metrics (FPS, frame times, percentiles)
//! - Network metrics (latency, throughput, request timing)
//! - Memory metrics (allocations, heap usage)
//! - Aggregation and statistical analysis
//! - Real-time monitoring capabilities

pub mod render;
pub mod network;
pub mod memory;
pub mod aggregator;
pub mod config;

pub use config::MetricsConfig;
pub use render::{RenderMetrics, RenderMetricsConfig, RenderStats, RenderTimer};
pub use network::{NetworkMetrics, NetworkMetricsConfig, RequestStats};
pub use memory::{MemoryMetrics, MemoryMetricsConfig, MemoryStats};
pub use aggregator::MetricsAggregator;

/// Re-export common types for convenience
pub mod prelude {
    pub use crate::{
        MetricsConfig,
        RenderMetrics, RenderMetricsConfig, RenderStats, RenderTimer,
        NetworkMetrics, NetworkMetricsConfig, RequestStats,
        MemoryMetrics, MemoryMetricsConfig, MemoryStats,
        MetricsAggregator,
    };
}

/// Error types for metrics collection
#[derive(thiserror::Error, Debug)]
pub enum MetricsError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Collection error: {0}")]
    Collection(String),
    
    #[error("Aggregation error: {0}")]
    Aggregation(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for metrics operations
pub type MetricsResult<T> = std::result::Result<T, MetricsError>;
