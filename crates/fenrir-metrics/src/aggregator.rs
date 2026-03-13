//! Metrics aggregation and analysis

use crate::{RenderStats, RequestStats, MemoryStats};
use serde::{Deserialize, Serialize};

/// Aggregates metrics from different sources
pub struct MetricsAggregator {
    // Implementation would aggregate metrics from multiple sources
}

impl MetricsAggregator {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn aggregate(&self, 
                    render_stats: &RenderStats, 
                    network_stats: &RequestStats, 
                    memory_stats: &MemoryStats) -> AggregatedMetrics {
        AggregatedMetrics {
            render: render_stats.clone(),
            network: network_stats.clone(),
            memory: memory_stats.clone(),
            overall_score: self.calculate_overall_score(render_stats, network_stats, memory_stats),
        }
    }
    
    fn calculate_overall_score(&self, 
                              render_stats: &RenderStats, 
                              _network_stats: &RequestStats, 
                              _memory_stats: &MemoryStats) -> f64 {
        // Simple scoring algorithm based on FPS
        if render_stats.fps >= 60.0 {
            100.0
        } else if render_stats.fps >= 30.0 {
            80.0 + (render_stats.fps - 30.0) / 30.0 * 20.0
        } else {
            render_stats.fps / 30.0 * 80.0
        }
    }
}

/// Aggregated metrics from all sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub render: RenderStats,
    pub network: RequestStats,
    pub memory: MemoryStats,
    pub overall_score: f64,
}
