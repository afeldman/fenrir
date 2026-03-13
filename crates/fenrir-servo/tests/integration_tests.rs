//! Integrationstest für Render-Metriken

#[cfg(test)]
mod integration_tests {
    use std::time::Duration;
    use fenrir_servo::{RenderMetrics, RenderMetricsConfig};
    
    #[test]
    fn test_basic_render_metrics() {
        let metrics = RenderMetrics::with_defaults();
        
        // Einige Testdaten
        metrics.record_render(Duration::from_millis(16));
        metrics.record_render(Duration::from_millis(18));
        metrics.record_render(Duration::from_millis(14));
        metrics.record_render(Duration::from_millis(20));
        metrics.record_render(Duration::from_millis(16));
        
        let stats = metrics.calculate_stats();
        
        assert_eq!(stats.count, 5);
        assert!(stats.avg.as_millis() >= 14 && stats.avg.as_millis() <= 20);
        assert_eq!(stats.min, Duration::from_millis(14));
        assert_eq!(stats.max, Duration::from_millis(20));
        assert_eq!(stats.window_size, 100);
    }
    
    #[test]
    fn test_window_rotation() {
        let config = RenderMetricsConfig {
            window_size: 10,  // Kleines Fenster für Test
            auto_log: false,
            log_interval: 0,
        };
        
        let metrics = RenderMetrics::new(config);
        
        // Mehr Renderings als Fenstergröße
        for i in 0..15 {
            metrics.record_render(Duration::from_millis(i as u64));
        }
        
        let stats = metrics.calculate_stats();
        
        // Sollte nur die letzten 10 Renderings enthalten
        assert_eq!(stats.count, 10);
        assert_eq!(stats.min, Duration::from_millis(5));  // Werte 5-14
        assert_eq!(stats.max, Duration::from_millis(14));
    }
    
    #[test]
    fn test_render_timer() {
        let metrics = RenderMetrics::with_defaults();
        
        {
            let timer = metrics.start_timing();
            std::thread::sleep(Duration::from_millis(10));
            timer.finish();
        }
        
        let stats = metrics.calculate_stats();
        assert_eq!(stats.count, 1);
        assert!(stats.avg.as_millis() >= 10);
    }
    
    #[test]
    fn test_clear_metrics() {
        let metrics = RenderMetrics::with_defaults();
        
        metrics.record_render(Duration::from_millis(16));
        metrics.record_render(Duration::from_millis(18));
        
        assert_eq!(metrics.calculate_stats().count, 2);
        
        metrics.clear();
        
        assert_eq!(metrics.calculate_stats().count, 0);
        assert_eq!(metrics.frame_count(), 0);
    }
    
    #[test]
    fn test_frame_count() {
        let metrics = RenderMetrics::with_defaults();
        
        assert_eq!(metrics.frame_count(), 0);
        
        metrics.record_render(Duration::from_millis(16));
        metrics.record_render(Duration::from_millis(18));
        
        assert_eq!(metrics.frame_count(), 2);
        
        // Auch nach Fensterrotation sollte frame_count weiter steigen
        let config = RenderMetricsConfig {
            window_size: 3,
            auto_log: false,
            log_interval: 0,
        };
        
        let metrics2 = RenderMetrics::new(config);
        
        for i in 0..10 {
            metrics2.record_render(Duration::from_millis(i as u64));
        }
        
        assert_eq!(metrics2.frame_count(), 10);
        assert_eq!(metrics2.calculate_stats().count, 3); // Nur letzte 3 im Fenster
    }
}
