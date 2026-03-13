//! Network metrics collection

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for network metrics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetricsConfig {
    /// Whether to track network metrics
    pub enabled: bool,
    
    /// Window size for request latency statistics
    pub window_size: usize,
    
    /// Whether to log slow requests
    pub log_slow_requests: bool,
    
    /// Threshold for slow requests in milliseconds
    pub slow_request_threshold_ms: u64,
    
    /// Whether to track DNS resolution times
    pub track_dns: bool,
    
    /// Whether to track TLS handshake times
    pub track_tls: bool,
}

impl Default for NetworkMetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            window_size: 100,
            log_slow_requests: true,
            slow_request_threshold_ms: 1000, // 1 second
            track_dns: true,
            track_tls: true,
        }
    }
}

/// Network metrics tracker
pub struct NetworkMetrics {
    config: NetworkMetricsConfig,
    // Implementation would track request latencies, DNS times, etc.
}

impl NetworkMetrics {
    pub fn new(config: NetworkMetricsConfig) -> Self {
        Self { config }
    }
    
    pub fn record_request(&self, _url: &str, _method: &str, _duration: Duration) {
        // Implementation would record request metrics
    }
    
    pub fn calculate_stats(&self) -> RequestStats {
        RequestStats::default()
    }
}

/// Network request statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestStats {
    pub total_requests: usize,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub slow_requests: usize,
    pub error_rate: f64,
}
