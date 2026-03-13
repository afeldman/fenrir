//! Render performance metrics with sliding window average.
//!
//! Tracks rendering times and calculates moving averages over the last N frames.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

/// Configuration for render metrics tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMetricsConfig {
    /// Number of samples to keep for moving average
    pub window_size: usize,
    
    /// Whether to log metrics automatically
    pub auto_log: bool,
    
    /// Log interval in frames (log every N frames)
    pub log_interval: usize,
    
    /// Alert threshold for slow frames (in milliseconds)
    pub slow_frame_threshold_ms: f64,
    
    /// Alert threshold for frame time variance
    pub high_variance_threshold_ms: f64,
    
    /// Whether to enable frame time prediction
    pub enable_prediction: bool,
    
    /// Prediction model window size
    pub prediction_window: usize,
}

impl Default for RenderMetricsConfig {
    fn default() -> Self {
        Self {
            window_size: 100,
            auto_log: true,
            log_interval: 10,
            slow_frame_threshold_ms: 33.0, // ~30 FPS
            high_variance_threshold_ms: 20.0,
            enable_prediction: false,
            prediction_window: 50,
        }
    }
}

/// Tracks render performance metrics.
pub struct RenderMetrics {
    config: RenderMetricsConfig,
    render_times: Mutex<VecDeque<Duration>>,
    frame_count: Mutex<usize>,
    last_log_frame: Mutex<usize>,
    slow_frames: Mutex<usize>,
    total_slow_time: Mutex<Duration>,
}

