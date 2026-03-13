//! Configuration for metrics collection

use serde::{Deserialize, Serialize};
use crate::render::RenderMetricsConfig;
use crate::network::NetworkMetricsConfig;
use crate::memory::MemoryMetricsConfig;

/// Global metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Render metrics configuration
    pub render: RenderMetricsConfig,
    
    /// Network metrics configuration
    pub network: NetworkMetricsConfig,
    
    /// Memory metrics configuration
    pub memory: MemoryMetricsConfig,
    
    /// Whether to enable real-time monitoring
    pub realtime_monitoring: bool,
    
    /// Monitoring interval in seconds
    pub monitoring_interval_secs: u64,
    
    /// Whether to export metrics to external systems
    pub export_metrics: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            render: RenderMetricsConfig::default(),
            network: NetworkMetricsConfig::default(),
            memory: MemoryMetricsConfig::default(),
            realtime_monitoring: true,
            monitoring_interval_secs: 5,
            export_metrics: false,
        }
    }
}

/// Environment-aware configuration builder
pub struct MetricsConfigBuilder {
    config: MetricsConfig,
}

impl MetricsConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: MetricsConfig::default(),
        }
    }
    
    pub fn for_development() -> Self {
        let mut builder = Self::new();
        builder.config.realtime_monitoring = true;
        builder.config.monitoring_interval_secs = 2;
        builder
    }
    
    pub fn for_production() -> Self {
        let mut builder = Self::new();
        builder.config.realtime_monitoring = false;
        builder.config.monitoring_interval_secs = 30;
        builder.config.export_metrics = true;
        builder
    }
    
    pub fn for_performance_testing() -> Self {
        let mut builder = Self::new();
        builder.config.render.window_size = 1000;
        builder.config.render.auto_log = true;
        builder.config.render.log_interval = 1;
        builder.config.realtime_monitoring = true;
        builder.config.monitoring_interval_secs = 1;
        builder
    }
    
    pub fn build(self) -> MetricsConfig {
        self.config
    }
}
