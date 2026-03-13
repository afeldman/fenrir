//! Example demonstrating the new separated architecture:
//! - fenrir-rendering: Pure rendering logic
//! - fenrir-metrics: Performance monitoring
//! - fenrir-servo: Integration layer

use std::sync::Arc;
use std::time::Duration;
use fenrir_metrics::render::{RenderMetrics, RenderMetricsConfig, RenderStats};
use fenrir_metrics::config::MetricsConfigBuilder;

/// Example showing how to use the separated architecture
struct PerformanceMonitor {
    render_metrics: Arc<RenderMetrics>,
}

impl PerformanceMonitor {
    fn new() -> Self {
        // Configure metrics for development
        let config = MetricsConfigBuilder::for_development()
            .build()
            .render;
        
        let render_metrics = Arc::new(RenderMetrics::new(config));
        
        Self { render_metrics }
    }
    
    fn simulate_rendering(&self, frames: usize) {
        println!("Simulating {} frames...", frames);
        
        for i in 0..frames {
            // Simulate varying frame times
            let base_time = 16.67; // 60 FPS target
            let variation = (i as f64 % 30.0) * 0.3;
            let jitter = if i % 50 == 0 { 20.0 } else { 0.0 }; // Occasional slow frame
            
            let frame_time = Duration::from_micros(
                ((base_time + variation + jitter) * 1000.0) as u64
            );
            
            self.render_metrics.record_render(frame_time);
            
            // Log progress
            if (i + 1) % 25 == 0 {
                let stats = self.render_metrics.calculate_stats();
                println!(
                    "Frame {}: {:.1} FPS, Avg: {:.2}ms",
                    i + 1,
                    stats.fps,
                    stats.avg.as_secs_f64() * 1000.0
                );
                
                // Check for anomalies
                let anomalies = self.render_metrics.check_anomalies();
                if !anomalies.is_empty() {
                    println!("  ⚠️  Anomalies detected: {:?}", anomalies);
                }
            }
        }
        
        println!("\nSimulation complete!");
        self.print_detailed_stats();
    }
    
    fn print_detailed_stats(&self) {
        let stats = self.render_metrics.calculate_stats();
        
        println!("\n=== Detailed Render Statistics ===");
        println!("{}", stats.format());
        
        println!("\n=== Performance Analysis ===");
        println!("Acceptable for 60 FPS: {}", stats.is_acceptable(60.0));
        println!("Acceptable for 30 FPS: {}", stats.is_acceptable(30.0));
        
        if let Some(prediction) = self.render_metrics.predict_next_frame() {
            println!("Predicted next frame: {:.2}ms", prediction.as_secs_f64() * 1000.0);
        }
        
        // Export as JSON
        if let Ok(json) = self.render_metrics.export_json() {
            println!("\n=== JSON Export (first 500 chars) ===");
            println!("{}...", &json[..json.len().min(500)]);
        }
    }
    
    fn run_realtime_monitoring(&self) {
        println!("Starting real-time monitoring (simulated)...");
        println!("Press Ctrl+C to stop.\n");
        
        let mut last_total_frames = 0;
        
        // Simulate monitoring loop
        for i in 0..10 {
            std::thread::sleep(Duration::from_secs(2));
            
            let current_stats = self.render_metrics.calculate_stats();
            let frames_since_last = current_stats.total_frames - last_total_frames;
            
            if frames_since_last > 0 {
                println!(
                    "[Monitor Update {}] FPS: {:.1}, Avg: {:.2}ms, Frames: {}",
                    i + 1,
                    current_stats.fps,
                    current_stats.avg.as_secs_f64() * 1000.0,
                    frames_since_last
                );
                
                // Performance alerts
                if current_stats.fps < 30.0 {
                    println!("  🔴 ALERT: Low FPS ({:.1} < 30)", current_stats.fps);
                } else if current_stats.fps < 50.0 {
                    println!("  🟡 WARNING: Moderate FPS ({:.1} < 50)", current_stats.fps);
                }
            }
            
            last_total_frames = current_stats.total_frames;
        }
    }
}

fn main() -> anyhow::Result<()> {
    println!("=== Fenrir Separated Architecture Demo ===");
    println!("This demonstrates the new architecture with separate crates for:");
    println!("1. fenrir-rendering: Pure rendering logic");
    println!("2. fenrir-metrics: Performance monitoring");
    println!("3. fenrir-servo: Integration layer\n");
    
    let monitor = PerformanceMonitor::new();
    
    // Phase 1: Basic simulation
    println!("--- Phase 1: Basic Simulation ---");
    monitor.simulate_rendering(75);
    
    // Phase 2: Clear and test window rotation
    println!("\n--- Phase 2: Window Rotation Test ---");
    monitor.render_metrics.clear();
    monitor.simulate_rendering(150); // More than default window size (100)
    
    // Phase 3: Real-time monitoring simulation
    println!("\n--- Phase 3: Real-time Monitoring ---");
    monitor.run_realtime_monitoring();
    
    // Phase 4: Different configurations
    println!("\n--- Phase 4: Configuration Examples ---");
    
    // Development config
    let dev_config = MetricsConfigBuilder::for_development().build();
    println!("Development config: {:?}", dev_config.render);
    
    // Production config  
    let prod_config = MetricsConfigBuilder::for_production().build();
    println!("Production config: {:?}", prod_config.render);
    
    // Performance testing config
    let perf_config = MetricsConfigBuilder::for_performance_testing().build();
    println!("Performance testing config: {:?}", perf_config.render);
    
    println!("\n=== Demo Complete ===");
    println!("The architecture successfully separates:");
    println!("- Rendering logic (fenrir-rendering)");
    println!("- Metrics collection (fenrir-metrics)");
    println!("- Integration (fenrir-servo)");
    println!("\nBenefits:");
    println!("✅ Clean separation of concerns");
    println!("✅ Reusable metrics across components");
    println!("✅ Easier testing and maintenance");
    println!("✅ Flexible configuration per environment");
    
    Ok(())
}