impl RenderMetrics {
    /// Create a new RenderMetrics tracker.
    pub fn new(config: RenderMetricsConfig) -> Self {
        let window_size = config.window_size;
        Self {
            config,
            render_times: Mutex::new(VecDeque::with_capacity(window_size)),
            frame_count: Mutex::new(0),
            last_log_frame: Mutex::new(0),
            slow_frames: Mutex::new(0),
            total_slow_time: Mutex::new(Duration::ZERO),
        }
    }
    
    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(RenderMetricsConfig::default())
    }
    
    /// Record a render duration.
    pub fn record_render(&self, duration: Duration) {
        let mut times = self.render_times.lock().unwrap();
        let mut frame_count = self.frame_count.lock().unwrap();
        let mut last_log_frame = self.last_log_frame.lock().unwrap();
        let mut slow_frames = self.slow_frames.lock().unwrap();
        let mut total_slow_time = self.total_slow_time.lock().unwrap();
        
        // Check for slow frame
        if duration.as_secs_f64() * 1000.0 > self.config.slow_frame_threshold_ms {
            *slow_frames += 1;
            *total_slow_time += duration;
        }
        
        // Add new duration
        times.push_back(duration);
        
        // Keep only window_size samples
        if times.len() > self.config.window_size {
            times.pop_front();
        }
        
        *frame_count += 1;
        
        // Auto-log if configured
        if self.config.auto_log && *frame_count - *last_log_frame >= self.config.log_interval {
            self.log_metrics();
            *last_log_frame = *frame_count;
        }
    }
    
    /// Start timing a render operation.
    pub fn start_timing(&self) -> RenderTimer<'_> {
        RenderTimer {
            metrics: self,
            start: Instant::now(),
        }
    }
    
    /// Calculate current statistics.
    pub fn calculate_stats(&self) -> RenderStats {
        let times = self.render_times.lock().unwrap();
        let slow_frames = *self.slow_frames.lock().unwrap();
        let total_slow_time = *self.total_slow_time.lock().unwrap();
        
        if times.is_empty() {
            return RenderStats::empty();
        }
        
        let count = times.len();
        let total: Duration = times.iter().sum();
        let avg = total / count as u32;
        
        let min = *times.iter().min().unwrap_or(&Duration::ZERO);
        let max = *times.iter().max().unwrap_or(&Duration::ZERO);
        
        // Calculate median
        let mut sorted: Vec<Duration> = times.iter().copied().collect();
        sorted.sort();
        let median = if count % 2 == 0 {
            let mid = count / 2;
            (sorted[mid - 1] + sorted[mid]) / 2
        } else {
            sorted[count / 2]
        };
        
        // Calculate percentiles
        let p95_index = (count as f64 * 0.95).floor() as usize;
        let p95 = if p95_index < count {
            sorted[p95_index]
        } else {
            sorted.last().copied().unwrap_or(Duration::ZERO)
        };
        
        let p99_index = (count as f64 * 0.99).floor() as usize;
        let p99 = if p99_index < count {
            sorted[p99_index]
        } else {
            sorted.last().copied().unwrap_or(Duration::ZERO)
        };
        
        // Calculate variance and standard deviation
        let avg_ms = avg.as_secs_f64() * 1000.0;
        let variance: f64 = times.iter()
            .map(|&d| {
                let diff = d.as_secs_f64() * 1000.0 - avg_ms;
                diff * diff
            })
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();
        
        // Calculate FPS
        let fps = if avg > Duration::ZERO {
            1000.0 / avg.as_secs_f64() * 1000.0
        } else {
            0.0
        };
        
        RenderStats {
            count,
            avg_ms: avg.as_secs_f64() * 1000.0,
            min_ms: min.as_secs_f64() * 1000.0,
            max_ms: max.as_secs_f64() * 1000.0,
            median_ms: median.as_secs_f64() * 1000.0,
            p95_ms: p95.as_secs_f64() * 1000.0,
            p99_ms: p99.as_secs_f64() * 1000.0,
            std_dev_ms: std_dev,
            fps,
            slow_frames,
            slow_frame_percentage: if *self.frame_count.lock().unwrap() > 0 {
                slow_frames as f64 / *self.frame_count.lock().unwrap() as f64 * 100.0
            } else {
                0.0
            },
            avg_slow_time_ms: if slow_frames > 0 {
                total_slow_time.as_secs_f64() * 1000.0 / slow_frames as f64
            } else {
                0.0
            },
            window_size: self.config.window_size,
            total_frames: *self.frame_count.lock().unwrap(),
        }
    }
    
    /// Predict next frame time based on historical data
    pub fn predict_next_frame(&self) -> Option<f64> {
        if !self.config.enable_prediction {
            return None;
        }
        
        let times = self.render_times.lock().unwrap();
        if times.len() < 2 {
            return None;
        }
        
        // Simple moving average prediction
        let window = times.len().min(self.config.prediction_window);
        let recent: Vec<Duration> = times.iter().rev().take(window).copied().collect();
        let sum: Duration = recent.iter().sum();
        
        Some(sum.as_secs_f64() * 1000.0 / recent.len() as f64)
    }
    
    /// Check for performance anomalies
    pub fn check_anomalies(&self) -> Vec<PerformanceAnomaly> {
        let stats = self.calculate_stats();
        let mut anomalies = Vec::new();
        
        // Check for slow average
        if stats.avg_ms > self.config.slow_frame_threshold_ms {
            anomalies.push(PerformanceAnomaly::SlowAverage(
                stats.avg_ms,
                self.config.slow_frame_threshold_ms
            ));
        }
        
        // Check for high variance
        if stats.max_ms - stats.min_ms > self.config.high_variance_threshold_ms {
            anomalies.push(PerformanceAnomaly::HighVariance(
                stats.max_ms - stats.min_ms,
                self.config.high_variance_threshold_ms
            ));
        }
        
        // Check for many slow frames
        if stats.slow_frame_percentage > 10.0 {
            anomalies.push(PerformanceAnomaly::ManySlowFrames(
                stats.slow_frame_percentage
            ));
        }
        
        anomalies
    }
    
    /// Log current metrics.
    pub fn log_metrics(&self) {
        let stats = self.calculate_stats();
        
        // Use tracing for logging
        tracing::debug!(
            count = stats.count,
            avg_ms = stats.avg_ms,
            min_ms = stats.min_ms,
            max_ms = stats.max_ms,
            median_ms = stats.median_ms,
            p95_ms = stats.p95_ms,
            p99_ms = stats.p99_ms,
            std_dev_ms = stats.std_dev_ms,
            fps = stats.fps,
            slow_frames = stats.slow_frames,
            slow_frame_percentage = stats.slow_frame_percentage,
            window_size = stats.window_size,
            total_frames = stats.total_frames,
            "Render performance metrics"
        );
        
        // Check and log anomalies
        let anomalies = self.check_anomalies();
        if !anomalies.is_empty() {
            for anomaly in anomalies {
                tracing::warn!("Performance anomaly detected: {:?}", anomaly);
            }
        }
    }
    
    /// Get the current frame count.
    pub fn frame_count(&self) -> usize {
        *self.frame_count.lock().unwrap()
    }
    
    /// Clear all recorded metrics.
    pub fn clear(&self) {
        let mut times = self.render_times.lock().unwrap();
        let mut frame_count = self.frame_count.lock().unwrap();
        let mut last_log_frame = self.last_log_frame.lock().unwrap();
        let mut slow_frames = self.slow_frames.lock().unwrap();
        let mut total_slow_time = self.total_slow_time.lock().unwrap();
        
        times.clear();
        *frame_count = 0;
        *last_log_frame = 0;
        *slow_frames = 0;
        *total_slow_time = Duration::ZERO;
    }
}

/// Statistics for render performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderStats {
    /// Number of samples in the window
    pub count: usize,
    
    /// Average render time in milliseconds
    pub avg_ms: f64,
    
    /// Minimum render time in milliseconds
    pub min_ms: f64,
    
    /// Maximum render time in milliseconds
    pub max_ms: f64,
    
    /// Median render time in milliseconds
    pub median_ms: f64,
    
    /// 95th percentile render time in milliseconds
    pub p95_ms: f64,
    
    /// 99th percentile render time in milliseconds
    pub p99_ms: f64,
    
    /// Standard deviation in milliseconds
    pub std_dev_ms: f64,
    
    /// Current FPS (frames per second)
    pub fps: f64,
    
    /// Number of slow frames detected
    pub slow_frames: usize,
    
    /// Percentage of frames that were slow
    pub slow_frame_percentage: f64,
    
    /// Average time of slow frames in milliseconds
    pub avg_slow_time_ms: f64,
    
    /// Configured window size
    pub window_size: usize,
    
    /// Total frames recorded (including rotated out)
    pub total_frames: usize,
}

impl RenderStats {
    /// Create empty stats.
    pub fn empty() -> Self {
        Self {
            count: 0,
            avg_ms: 0.0,
            min_ms: 0.0,
            max_ms: 0.0,
            median_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
            std_dev_ms: 0.0,
            fps: 0.0,
            slow_frames: 0,
            slow_frame_percentage: 0.0,
            avg_slow_time_ms: 0.0,
            window_size: 0,
            total_frames: 0,
        }
    }
    
    /// Format stats as a human-readable string.
    pub fn format(&self) -> String {
        if self.count == 0 {
            return "No render data available".to_string();
        }
        
        format!(
            "Render stats (last {} of {} frames, total: {}):\n\
            - FPS: {:.1}\n\
            - Average: {:.2}ms\n\
            - Min/Max: {:.2}ms / {:.2}ms\n\
            - Median: {:.2}ms\n\
            - 95th/99th percentile: {:.2}ms / {:.2}ms\n\
            - Std Dev: {:.2}ms\n\
            - Slow frames: {} ({:.1}%)\n\
            - Avg slow time: {:.2}ms",
            self.count,
            self.window_size,
            self.total_frames,
            self.fps,
            self.avg_ms,
            self.min_ms,
            self.max_ms,
            self.median_ms,
            self.p95_ms,
            self.p99_ms,
            self.std_dev_ms,
            self.slow_frames,
            self.slow_frame_percentage,
            self.avg_slow_time_ms,
        )
    }
    
    /// Check if performance is acceptable
    pub fn is_acceptable(&self, target_fps: f64) -> bool {
        self.fps >= target_fps * 0.9 && // Within 10% of target
        self.slow_frame_percentage < 5.0 && // Less than 5% slow frames
        self.p95_ms < (1000.0 / target_fps) * 1.5 // 95th percentile within 50% of frame budget
    }
}

/// Performance anomaly types
#[derive(Debug, Clone)]
pub enum PerformanceAnomaly {
    /// Average frame time exceeds threshold
    SlowAverage(f64, f64),
    
    /// High variance in frame times
    HighVariance(f64, f64),
    
    /// Many frames are slow
    ManySlowFrames(f64),
}

/// Timer for measuring render duration.
pub struct RenderTimer<'a> {
    metrics: &'a RenderMetrics,
    start: Instant,
}

impl<'a> RenderTimer<'a> {
    /// Finish timing and record the duration.
    pub fn finish(self) {
        let duration = self.start.elapsed();
        self.metrics.record_render(duration);
    }
    
    /// Finish timing and return the duration without recording.
    pub fn finish_without_record(self) -> Duration {
        self.start.elapsed()
    }
}

impl<'a> Drop for RenderTimer<'a> {
    fn drop(&mut self) {
        // Auto-record on drop if not already done
        if !std::thread::panicking() {
            let duration = self.start.elapsed();
            self.metrics.record_render(duration);
        }
    }
}
